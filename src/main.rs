#[macro_use]
extern crate enum_primitive;

#[macro_use]
extern crate num;

mod gb;

use gb::gameboy::*;

fn main(){
	let a: [u8;4]=[1,2,3,4];
	let mut gb=GameBoy::new(Box::new(a));
	println!("{}",gb);
}