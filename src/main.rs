// use std::io::{self, Write};
// use std::process::Command;
use block_utils::*;
use std::collections::HashSet;
use std::ffi::OsString;
use std::fs;
use std::io;

fn main() -> io::Result<()> {
    let block_devices = get_block_devices().expect("failed to list block devices");
    let mounted_devices = get_mounted_devices().expect("failed to list mounted block devices");
    let mut unmounted_devices: HashSet<String> = HashSet::new();

    println!("mounted");
    // println!("{:#?}", mounted_devices);
    for dev in &mounted_devices {
        println!("{:?}", dev.name);
    }

    println!("block");
    println!("{:#?}", block_devices);
    // for dev in block_devices {
    //     println!("{:?}", dev);
    // }

    // for dev in &block_devices {
    //     let name = dev
    //         .to_str()
    //         .expect("failed to convert path as valid UTF8")
    //         .split("/")
    //         .collect::<Vec<&str>>()[2];
    //     unmounted_devices.insert(name.to_string());
    // }
    // for dev in mounted_devices {
    //     unmounted_devices.remove(&dev.name);
    // }

    // println!("{:#?}", unmounted_devices);

    Ok(())
}

fn get_block_devices() -> Result<Vec<String>, io::Error> {
    fs::read_dir("/sys/block")?
        .map(|res| res.map(|e| e.file_name().into_string().unwrap()))
        .filter_map(|res| {
            res.map(|path| {
                if !path.contains(&"loop") && path != "sr0" {
                    Some(path)
                } else {
                    None
                }
            })
            .transpose()
        })
        .collect()
}
