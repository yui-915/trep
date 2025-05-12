use clap::Parser;
use std::collections::HashMap;
use std::io::{Read, Write};

const BUF_SIZE: usize = 1024 * 1024;

#[derive(Parser)]
struct Cli {
    map_path: std::path::PathBuf,
    command: Vec<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let child = std::process::Command::new("script")
        .args(["-c", &cli.command.join(" "), "-q", "-e", "/dev/null"])
        .stdout(std::process::Stdio::piped())
        .spawn()?;

    let map_source = std::fs::read_to_string(cli.map_path)?;
    let map: HashMap<String, String> = toml::from_str(&map_source)?;
    let map: HashMap<Vec<u8>, Vec<u8>> = map
        .into_iter()
        .map(|(k, v)| (k.bytes().collect(), v.bytes().collect()))
        .collect();

    let mut buf = vec![];
    let mut cout = child.stdout.unwrap();
    let mut stdout = std::io::stdout();
    loop {
        buf.resize(BUF_SIZE, 0);
        let read = cout.read(&mut buf)?;
        buf.truncate(read);
        if buf.is_empty() {
            return Ok(());
        }
        while !buf.is_empty() {
            if let Some((k, v)) = map
                .iter()
                .find(|(k, _)| (buf.len() >= k.len() && buf[..k.len()] == **k))
            {
                stdout.write_all(v)?;
                buf.drain(0..k.len());
            } else {
                stdout.write_all(&[buf[0]])?;
                buf.remove(0);
            };
        }
        stdout.flush()?;
    }
}
