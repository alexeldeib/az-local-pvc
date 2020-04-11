FROM rust:1.42-buster as build
WORKDIR /app

ARG APP="az-local-pvc"
ENV APP="${APP}"

RUN rustup target add x86_64-unknown-linux-musl

# Create a dummy project and build the app's dependencies.
# If the Cargo.toml or Cargo.lock files have not changed,
# we can use the docker build cache and skip these (typically slow) steps.
RUN USER=root cargo new "${APP}"
WORKDIR "/app/${APP}"
COPY Cargo.toml Cargo.lock ./
RUN cargo build --release

# Copy the source and build the application.
COPY src ./src
RUN cargo install --target x86_64-unknown-linux-musl --path .

# Fix bugs in distroless
# FROM amd64/busybox:latest as busybox
FROM gcr.io/distroless/base:debug
# COPY --from=busybox /bin/busybox /busybox/busybox
# RUN ["/busybox/busybox", "--install", "/bin"]

# FROM ubuntu:18.04
# Copy the statically-linked binary into a scratch container.
COPY --from=build /usr/local/cargo/bin/az-local-pvc .
ENTRYPOINT ["./az-local-pvc"]