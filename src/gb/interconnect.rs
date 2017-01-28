use gb::cartridge::Cartridge;


pub struct Interconnect{
    cartridge: Cartridge,
}

impl Interconnect{
    pub fn new(cart:Vec<u8>)->Self{
        Interconnect{
            cartridge: Cartridge::new(cart),
        }
    }

    //reads 8bits
    pub fn read(&self,address:u16)->u8{
        //TODO finish
        match address{
            0x0000...0x7FFF |
            0xA000...0xBFFF => self.cartridge.read(address),
            _               => unimplemented!(),
        }
    }

    //reads 16bits
    pub fn read_16bits(&self,address:u16)->u16{
        (self.read(address) as u16) << 8 | (self.read(address+1) as u16)
    }

    pub fn write(&mut self,address:u16,data:u8){
        //TODO finish
        match address{
            0x0000...0x7FFF |
            0xA000...0xBFFF => self.cartridge.write(address,data),
            _               => unimplemented!(),
        }
    }
}