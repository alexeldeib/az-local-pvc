use std::io::{self, Write};
use std::process::Command;

fn main() {
    let output = Command::new("cmd")
        .arg("ls")
        .output()
        .expect("failed to execute process");
    let _ = io::stdout().write_all(&output.stdout);
}
