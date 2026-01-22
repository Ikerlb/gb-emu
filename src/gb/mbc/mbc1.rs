/// MBC1 - The most common mapper (e.g., Pokemon Red/Blue, Zelda)
///
/// Features:
/// - Up to 2MB ROM (128 banks)
/// - Up to 32KB RAM (4 banks)
/// - Two banking modes: ROM mode and RAM mode

use super::traits::{Memory, Stable};

pub struct Mbc1 {
    rom: Vec<u8>,
    ram: Vec<u8>,

    rom_bank: u8,      // 5-bit register (writes to 0x2000-0x3FFF)
    ram_bank: u8,      // 2-bit register (writes to 0x4000-0x5FFF)
    ram_enabled: bool, // Writes to 0x0000-0x1FFF
    banking_mode: u8,  // 0 = ROM mode, 1 = RAM mode

    has_battery: bool,
}

impl Mbc1 {
    pub fn new(rom: Vec<u8>, ram_size: usize, has_battery: bool) -> Self {
        Self {
            rom,
            ram: vec![0; ram_size],
            rom_bank: 1,
            ram_bank: 0,
            ram_enabled: false,
            banking_mode: 0,
            has_battery,
        }
    }

    fn effective_rom_bank(&self) -> usize {
        let bank = if self.banking_mode == 0 {
            // ROM mode: use both registers
            (self.rom_bank & 0x1F) | (self.ram_bank << 5)
        } else {
            // RAM mode: only lower 5 bits
            self.rom_bank & 0x1F
        };

        // Quirk: bank 0 becomes bank 1
        let bank = if (bank & 0x1F) == 0 { bank | 1 } else { bank };

        bank as usize
    }

    fn effective_ram_bank(&self) -> usize {
        if self.banking_mode == 1 {
            self.ram_bank as usize
        } else {
            0
        }
    }
}

impl Memory for Mbc1 {
    fn read(&self, addr: u16) -> u8 {
        match addr {
            // Fixed bank area
            0x0000..=0x3FFF => {
                let physical = if self.banking_mode == 1 {
                    let bank = (self.ram_bank as usize) << 5;
                    (bank * 0x4000) + (addr as usize)
                } else {
                    addr as usize
                };
                self.rom.get(physical).copied().unwrap_or(0xFF)
            }

            // Switchable bank area
            0x4000..=0x7FFF => {
                let bank = self.effective_rom_bank();
                let offset = (addr - 0x4000) as usize;
                let physical = (bank * 0x4000) + offset;
                self.rom.get(physical).copied().unwrap_or(0xFF)
            }

            // External RAM
            0xA000..=0xBFFF => {
                if !self.ram_enabled || self.ram.is_empty() {
                    return 0xFF;
                }
                let bank = self.effective_ram_bank();
                let offset = (addr - 0xA000) as usize;
                let physical = (bank * 0x2000) + offset;
                self.ram.get(physical).copied().unwrap_or(0xFF)
            }

            _ => 0xFF,
        }
    }

    fn write(&mut self, addr: u16, val: u8) {
        match addr {
            // RAM enable
            0x0000..=0x1FFF => {
                self.ram_enabled = (val & 0x0F) == 0x0A;
            }

            // ROM bank (lower 5 bits)
            0x2000..=0x3FFF => {
                self.rom_bank = val & 0x1F;
            }

            // RAM bank / ROM upper bits
            0x4000..=0x5FFF => {
                self.ram_bank = val & 0x03;
            }

            // Banking mode
            0x6000..=0x7FFF => {
                self.banking_mode = val & 0x01;
            }

            // RAM write
            0xA000..=0xBFFF => {
                if !self.ram_enabled || self.ram.is_empty() {
                    return;
                }
                let bank = self.effective_ram_bank();
                let offset = (addr - 0xA000) as usize;
                let physical = (bank * 0x2000) + offset;
                if physical < self.ram.len() {
                    self.ram[physical] = val;
                }
            }

            _ => {}
        }
    }
}

impl Stable for Mbc1 {
    fn save_data(&self) -> Vec<u8> {
        if self.has_battery {
            self.ram.clone()
        } else {
            Vec::new()
        }
    }

    fn load_data(&mut self, data: &[u8]) {
        if self.has_battery {
            let len = data.len().min(self.ram.len());
            self.ram[..len].copy_from_slice(&data[..len]);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_mbc1(rom_size: usize, ram_size: usize) -> Mbc1 {
        let rom = vec![0; rom_size];
        Mbc1::new(rom, ram_size, false)
    }

    #[test]
    fn bank_switching() {
        let mut rom = vec![0; 0x80000]; // 512KB
        rom[0x4000] = 0x11;              // Bank 1
        rom[0x4000 + 0x4000] = 0x22;     // Bank 2
        rom[0x4000 + 0x8000] = 0x33;     // Bank 3

        let mut mbc = Mbc1::new(rom, 0, false);

        assert_eq!(mbc.read(0x4000), 0x11); // Default bank 1

        mbc.write(0x2000, 0x02);
        assert_eq!(mbc.read(0x4000), 0x22);

        mbc.write(0x2000, 0x03);
        assert_eq!(mbc.read(0x4000), 0x33);
    }

    #[test]
    fn bank_0_becomes_bank_1() {
        let mut rom = vec![0; 0x80000];
        rom[0x4000] = 0xAA; // Bank 1

        let mut mbc = Mbc1::new(rom, 0, false);

        mbc.write(0x2000, 0x00); // Try to select bank 0
        assert_eq!(mbc.read(0x4000), 0xAA); // Gets bank 1 instead
    }

    #[test]
    fn ram_disabled_by_default() {
        let mut mbc = make_mbc1(0x8000, 0x2000);

        assert_eq!(mbc.read(0xA000), 0xFF);
    }

    #[test]
    fn ram_enable_disable() {
        let mut mbc = make_mbc1(0x8000, 0x2000);

        // Enable RAM
        mbc.write(0x0000, 0x0A);
        mbc.write(0xA000, 0x42);
        assert_eq!(mbc.read(0xA000), 0x42);

        // Disable RAM
        mbc.write(0x0000, 0x00);
        assert_eq!(mbc.read(0xA000), 0xFF);
    }

    #[test]
    fn save_data_with_battery() {
        let mut mbc = Mbc1::new(vec![0; 0x8000], 0x2000, true);

        mbc.write(0x0000, 0x0A); // Enable RAM
        mbc.write(0xA000, 0x42);

        let save = mbc.save_data();
        assert_eq!(save[0], 0x42);
    }

    #[test]
    fn save_data_without_battery() {
        let mut mbc = Mbc1::new(vec![0; 0x8000], 0x2000, false);

        mbc.write(0x0000, 0x0A);
        mbc.write(0xA000, 0x42);

        let save = mbc.save_data();
        assert!(save.is_empty());
    }
}
