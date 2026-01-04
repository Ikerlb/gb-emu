mod gb;

use clap::Parser;
use gb::gameboy::*;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

/// Game Boy emulator
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the ROM file to load
    #[arg(short, long, value_name = "FILE")]
    file_path: PathBuf,
}

fn main() {
    let args = Args::parse();
    let file_name = args.file_path;
    let file_buf = load_file(file_name);
    let mut gb = GameBoy::new(file_buf);
    gb.run();
}

fn load_file(file_name: PathBuf) -> Vec<u8> {
    let mut file = File::open(file_name).unwrap();
    let mut file_buf = Vec::new();
    file.read_to_end(&mut file_buf).unwrap();
    file_buf
}
