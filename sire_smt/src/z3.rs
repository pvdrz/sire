use std::io::{Read, Write};
use std::process::{Command, Stdio};

pub fn call(code: &str) -> Result<String, Box<dyn std::error::Error>> {
    let mut buffer = String::new();

    let child =
        Command::new("z3").arg("-in").stdin(Stdio::piped()).stdout(Stdio::piped()).spawn()?;

    child.stdin.expect("stdin is none").write(code.as_bytes())?;

    child.stdout.expect("stdout is none").read_to_string(&mut buffer)?;

    Ok(buffer)
}
