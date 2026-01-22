/// MBC0 - No mapper (e.g., Tetris)
///
/// The simplest cartridge type: just ROM, no banking.

use super::traits::{Memory, Stable};

pub struct Mbc0 {
    rom: Vec<u8>,
}

impl Mbc0 {
    pub fn new(rom: Vec<u8>) -> Self {
        Self { rom }
    }
}

impl Memory for Mbc0 {
    fn read(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x7FFF => self.rom.get(addr as usize).copied().unwrap_or(0xFF),
            _ => 0xFF,
        }
    }

    fn write(&mut self, _addr: u16, _val: u8) {
        // MBC0 ignores all writes
    }
}

impl Stable for Mbc0 {
    fn save_data(&self) -> Vec<u8> {
        Vec::new()
    }

    fn load_data(&mut self, _data: &[u8]) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_rom() {
        let mut rom = vec![0; 0x8000];
        rom[0x100] = 0xAB;
        rom[0x4000] = 0xCD;

        let mbc = Mbc0::new(rom);

        assert_eq!(mbc.read(0x100), 0xAB);
        assert_eq!(mbc.read(0x4000), 0xCD);
    }

    #[test]
    fn ignores_writes() {
        let rom = vec![0; 0x8000];
        let mut mbc = Mbc0::new(rom);

        mbc.write(0x2000, 0xFF);
        // No panic = success
    }

    #[test]
    fn out_of_bounds_returns_ff() {
        let rom = vec![0; 0x100]; // Small ROM
        let mbc = Mbc0::new(rom);

        assert_eq!(mbc.read(0x7FFF), 0xFF);
    }
}
