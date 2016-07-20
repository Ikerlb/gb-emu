use std::fmt::{Display,Result,Formatter};

//Only implements mbc1 and mbc2 which consist of about 98% of gb games
pub struct Interconnect{
	cart_rom: Box<[u8]>, 
	main_mem: [u8; 0x10000],	
	current_rom_bank: u8,
	current_ram_bank: u8,
	ram_banks: [u8;0x8000],
	mbc: usize,    
	enable_ram_banking: bool, 
    rom_banking: bool           //handles wether to change ram bank or 
                                //upper bits of rom bank
}

//Get info on rom banking from cart_rom[0x147]
fn mbc(num:u8) -> usize{
	match num{
        1|2|3 => 1,
        5|6 => 2,
        _ => 0
	}
}

impl Interconnect {

    pub fn new(ct_rom: Box<[u8]>) -> Self{
    	let num=ct_rom[0x147];
		Interconnect{
			cart_rom: ct_rom,
			main_mem: [0;0x10000],
			current_rom_bank: 1,
			current_ram_bank: 0,
			ram_banks: [0;0x8000],
			mbc: mbc(num),
			enable_ram_banking: false,
            rom_banking: true
		}    	
    }

    //TODO FINISH!
    //Method to write memory and update ram/rom banking
    pub fn write_memory(&mut self,data: u8,address: u16){
    	//Handle banking by attempting to write ROM
    	if address<0x8000 {
    		self.handle_banking(data,address);
    	}
    	//RAM Banking
    	else if (address >= 0xA000)&&(address < 0xC000) {
    		if self.enable_ram_banking{
    			let new_address=(address-0xA000)+(self.current_ram_bank as u16*0x2000);
    			self.ram_banks[new_address as usize]=data;
    		}
    	}
    	//ECHO RAM
    	else if (address>=0xE000)&&(address<0xFE00) {
    		self.write_memory(data,address-0x2000);
    		self.main_mem[address as usize]=data;
    	}
    	//Restricted!
    	else if (address>=0xFEA0)&&(address<0xFEFF) { 
    		panic!("Restricted!");
    	}
    	//Otherwise write
    	else{
    		self.main_mem[address as usize]=data;
    	}
    }

    //Method to read memory based on memory map
    pub fn read_memory(&mut self,address: u16)->u8{
    	//Read with rom banking
    	if (address>=0x4000)&&(address<=0x7FFF){
    		let new_address: u16=(address-0x4000u16)+(self.current_rom_bank as u16*0x4000);
    		self.cart_rom[new_address as usize]
    	}
    	//Read with ram banking
    	else if (address>=0xA000)&&(address<=0xBFFF){
    		let new_address: u16=(address-0xA000u16)+(self.current_ram_bank as u16*0x2000);
    		self.ram_banks[new_address as usize]
    	}
    	//Otherwise just read
    	else{
    		self.main_mem[address as usize]
    	}
    }

    fn handle_banking(&mut self,data:u8,address: u16){
    	//do ram enable
    	if address<0x2000 {
    		if self.mbc>0 {
    			self.do_ram_bank_enable(data,address);
    		}
    	}
    	//do lo rom bank change
    	else if (address >= 0x200)&&(address < 0x4000){
    		if self.mbc>0 {
                //lower bits rom bank change 
                self.do_lo_rom_bank_change(data);
    		}
    	}
        //do hi rom bank change or ram bank change
        else if (address>=0x4000)&&(address<0x6000){
            if self.mbc==1 {
                if self.rom_banking {
                    self.do_hi_rom_bank_change(data);
                }
                else {
                    self.current_ram_bank=data&0x3;
                }
            }
        }
    }

    fn do_ram_bank_enable(&mut self,data: u8,address: u16){
    	if self.mbc==2 {
	    	//if the 4th bit is 1 then we return
	    	if data&1<<3!=0 {
	    		return;
	    	}
	    }
	    //Otherwise its mbc1 
	    let lower_nibble: u8=data&0xFu8;
	    //Enable ram bank if 0xA
	    if lower_nibble==0xAu8{
	    	self.enable_ram_banking=true;
	    } 
	    //Disable ram if 0
	    else if lower_nibble==0x0u8{
	    	self.enable_ram_banking=false;
	    }
    }

    ////////////////////////////////////////////
    ///   TODO?? lsb of upper address == 1   ///
    ////////////////////////////////////////////
    fn do_lo_rom_bank_change(&mut self,data: u8){
        if self.mbc==2 {
            self.current_rom_bank=data&0xF;
            if self.current_rom_bank==0 {
                self.current_rom_bank+=1;
            }
            return;
        }
        self.current_rom_bank&=0xE0;      //turn off lower 5bits
        self.current_rom_bank|=data&0x1F; //get data's 5 lower bits
        if self.current_rom_bank==0 {
            self.current_rom_bank+=1;
        }       
    }

    fn do_hi_rom_bank_change(&mut self,data: u8){
        self.current_rom_bank&=0x1F;      //turn of upper 3bits
        self.current_rom_bank|=data&0xE0; //get data's 3 upper bits  
        if self.current_rom_bank==0 {
            self.current_rom_bank+=1;
        }
    }


}