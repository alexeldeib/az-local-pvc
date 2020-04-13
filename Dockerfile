FROM rust:1.42-buster as build

ARG APP="az-local-pvc"
ENV APP="${APP}"

ENV PKG_CONFIG_ALLOW_CROSS=1
ENV PKG_CONFIG_PATH=/usr/lib/x86_64-linux-gnu/pkgconfig

RUN apt update && apt install -y libudev-dev pkg-config

# setup util-linux for full versions of blkid, mount, and mkfs
RUN apt install -y bison autopoint gettext
RUN git clone git://git.kernel.org/pub/scm/utils/util-linux/util-linux.git /app/src/util-linux
WORKDIR /app/src/util-linux
RUN ./autogen.sh
RUN ./configure \
    LDFLAGS="-static" \
    --disable-rpath \
    --disable-makeinstall-chown \
    --enable-static-programs \
    --enable-mount \
    --enable-blkid
RUN make LDFLAGS="-all-static" -j$(nproc) && DESTDIR=/app/util-linux make -j$(nproc) install
RUN wc -c blkid.static | numfmt --to=iec-i 
RUN wc -c mount.static | numfmt --to=iec-i 

# file because it's useful on block devices
RUN git clone https://github.com/file/file /app/src/file
WORKDIR /app/src/file
RUN autoreconf -i && ./configure LDFLAGS="-static" && make LDFLAGS="-all-static" -j$(nproc) && DESTDIR=/app/file make install
RUN wc -c src/file | numfmt --to=iec-i

RUN git clone git://git.kernel.org/pub/scm/fs/ext2/e2fsprogs.git /app/src/e2fsprogs
WORKDIR /app/src/e2fsprogs
RUN ./configure \
    LDFLAGS="-static" \
    --enable-lto \
    --disable-rpath
RUN make LDFLAGS="--static" -j$(nproc) && DESTDIR=/app/e2fsprogs make -j$(nproc) install

RUN rustup target add x86_64-unknown-linux-musl

# Create a dummy project and build the app's dependencies.
# If the Cargo.toml or Cargo.lock files have not changed,
# we can use the docker build cache and skip these (typically slow) steps.
WORKDIR /app/src
RUN USER=root cargo new "${APP}"
WORKDIR "/app/src/${APP}"

# Copy the source and build the application.
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release --target x86_64-unknown-linux-musl

# Fix bugs in distroless busybox; simple `ls` doesn't work.
FROM amd64/busybox:uclibc as busybox
FROM gcr.io/distroless/cc:debug
COPY --from=busybox /bin/busybox /busybox/busybox
RUN ["/busybox/busybox", "--install", "/bin"]

# Copy proper versions of util-linux...might as well build our own distro.
COPY --from=build /app/src/az-local-pvc/target/x86_64-unknown-linux-musl/release/az-local-pvc .
COPY --from=build /app/util-linux/bin/mount.static /usr/local/bin/mount.static
COPY --from=build /app/util-linux/bin/umount.static /usr/local/bin/umount.static
COPY --from=build /app/util-linux/sbin/blkid.static /usr/local/bin/blkid.static
COPY --from=build /app/util-linux/sbin/mkfs /usr/local/bin/mkfs
COPY --from=build /app/e2fsprogs/sbin/* /usr/local/bin/
COPY --from=build /app/e2fsprogs/etc/* /etc/
COPY --from=build /app/file/usr/local/bin/file /usr/local/bin/file
COPY --from=build /app/file/usr/local/share/misc/magic.mgc usr/local/share/misc/magic.mgc

ENTRYPOINT ["./az-local-pvc"]
