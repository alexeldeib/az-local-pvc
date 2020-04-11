#[macro_use]
extern crate crossbeam_channel;
use crossbeam_channel::tick;

use std::fs;
use std::io::{self, Error, ErrorKind};
use std::process::{self, Command, Output};
use std::time::Duration;

fn main() {
    println!("started binary");
    println!("starting first run");
    let mut result = main_loop();
    
    println!("beginning ticker");
    let ticker = tick(Duration::from_secs(5));
    while let Ok(_) = result {
        select! {
            recv(ticker) -> _ => result = main_loop(),
        }
    }

    println!("failed in main loop: {:?}", result);
    process::exit(1);
}

fn main_loop() -> io::Result<()> {
    println!("starting main loop");
    let block_devices = get_block_devices()
        .expect("failed to list block devices")
        .into_iter()
        .filter(|dev| dev.contains("nvme"))
        .collect::<Vec<String>>();

    println!("finished discovering block devices");

    println!("enumerating known devices");
    for dev in block_devices {
        let needs_fs = should_format(&dev)?;
        let uuid = get_uuid(&dev).unwrap_or(String::from(""));
        println!("found disk /dev/{} with existing uuid '{}'", dev, uuid);
        if needs_fs {
            println!("formatting disk /dev/{}", dev);
            format_device(&dev)?;
        }
        mount_device(&dev)?;
    }
    println!("successfully scanned and formatted all disks");
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