/// MBC (Memory Bank Controller) implementations
///
/// Structure:
/// - traits.rs  - Core traits (Memory, Stable, Mbc)
/// - mbc0.rs    - No mapper (simple ROMs like Tetris)
/// - mbc1.rs    - Most common mapper (Pokemon, Zelda, etc.)

mod traits;
mod mbc0;
mod mbc1;

pub use traits::{Memory, Stable, Mbc};
use mbc0::Mbc0;
use mbc1::Mbc1;

/// Unified cartridge interface
pub struct Cartridge {
    mbc: Box<dyn Mbc>,
    #[allow(dead_code)] // Used in tests
    title: String,
    #[allow(dead_code)] // Used in tests
    has_battery: bool,
}

impl Cartridge {
    pub fn new(rom: Vec<u8>) -> Self {
        let title = extract_title(&rom);
        let cart_type = rom[0x147];
        let ram_size = parse_ram_size(rom[0x149]);
        let has_battery = is_battery_backed(cart_type);

        let mbc: Box<dyn Mbc> = match cart_type {
            0x00 => Box::new(Mbc0::new(rom)),
            0x01..=0x03 => Box::new(Mbc1::new(rom, ram_size, has_battery)),
            _ => panic!("Unsupported cartridge type: 0x{:02X}", cart_type),
        };

        Self { mbc, title, has_battery }
    }

    pub fn read(&self, addr: u16) -> u8 {
        self.mbc.read(addr)
    }

    pub fn write(&mut self, addr: u16, val: u8) {
        self.mbc.write(addr, val);
    }

    #[allow(dead_code)] // Used in tests
    pub fn title(&self) -> &str {
        &self.title
    }

    #[allow(dead_code)] // Used in tests
    pub fn has_battery(&self) -> bool {
        self.has_battery
    }

    #[allow(dead_code)] // Infrastructure for future save file support
    pub fn save_data(&self) -> Vec<u8> {
        self.mbc.save_data()
    }

    #[allow(dead_code)] // Infrastructure for future save file support
    pub fn load_data(&mut self, data: &[u8]) {
        self.mbc.load_data(data);
    }
}

fn extract_title(rom: &[u8]) -> String {
    rom[0x134..0x144]
        .iter()
        .take_while(|&&b| b != 0)
        .map(|&b| b as char)
        .collect()
}

fn parse_ram_size(code: u8) -> usize {
    match code {
        0x00 => 0,
        0x01 => 2 * 1024,
        0x02 => 8 * 1024,
        0x03 => 32 * 1024,
        0x04 => 128 * 1024,
        0x05 => 64 * 1024,
        _ => 0,
    }
}

fn is_battery_backed(cart_type: u8) -> bool {
    matches!(cart_type, 0x03 | 0x06 | 0x09 | 0x0D | 0x0F | 0x10 | 0x13 | 0x1B | 0x1E)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_rom(cart_type: u8, ram_size: u8) -> Vec<u8> {
        let mut rom = vec![0; 0x8000];
        rom[0x147] = cart_type;
        rom[0x148] = 0x00;
        rom[0x149] = ram_size;
        rom
    }

    #[test]
    fn creates_mbc0() {
        let rom = make_rom(0x00, 0x00);
        let cart = Cartridge::new(rom);
        assert!(!cart.has_battery());
    }

    #[test]
    fn creates_mbc1() {
        let rom = make_rom(0x01, 0x00);
        let _cart = Cartridge::new(rom);
    }

    #[test]
    fn creates_mbc1_with_battery() {
        let rom = make_rom(0x03, 0x02);
        let cart = Cartridge::new(rom);
        assert!(cart.has_battery());
    }

    #[test]
    fn extracts_title() {
        let mut rom = make_rom(0x00, 0x00);
        rom[0x134..0x13A].copy_from_slice(b"TETRIS");

        let cart = Cartridge::new(rom);
        assert_eq!(cart.title(), "TETRIS");
    }
}
