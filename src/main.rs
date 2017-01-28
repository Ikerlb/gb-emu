#[macro_use]
extern crate enum_primitive;

#[macro_use]
extern crate num;

mod gb;

use gb::gameboy::*;
use std::env;
use std::fs::File;
use std::io::Read;

fn main(){
	let file_name=env::args().nth(1).unwrap();
	let file_buf=load_file(file_name);
	let mut gb=GameBoy::new(file_buf);
	gb.run();
}

fn load_file(file_name: String) -> Vec<u8>{
	let mut file = File::open(file_name).unwrap();
	let mut file_buf = Vec::new();
	file.read_to_end(&mut file_buf).unwrap();
	file_buf
} 