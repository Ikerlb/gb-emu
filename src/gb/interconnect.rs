use crate::gb::mbc::Cartridge;


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
            0x0000..=0x7FFF |
            0xA000..=0xBFFF => self.cartridge.read(address),
            _               => unimplemented!(),
        }
    }

    /// Attempts to read a byte from the given address.
    /// Returns None for unimplemented memory regions instead of panicking.
    pub fn try_read(&self, address: u16) -> Option<u8> {
        match address {
            0x0000..=0x7FFF |
            0xA000..=0xBFFF => Some(self.cartridge.read(address)),
            _ => None,
        }
    }

    //reads 16bits
    pub fn read_16bits(&self,address:u16)->u16{
        (self.read(address) as u16) << 8 | (self.read(address+1) as u16)
    }

    pub fn write(&mut self,address:u16,data:u8){
        //TODO finish
        match address{
            0x0000..=0x7FFF |
            0xA000..=0xBFFF => self.cartridge.write(address,data),
            _               => unimplemented!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_rom() -> Vec<u8> {
        let mut rom = vec![0; 0x8000];
        // Set up a minimal valid cartridge header
        rom[0x0147] = 0x00; // ROM ONLY (no MBC)
        rom[0x0148] = 0x00; // 32KB ROM
        rom[0x0149] = 0x00; // No RAM

        // Add some test data
        rom[0x0100] = 0xAB;
        rom[0x0150] = 0xCD;
        rom[0x7FFF] = 0xEF;
        rom
    }

    #[test]
    fn test_interconnect_creation() {
        let rom = create_test_rom();
        let inter = Interconnect::new(rom);
        // Should not panic
    }

    #[test]
    fn test_read_rom_bank_0() {
        let rom = create_test_rom();
        let inter = Interconnect::new(rom);

        // Read from bank 0
        assert_eq!(inter.read(0x0100), 0xAB);
        assert_eq!(inter.read(0x0150), 0xCD);
    }

    #[test]
    fn test_read_rom_bank_n() {
        let rom = create_test_rom();
        let inter = Interconnect::new(rom);

        // Read from end of ROM
        assert_eq!(inter.read(0x7FFF), 0xEF);
    }

    #[test]
    fn test_read_16bits() {
        let mut rom = create_test_rom();
        rom[0x0200] = 0x34;
        rom[0x0201] = 0x12;

        let inter = Interconnect::new(rom);

        // Read 16-bit value (little-endian)
        let value = inter.read_16bits(0x0200);
        assert_eq!(value, 0x3412);
    }

    #[test]
    fn test_read_16bits_boundary() {
        let mut rom = create_test_rom();
        rom[0x0000] = 0xFF;
        rom[0x0001] = 0x00;

        let inter = Interconnect::new(rom);
        let value = inter.read_16bits(0x0000);
        assert_eq!(value, 0xFF00);
    }

    #[test]
    fn test_try_read_rom() {
        let mut rom = create_test_rom();
        rom[0x0100] = 0xAB;
        rom[0x4000] = 0xCD;

        let inter = Interconnect::new(rom);

        // ROM addresses should return Some
        assert_eq!(inter.try_read(0x0100), Some(0xAB));
        assert_eq!(inter.try_read(0x4000), Some(0xCD));
        assert_eq!(inter.try_read(0x0000), Some(0x00));
    }

    #[test]
    fn test_try_read_unimplemented() {
        let rom = create_test_rom();
        let inter = Interconnect::new(rom);

        // Unimplemented regions should return None
        assert_eq!(inter.try_read(0x8000), None); // VRAM
        assert_eq!(inter.try_read(0xC000), None); // WRAM
        assert_eq!(inter.try_read(0xFE00), None); // OAM
        assert_eq!(inter.try_read(0xFF00), None); // I/O
        assert_eq!(inter.try_read(0xFF80), None); // HRAM
        assert_eq!(inter.try_read(0xFFFF), None); // IE register
    }
}