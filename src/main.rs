use std::fs;
use std::io::{self, Error, ErrorKind};
use std::process::{self, Command, Output};
use std::str::FromStr;
use std::sync::Mutex;
use std::time::Duration;

use slog_atomic::{AtomicSwitch};
use slog::{Drain, o, info};
use clap::{value_t, Arg, App};
use crossbeam_channel::{select, tick};

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
        .about("Teaches argument parsing")
        .arg(Arg::with_name("output")
                 .short('o')
                 .long("output")
                 .takes_value(true)
                 .required(false)
                 .possible_values(&["json", "text"])
                 .help("Output format"))
        .get_matches();

    // let drain = slog_json::Json::default(std::io::stderr()).fuse();
    // let drain = slog_async::Async::new(drain).build().fuse();
    // let drain = AtomicSwitch::new(drain);
    // let ctrl = drain.ctrl();
    // let log = slog::Logger::root(drain.fuse(), o!());

    let log_format = value_t!(matches.value_of("output"), LogFormat).unwrap_or(LogFormat::Json);

    let log = match log_format {
        LogFormat::Json => {
            let drain = slog_json::Json::default(std::io::stderr()).fuse();
            let drain = Mutex::new(slog_async::Async::new(drain).build().fuse());
            let drain = AtomicSwitch::new(drain);
            slog::Logger::root(drain.fuse(), o!())
        },
        LogFormat::Text => {
            let decorator = slog_term::TermDecorator::new().build();
            let drain = Mutex::new(slog_term::FullFormat::new(decorator).build());
            let drain = AtomicSwitch::new(drain);
            slog::Logger::root(drain.fuse(), o!())
        },
    };

    // if log_format == LogFormat::Text {
    //     let decorator = slog_term::TermDecorator::new().build();
    //     let drain = Mutex::new(slog_term::FullFormat::new(decorator).build());
    //     ctrl.set(drain.fuse());
    // }


    info!(log, "started binary");
    info!(log, "starting first run");
    let mut result = main_loop(&log);
    
    info!(log, "beginning ticker");
    let ticker = tick(Duration::from_secs(5));
    while let Ok(_) = result {
        select! {
            recv(ticker) -> _ => result = main_loop(&log),
        }
    }

    info!(log, "failed in main loop: {:?}", result);
    process::exit(1);
}

fn main_loop(log: &slog::Logger) -> io::Result<()> {
    info!(log, "starting main loop");
    let block_devices = get_block_devices()
        .expect("failed to list block devices")
        .into_iter()
        .filter(|dev| dev.contains("nvme"))
        .collect::<Vec<String>>();

    info!(log, "finished discovering block devices");

    info!(log, "enumerating known devices");
    for dev in block_devices {
        let needs_fs = should_format(&dev)?;
        let uuid = get_uuid(&dev).unwrap_or(String::from(""));
        info!(log, "found disk /dev/{} with existing uuid '{}'", dev, uuid);
        if needs_fs {
            info!(log, "formatting disk /dev/{}", dev);
            format_device(&dev)?;
        }
        mount_device(&dev)?;
    }
    info!(log, "successfully scanned and formatted all disks");
    Ok(())
}

fn get_block_devices() -> io::Result<Vec<String>> {
    fs::read_dir("/sys/block")?
        .map(|res| res.map(|e| e.file_name().into_string().unwrap()))
        .collect()
}

fn get_uuid(path: &str) -> io::Result<String> {
    let output = Command::new("blkid")
        .arg("-o")
        .arg("value")
        .arg("-s")
        .arg("UUID")
        .arg(format!("/dev/{}", path))
        .output();

    match output {
        Ok(out) => match out.stdout.is_empty() {
            true => Err(Error::new(ErrorKind::Other, "expected uuid for disk")),
            false => match String::from_utf8(out.stdout) {
                Ok(uuid) => Ok(uuid),
                Err(e) => Err(Error::new(ErrorKind::Other, e)),
            },
        },
        Err(e) => Err(e),
    }
}

fn should_format(path: &str) -> io::Result<bool> {
    let output = Command::new("blkid")
        .arg("-o")
        .arg("value")
        .arg("-s")
        .arg("UUID")
        .arg(format!("/dev/{}", path))
        .output();

    match output {
        Ok(out) => Ok(out.stdout.is_empty()),
        Err(e) => Err(Error::new(ErrorKind::Other, e)),
    }
}

fn format_device(path: &str) -> io::Result<Output> {
    Command::new("mke2fs")
        .arg("-t")    
        .arg("ext4")
        .arg(format!("/dev/{}", path))
        .output()
}

fn mount_device(path: &str) -> io::Result<Output> {    
    // TODO(ace): don't shell out for this
    Command::new("mkdir")    
        .arg("-p")
        .arg(format!("/mnt/pv-disks/{}", &path))
        .output()?;

    Command::new("mount")
        .arg(format!("/dev/{}", &path))
        .arg(format!("/mnt/pv-disks/{}", &path))
        .output()
}
