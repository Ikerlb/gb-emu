mod gb;

use gb::gameboy::*;

fn main(){
	//let cpu=Cpu::new();
	//println!("{:#?}",cpu);
	let a: [u8;4]=[1,2,3,4];
	let gb=GameBoy::new(Box::new(a));
	println!("{:#?}",gb);
}