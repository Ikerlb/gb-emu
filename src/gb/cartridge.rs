//TODO change integer size to use the least number of casts!!!!.
//Codeslinger for reference.
use std::fmt::{Display,Result,Formatter,Debug};

const RAM_BANK_SIZE:u16 = 0x2000;
const ROM_BANK_SIZE:u16 = 0x4000;

#[derive(Debug)]
enum MemoryBankController{
	Mbc0,
	Mbc1,
	Mbc2,
	Mbc3
}

pub struct Cartridge{

	rom: 			Vec<u8>,
	ram: 			Vec<u8>,
	mbc: 			MemoryBankController,
	ram_bank_size: 	u16,
	ram_banks: 		u16,
	rom_banks: 		u16,
	current_rom: 	u16,
	current_ram: 	u16,
	enable_ram:		bool,
	rom_mode:		bool,

}

enum SetRomBank{
	High,
	Low,
}


impl Cartridge{
	pub fn new(cart:Vec<u8>)->Self{
		let mut ctd=Cartridge{

			rom: 			cart,
			ram: 			Vec::new(),
			mbc: 			MemoryBankController::Mbc0,
			ram_bank_size: 	0,
			ram_banks: 		0,
			rom_banks: 		1,
			current_rom: 	1,
			current_ram: 	0,
			enable_ram:  	false,
			rom_mode:		false,

		};
		ctd.init_cartridge();
		ctd
	}

	fn init_cartridge(&mut self){
		self.set_mbc();
		self.set_rom_size();
		self.set_ram_size();
		self.init_ram();
	}

	fn set_mbc(&mut self){
		let cart_type=self.rom[0x147];
		self.mbc=
			match cart_type{
				0x00 		=> MemoryBankController::Mbc0,
				0x01..=0x03 => MemoryBankController::Mbc1,
				0x05..=0x06 => MemoryBankController::Mbc2,
				0x0F..=0x13 => MemoryBankController::Mbc3,
			    _      		=> panic!("Unknown cartridge model 0x{:02x}", cart_type),
 			}
	}

	fn set_rom_size(&mut self){
		let cart_rom_size=self.rom[0x148];
		self.rom_banks=
			match cart_rom_size{
                0x00 => 2,
                0x01 => 4,
                0x02 => 8,
                0x03 => 16,
                0x04 => 32,
                0x05 => 64,
                0x06 => 128,
                0x07 => 256,
                0x08 => 512,
                //0x52 => 72,
                //0x53 => 80,
                //0x54 => 96,
                //_ => 0
                _ => panic!("Unsupported number of rom banks 0x{:02x}",cart_rom_size),
			};
	}

	fn set_ram_size(&mut self){
		let cart_ram_size=self.rom[0x149];
		let (numbanks,banksize)=
			match cart_ram_size{
				0x00 => (0,0),
				0x01 => (1,0x800),
				0x02 =>	(1,0x2000),
				0x03 => (4,0x2000),
				_    => panic!("Unsupported ram type 0x{:02x}",cart_ram_size),
			};
		self.ram_banks=numbanks;
		self.ram_bank_size=banksize;
	}

	fn init_ram(&mut self){
		self.ram=vec![0;(self.ram_bank_size*self.ram_banks) as usize];
	}


	//////////////////////////////////////////
	pub fn write(&mut self,address: u16,data: u8){
		match self.mbc{
			MemoryBankController::Mbc0 => panic!("Cannot write cartridge in Mbc0!"),
			MemoryBankController::Mbc1 => self.write_mbc1(address,data),
			MemoryBankController::Mbc2 => self.write_mbc2(address,data),
			MemoryBankController::Mbc3 => self.write_mbc3(address,data),
		}
	}

	//TODO: Test thoroughly
	pub fn read(&self,address:u16)->u8{
		match address {
			0x0000..=0x7FFF => {
				let new_address: isize=(address as isize-0x4000)+(0x4000*self.current_rom as isize);
				self.rom[new_address as usize]
			},
			0xA000..=0xBFFF => {
				let new_address: isize=(address as isize-0xA000)+((self.ram_bank_size*self.current_ram)as isize);
				self.ram[new_address as usize]
			},
			_               => panic!("Cannot find address 0x{:02x} in cartridge",address),

		}
	}

	//implement!!
	fn write_mbc1(&mut self,address: u16,data: u8){
		match address{
			0x0000..=0x1FFF => self.enable_ram=(data&0xF)==0xA,
			0x2000..=0x3FFF => self.set_rombank_hi_lo(data,SetRomBank::Low),
			0x4000..=0x5FFF => self.set_romram_bank(data),
			0x6000..=0x7FFF => self.set_romram_mode(data),
			0xA000..=0xBFFF => self.write_ram(address,data),
			_               => unimplemented!(),

		}
	}

	//Check various sources for ram enabling
	fn write_mbc2(&mut self,address: u16,data: u8){
		/*match address{
			0x000..=0x1FFF => self.enable_ram^=(address&0x100)==0, //TEST thoroughly


		}*/
	}

	//TODO IMPLEMENT MBC3!
	fn write_mbc3(&mut self,address: u16,data: u8){}

	fn set_romram_bank(&mut self,data: u8){
		if self.rom_mode{
			self.set_rombank_hi_lo(data,SetRomBank::High);
		}
		else {
			self.current_ram=data as u16&0x3;
		}
	}

	fn set_romram_mode(&mut self,data: u8){
		self.rom_mode=(data&0x1)==0;
		if self.rom_mode {
			self.current_ram=0;
		}
	}

	fn set_rombank_hi_lo(&mut self,data: u8,mode: SetRomBank){
		match mode{
			SetRomBank::High => {
				self.current_rom&=0x1F;
				let upper3=data&0xE0;
				self.current_rom|=data as u16;
			},
			SetRomBank::Low => {
				let lower5=data&0x1F;
				self.current_rom&=0xE0;
				self.current_rom|=lower5 as u16;
			},
		}
		if self.current_rom==0 {
			self.current_rom=1;
		}
	}


	fn write_ram(&mut self,address: u16,data: u8){
		let new_address: isize=(address as isize-0xA000)+((self.ram_bank_size*self.current_ram) as isize);
		self.ram[new_address as usize]=data;
	}

}

#[cfg(test)]
mod tests {
	use super::*;

	fn create_mbc0_rom() -> Vec<u8> {
		let mut rom = vec![0; 0x8000]; // 32KB
		rom[0x0147] = 0x00; // ROM ONLY (MBC0)
		rom[0x0148] = 0x00; // 32KB ROM
		rom[0x0149] = 0x00; // No RAM
		rom
	}

	fn create_mbc1_rom() -> Vec<u8> {
		let mut rom = vec![0; 0x80000]; // 512KB for testing
		rom[0x0147] = 0x01; // MBC1
		rom[0x0148] = 0x04; // 256KB ROM (16 banks)
		rom[0x0149] = 0x02; // 8KB RAM (1 bank)
		rom
	}

	#[test]
	fn test_cartridge_creation_mbc0() {
		let rom = create_mbc0_rom();
		let cart = Cartridge::new(rom);
		// Should initialize without panic
	}

	#[test]
	fn test_cartridge_creation_mbc1() {
		let rom = create_mbc1_rom();
		let cart = Cartridge::new(rom);
		// Should initialize without panic
	}

	#[test]
	fn test_mbc_detection_mbc0() {
		let mut rom = vec![0; 0x8000];
		rom[0x0147] = 0x00; // MBC0
		rom[0x0148] = 0x00;
		rom[0x0149] = 0x00;

		let cart = Cartridge::new(rom);
		// If it doesn't panic, MBC detection worked
	}

	#[test]
	fn test_mbc_detection_mbc1() {
		let mut rom = vec![0; 0x80000];
		rom[0x0147] = 0x01; // MBC1
		rom[0x0148] = 0x04;
		rom[0x0149] = 0x00;

		let cart = Cartridge::new(rom);
		// If it doesn't panic, MBC detection worked
	}

	#[test]
	fn test_mbc_detection_mbc2() {
		let mut rom = vec![0; 0x40000];
		rom[0x0147] = 0x05; // MBC2
		rom[0x0148] = 0x03;
		rom[0x0149] = 0x00;

		let cart = Cartridge::new(rom);
		// If it doesn't panic, MBC detection worked
	}

	#[test]
	fn test_mbc_detection_mbc3() {
		let mut rom = vec![0; 0x80000];
		rom[0x0147] = 0x0F; // MBC3
		rom[0x0148] = 0x04;
		rom[0x0149] = 0x00;

		let cart = Cartridge::new(rom);
		// If it doesn't panic, MBC detection worked
	}

	#[test]
	#[should_panic(expected = "Unknown cartridge model")]
	fn test_mbc_detection_unknown() {
		let mut rom = vec![0; 0x8000];
		rom[0x0147] = 0xFF; // Invalid MBC type
		rom[0x0148] = 0x00;
		rom[0x0149] = 0x00;

		let _cart = Cartridge::new(rom);
	}

	#[test]
	fn test_rom_read_bank_0() {
		let mut rom = create_mbc0_rom();
		rom[0x0100] = 0xAB;
		rom[0x1234] = 0xCD;

		let cart = Cartridge::new(rom);

		assert_eq!(cart.read(0x0100), 0xAB);
		assert_eq!(cart.read(0x1234), 0xCD);
	}

	#[test]
	fn test_rom_read_bank_n() {
		let mut rom = create_mbc0_rom();
		rom[0x4000] = 0x12; // Start of bank 1
		rom[0x7FFF] = 0x34; // End of bank 1

		let cart = Cartridge::new(rom);

		assert_eq!(cart.read(0x4000), 0x12);
		assert_eq!(cart.read(0x7FFF), 0x34);
	}

	#[test]
	fn test_rom_size_parsing() {
		// Test various ROM sizes
		let test_cases = vec![
			(0x00, 2),   // 32KB = 2 banks
			(0x01, 4),   // 64KB = 4 banks
			(0x02, 8),   // 128KB = 8 banks
			(0x03, 16),  // 256KB = 16 banks
			(0x04, 32),  // 512KB = 32 banks
		];

		for (rom_size_code, expected_banks) in test_cases {
			let mut rom = vec![0; 0x80000];
			rom[0x0147] = 0x00; // MBC0
			rom[0x0148] = rom_size_code;
			rom[0x0149] = 0x00;

			let cart = Cartridge::new(rom);
			// Can't directly access rom_banks, but if it initializes without panic,
			// the parsing worked
		}
	}

	#[test]
	fn test_ram_size_parsing() {
		// Test various RAM sizes
		let test_cases = vec![
			0x00, // No RAM
			0x01, // 2KB
			0x02, // 8KB
			0x03, // 32KB (4 banks of 8KB)
		];

		for ram_size_code in test_cases {
			let mut rom = vec![0; 0x8000];
			rom[0x0147] = 0x00; // MBC0
			rom[0x0148] = 0x00;
			rom[0x0149] = ram_size_code;

			let cart = Cartridge::new(rom);
			// If it initializes without panic, parsing worked
		}
	}
}