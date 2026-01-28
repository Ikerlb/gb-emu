use crate::gb::mbc::Cartridge;
use crate::gb::ppu::Ppu;
use crate::gb::timer::Timer;
use crate::gb::joypad::{Joypad, Button};

/// Interrupt flag bits
pub const INT_VBLANK: u8 = 0x01;  // Bit 0: V-Blank
pub const INT_STAT: u8 = 0x02;    // Bit 1: LCD STAT
pub const INT_TIMER: u8 = 0x04;   // Bit 2: Timer
pub const INT_SERIAL: u8 = 0x08;  // Bit 3: Serial
pub const INT_JOYPAD: u8 = 0x10;  // Bit 4: Joypad

pub struct Interconnect {
    cartridge: Cartridge,
    ppu: Ppu,
    timer: Timer,
    joypad: Joypad,
    vram: Vec<u8>,         // 0x8000-0x9FFF (8KB)
    wram: Vec<u8>,         // 0xC000-0xDFFF (8KB)
    oam: Vec<u8>,          // 0xFE00-0xFE9F (160 bytes)
    io_registers: Vec<u8>, // 0xFF00-0xFF7F (128 bytes, but some routed to PPU/Timer/Joypad)
    hram: Vec<u8>,         // 0xFF80-0xFFFE (127 bytes)
    ie_register: u8,       // 0xFFFF - Interrupt Enable
    if_register: u8,       // 0xFF0F - Interrupt Flags
}

impl Interconnect {
    pub fn new(cart: Vec<u8>) -> Self {
        Interconnect {
            cartridge: Cartridge::new(cart),
            ppu: Ppu::new(),
            timer: Timer::new(),
            joypad: Joypad::new(),
            vram: vec![0; 0x2000],         // 8KB initialized to 0
            wram: vec![0; 0x2000],         // 8KB initialized to 0
            oam: vec![0; 160],             // 160 bytes initialized to 0
            io_registers: vec![0xFF; 128], // 128 bytes stubbed to 0xFF
            hram: vec![0; 127],            // 127 bytes initialized to 0
            ie_register: 0,                // Interrupt Enable
            if_register: 0xE0,             // Interrupt Flags (upper bits always 1)
        }
    }

    /// Advance the PPU and Timer by the given number of T-cycles
    pub fn step(&mut self, cycles: u32) {
        // Step PPU (returns true if VBlank was just entered)
        let entered_vblank = self.ppu.step(cycles, &self.vram, &self.oam);

        // Step Timer
        self.timer.step(cycles);

        // Check for timer interrupt
        if self.timer.interrupt_requested {
            self.if_register |= INT_TIMER;
            self.timer.clear_interrupt();
        }

        // Request VBlank interrupt only when entering VBlank (not continuously)
        if entered_vblank {
            self.if_register |= INT_VBLANK;
        }
    }

    /// Legacy method - calls step()
    pub fn step_ppu(&mut self, cycles: u32) {
        self.step(cycles);
    }

    /// Get pending interrupts (IF & IE)
    pub fn pending_interrupts(&self) -> u8 {
        self.if_register & self.ie_register & 0x1F
    }

    /// Clear a specific interrupt flag
    pub fn clear_interrupt(&mut self, interrupt: u8) {
        self.if_register &= !interrupt;
    }

    /// Request an interrupt
    pub fn request_interrupt(&mut self, interrupt: u8) {
        self.if_register |= interrupt;
    }

    /// Get IE register
    pub fn ie(&self) -> u8 {
        self.ie_register
    }

    /// Get IF register
    pub fn if_reg(&self) -> u8 {
        self.if_register | 0xE0 // Upper 3 bits always read as 1
    }

    /// Check if a new frame is ready for display
    pub fn frame_ready(&self) -> bool {
        self.ppu.frame_ready
    }

    /// Get a reference to the framebuffer
    pub fn framebuffer(&self) -> &[u32] {
        &self.ppu.framebuffer
    }

    /// Clear the frame_ready flag after displaying
    pub fn clear_frame_ready(&mut self) {
        self.ppu.frame_ready = false;
    }

    /// Press a joypad button
    pub fn press_button(&mut self, button: Button) {
        self.joypad.press(button);
    }

    /// Release a joypad button
    pub fn release_button(&mut self, button: Button) {
        self.joypad.release(button);
    }

    /// Perform OAM DMA transfer
    /// Copies 160 bytes from source (data << 8) to OAM (0xFE00-0xFE9F)
    fn oam_dma(&mut self, data: u8) {
        let source = (data as u16) << 8;
        for i in 0..160u16 {
            let byte = self.read(source + i);
            self.oam[i as usize] = byte;
        }
    }

    /// Reads 8 bits from the given address
    pub fn read(&self, address: u16) -> u8 {
        match address {
            // Cartridge ROM (banks 0 and N)
            0x0000..=0x7FFF => self.cartridge.read(address),
            // VRAM
            0x8000..=0x9FFF => self.vram[(address - 0x8000) as usize],
            // Cartridge External RAM
            0xA000..=0xBFFF => self.cartridge.read(address),
            // WRAM
            0xC000..=0xDFFF => self.wram[(address - 0xC000) as usize],
            // Echo RAM (mirror of WRAM)
            0xE000..=0xFDFF => self.wram[(address - 0xE000) as usize],
            // OAM
            0xFE00..=0xFE9F => self.oam[(address - 0xFE00) as usize],
            // Unusable region
            0xFEA0..=0xFEFF => 0xFF,
            // Joypad register
            0xFF00 => self.joypad.read(),
            // Timer registers
            0xFF04..=0xFF07 => self.timer.read(address),
            // IF - Interrupt Flags
            0xFF0F => self.if_register | 0xE0,
            // OAM DMA register (returns 0xFF, write-only effectively)
            0xFF46 => 0xFF,
            // PPU registers (excluding DMA at 0xFF46)
            0xFF40..=0xFF45 | 0xFF47..=0xFF4B => self.ppu.read(address),
            // Other I/O registers
            0xFF01..=0xFF03 | 0xFF08..=0xFF0E | 0xFF10..=0xFF3F | 0xFF4C..=0xFF7F => {
                self.io_registers[(address - 0xFF00) as usize]
            }
            // HRAM
            0xFF80..=0xFFFE => self.hram[(address - 0xFF80) as usize],
            // IE Register
            0xFFFF => self.ie_register,
        }
    }

    /// Reads 16 bits from the given address (little-endian: low byte first)
    pub fn read_16bits(&self, address: u16) -> u16 {
        let lo = self.read(address) as u16;
        let hi = self.read(address.wrapping_add(1)) as u16;
        (hi << 8) | lo
    }

    /// Writes 16 bits to the given address (little-endian: low byte first)
    pub fn write_16bits(&mut self, address: u16, value: u16) {
        self.write(address, value as u8);
        self.write(address.wrapping_add(1), (value >> 8) as u8);
    }

    /// Writes 8 bits to the given address
    pub fn write(&mut self, address: u16, data: u8) {
        match address {
            // Cartridge ROM (banks 0 and N) - writes may control MBC
            0x0000..=0x7FFF => self.cartridge.write(address, data),
            // VRAM
            0x8000..=0x9FFF => self.vram[(address - 0x8000) as usize] = data,
            // Cartridge External RAM
            0xA000..=0xBFFF => self.cartridge.write(address, data),
            // WRAM
            0xC000..=0xDFFF => self.wram[(address - 0xC000) as usize] = data,
            // Echo RAM (mirror of WRAM) - writes go to underlying WRAM
            0xE000..=0xFDFF => self.wram[(address - 0xE000) as usize] = data,
            // OAM
            0xFE00..=0xFE9F => self.oam[(address - 0xFE00) as usize] = data,
            // Unusable region - writes ignored
            0xFEA0..=0xFEFF => { /* ignored */ }
            // Joypad register
            0xFF00 => self.joypad.write(data),
            // Timer registers
            0xFF04..=0xFF07 => self.timer.write(address, data),
            // IF - Interrupt Flags
            0xFF0F => self.if_register = data | 0xE0,
            // OAM DMA transfer
            0xFF46 => self.oam_dma(data),
            // PPU registers (excluding DMA at 0xFF46)
            0xFF40..=0xFF45 | 0xFF47..=0xFF4B => self.ppu.write(address, data),
            // Other I/O registers
            0xFF01..=0xFF03 | 0xFF08..=0xFF0E | 0xFF10..=0xFF3F | 0xFF4C..=0xFF7F => {
                self.io_registers[(address - 0xFF00) as usize] = data
            }
            // HRAM
            0xFF80..=0xFFFE => self.hram[(address - 0xFF80) as usize] = data,
            // IE Register
            0xFFFF => self.ie_register = data,
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
        rom[0x0200] = 0x34; // Low byte
        rom[0x0201] = 0x12; // High byte

        let inter = Interconnect::new(rom);

        // Read 16-bit value (little-endian: low byte at lower address)
        let value = inter.read_16bits(0x0200);
        assert_eq!(value, 0x1234);
    }

    #[test]
    fn test_read_16bits_boundary() {
        let mut rom = create_test_rom();
        rom[0x0000] = 0xFF; // Low byte
        rom[0x0001] = 0x00; // High byte

        let inter = Interconnect::new(rom);
        let value = inter.read_16bits(0x0000);
        assert_eq!(value, 0x00FF);
    }

    #[test]
    fn test_vram_read_write() {
        let rom = create_test_rom();
        let mut inter = Interconnect::new(rom);

        // VRAM initially 0
        assert_eq!(inter.read(0x8000), 0);
        assert_eq!(inter.read(0x9FFF), 0);

        // Write and read back
        inter.write(0x8000, 0xAB);
        inter.write(0x9FFF, 0xCD);
        assert_eq!(inter.read(0x8000), 0xAB);
        assert_eq!(inter.read(0x9FFF), 0xCD);
    }

    #[test]
    fn test_wram_read_write() {
        let rom = create_test_rom();
        let mut inter = Interconnect::new(rom);

        // WRAM initially 0
        assert_eq!(inter.read(0xC000), 0);
        assert_eq!(inter.read(0xDFFF), 0);

        // Write and read back
        inter.write(0xC000, 0x12);
        inter.write(0xDFFF, 0x34);
        assert_eq!(inter.read(0xC000), 0x12);
        assert_eq!(inter.read(0xDFFF), 0x34);
    }

    #[test]
    fn test_echo_ram_mirrors_wram() {
        let rom = create_test_rom();
        let mut inter = Interconnect::new(rom);

        // Write to WRAM, read from Echo RAM
        inter.write(0xC000, 0xAA);
        assert_eq!(inter.read(0xE000), 0xAA);

        // Write to Echo RAM, read from WRAM
        inter.write(0xE100, 0xBB);
        assert_eq!(inter.read(0xC100), 0xBB);

        // Test boundary
        inter.write(0xDDFF, 0xCC);
        assert_eq!(inter.read(0xFDFF), 0xCC);
    }

    #[test]
    fn test_oam_read_write() {
        let rom = create_test_rom();
        let mut inter = Interconnect::new(rom);

        // OAM initially 0
        assert_eq!(inter.read(0xFE00), 0);
        assert_eq!(inter.read(0xFE9F), 0);

        // Write and read back
        inter.write(0xFE00, 0x55);
        inter.write(0xFE9F, 0x66);
        assert_eq!(inter.read(0xFE00), 0x55);
        assert_eq!(inter.read(0xFE9F), 0x66);
    }

    #[test]
    fn test_unusable_region() {
        let rom = create_test_rom();
        let mut inter = Interconnect::new(rom);

        // Unusable region always returns 0xFF
        assert_eq!(inter.read(0xFEA0), 0xFF);
        assert_eq!(inter.read(0xFEFF), 0xFF);

        // Writes are ignored
        inter.write(0xFEA0, 0x00);
        inter.write(0xFEFF, 0x00);
        assert_eq!(inter.read(0xFEA0), 0xFF);
        assert_eq!(inter.read(0xFEFF), 0xFF);
    }

    #[test]
    fn test_io_registers_stub() {
        let rom = create_test_rom();
        let mut inter = Interconnect::new(rom);

        // Stubbed I/O registers initially 0xFF
        // Note: 0xFF00 is joypad (handled specially), use 0xFF01 for stub test
        assert_eq!(inter.read(0xFF01), 0xFF);
        assert_eq!(inter.read(0xFF7F), 0xFF);

        // Can write and read back
        inter.write(0xFF01, 0x00);
        inter.write(0xFF7F, 0x12);
        assert_eq!(inter.read(0xFF01), 0x00);
        assert_eq!(inter.read(0xFF7F), 0x12);
    }

    #[test]
    fn test_joypad_register() {
        let rom = create_test_rom();
        let mut inter = Interconnect::new(rom);

        // Joypad initially returns 0xFF (neither selection active, all buttons read high)
        // Bits 6-7 = 1, bits 4-5 = 1 (neither selected), bits 0-3 = 1
        assert_eq!(inter.read(0xFF00), 0xFF);

        // Select direction buttons (bit 4 = 0)
        inter.write(0xFF00, 0x20);
        // Should read 0xEF (bit 4=0 for directions selected, all buttons released)
        assert_eq!(inter.read(0xFF00) & 0x3F, 0x2F);
    }

    #[test]
    fn test_hram_read_write() {
        let rom = create_test_rom();
        let mut inter = Interconnect::new(rom);

        // HRAM initially 0
        assert_eq!(inter.read(0xFF80), 0);
        assert_eq!(inter.read(0xFFFE), 0);

        // Write and read back
        inter.write(0xFF80, 0x77);
        inter.write(0xFFFE, 0x88);
        assert_eq!(inter.read(0xFF80), 0x77);
        assert_eq!(inter.read(0xFFFE), 0x88);
    }

    #[test]
    fn test_ie_register_read_write() {
        let rom = create_test_rom();
        let mut inter = Interconnect::new(rom);

        // IE register initially 0
        assert_eq!(inter.read(0xFFFF), 0);

        // Write and read back
        inter.write(0xFFFF, 0x1F);
        assert_eq!(inter.read(0xFFFF), 0x1F);
    }

    #[test]
    fn test_memory_region_boundaries() {
        let rom = create_test_rom();
        let mut inter = Interconnect::new(rom);

        // Test boundaries between regions
        // VRAM end / External RAM start
        inter.write(0x9FFF, 0xAA);
        assert_eq!(inter.read(0x9FFF), 0xAA);

        // WRAM end
        inter.write(0xDFFF, 0xBB);
        assert_eq!(inter.read(0xDFFF), 0xBB);

        // Echo RAM end mirrors WRAM at 0xDDFF (0xFDFF - 0xE000 = 0x1DFF offset)
        inter.write(0xDDFF, 0xCC);
        assert_eq!(inter.read(0xFDFF), 0xCC);

        // Echo RAM end / OAM start
        assert_eq!(inter.read(0xFDFF), 0xCC);
        inter.write(0xFE00, 0xCC);
        assert_eq!(inter.read(0xFE00), 0xCC);

        // OAM end / Unusable start
        inter.write(0xFE9F, 0xDD);
        assert_eq!(inter.read(0xFE9F), 0xDD);
        assert_eq!(inter.read(0xFEA0), 0xFF);

        // Unusable end / I/O start
        assert_eq!(inter.read(0xFEFF), 0xFF);
        // I/O starts at 0xFF00

        // I/O end / HRAM start
        inter.write(0xFF7F, 0xEE);
        assert_eq!(inter.read(0xFF7F), 0xEE);
        inter.write(0xFF80, 0x11);
        assert_eq!(inter.read(0xFF80), 0x11);

        // HRAM end / IE register
        inter.write(0xFFFE, 0x22);
        assert_eq!(inter.read(0xFFFE), 0x22);
        inter.write(0xFFFF, 0x33);
        assert_eq!(inter.read(0xFFFF), 0x33);
    }
}