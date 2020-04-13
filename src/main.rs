use std::fs;
use std::io::{self, BufRead, BufReader, Error, ErrorKind};
use std::process::{Command, Output};
use std::str::FromStr;
use std::sync::Mutex;
use std::time::Duration;

use clap::{value_t, App, Arg};
use crossbeam_channel::{select, tick};
use slog::{error, info, o, Drain, FnValue, Record};
use slog_atomic::AtomicSwitch;

#[derive(Debug, PartialEq)]
enum LogFormat {
    Json,
    Text,
}

impl FromStr for LogFormat {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "json" => Ok(LogFormat::Json),
            "text" => Ok(LogFormat::Text),
            _ => Err("no match"),
        }
    }
}

fn main() {
    let matches = App::new("az-local-pvc")
        .version("0.0.1-alpha.0")
        .author("Alexander Eldeib <alexeldeib@gmail.com>")
        .about("formats and mounts nvme drives for use")
        .arg(
            Arg::with_name("output")
                .short('o')
                .long("output")
                .takes_value(true)
                .required(false)
                .possible_values(&["json", "text"])
                .help("Output format"),
        )
        .get_matches();

    let log_format = value_t!(matches.value_of("output"), LogFormat).unwrap_or(LogFormat::Json);

    let log = match log_format {
        LogFormat::Json => {
            let drain = slog_json::Json::default(io::stderr()).fuse();
            let drain = Mutex::new(slog_async::Async::new(drain).build().fuse());
            let drain = AtomicSwitch::new(drain);
            slog::Logger::root(drain.fuse(), o!())
        }
        LogFormat::Text => {
            let decorator = slog_term::TermDecorator::new().build();
            let drain = Mutex::new(slog_term::FullFormat::new(decorator).build());
            let drain = AtomicSwitch::new(drain);
            slog::Logger::root(drain.fuse(), o!())
        }
    };

    info!(log, "started binary");
    info!(log, "starting first run");
    let mut result = work(&log);

    info!(log, "beginning ticker");
    let ticker = tick(Duration::from_secs(5));
    while let Ok(_) = result {
        select! {
            recv(ticker) -> _ => result = work(&log),
        }
    }

    info!(log, "failed in main loop: {:?}", result);
}


fn work(log: &slog::Logger) -> io::Result<()> {
    // read block devices from sysfs.
    // TODO(ace): we ignore failed conversions from OsString -> String (maybe can avoid?)
    let dirs: Vec<String> = match fs::read_dir("/sys/block") {
        Err(e) => return Err(e),
        Ok(o) => o
            .map(|res| res.map(|e| e.file_name().into_string()))
            .filter_map(|c| c.ok())
            .map(|res| res.unwrap())
            .filter(|dev| dev.contains("nvme"))
            .collect(),
    };

    for path in dirs {
        // get uuid via blkid, if empty needs to be formatted
        let output = Command::new("blkid")
            .arg("-o")
            .arg("value")
            .arg("-s")
            .arg("UUID")
            .arg(format!("/dev/{}", path))
            .output()?;

        // executed, but no UUID. needs to be formatted
        if !output.status.success() || output.stdout.is_empty() {
            if let Err(e) = Command::new("mkfs.ext4")
                .arg(format!("/dev/{}", path))
                .output()
            {
                return Err(e);
            };
        }


        let uuid = match String::from_utf8(output.stdout) {
            Err(e) => return Err(Error::new(ErrorKind::Other, e)),
            Ok(uuid) => uuid,
        };
        let uuid = uuid.trim_right();
        info!(log, "{:?}", uuid);

        let desired_mount = String::from(format!("/mnt/pv-disks/{}", &uuid));

        if let Err(e) = Command::new("mkdir").arg("-p").arg(&desired_mount).output() {
            return Err(e);
        };

        let mounts: Vec<String> = match Command::new("mount.static").output() {
            Err(e) => return Err(e),
            Ok(o) => {
                if !o.status.success() {
                    return Err(Error::new(
                        ErrorKind::Other,
                        "failed to execute mount, should never happen",
                    ));
                }
                match String::from_utf8(o.stdout) {
                    Err(e) => return Err(Error::new(ErrorKind::Other, e)),
                    Ok(o) => o
                        .lines()
                        .filter(|line| line.find(&path).is_some())
                        .map(|line| String::from(line))
                        .collect(),
                }
            }
        };

        match mounts.len() {
            0 => {
                if let Err(e) = Command::new("mount.static")
                    .arg(format!("/dev/{}", &path))
                    .arg(&desired_mount)
                    .output()
                {
                    return Err(e);
                };
            }
            1 => match mounts[0].as_str() {
                desired_mount => {
                    info!(
                        log,
                        "{}",
                        format!(
                            "already correctly mounted disk {:#?}, uuid: {:#?}",
                            &path, &uuid),
                        )
                    );
                    continue;
                }
                other => {
                    match Command::new("umount.static")
                        .arg(format!("/dev/{}", &path))
                        .output()
                    {
                        Err(e) => return Err(e),
                        Ok(out) => {
                            if !out.status.success() {
                                return Err(Error::new(ErrorKind::Other, format!("failed to unmount wrongly mounted device -- stdout: {:?} -- stderr: {:?}", out.stdout, out.stderr)));
                            }
                            Command::new("mount.static")
                                .arg(format!("/dev/{}", &path))
                                .arg(&desired_mount)
                                .output()
                                .map(|_| ())?
                        }
                    }
                }
            },
            _ => {
                error!(
                    log,
                    "{}",
                    format!("found multiple mountpoints for disk: {:#?}", &path)
                );
            }
        }
    }
    Ok(())
}
