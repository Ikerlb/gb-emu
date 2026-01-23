/// PPU (Picture Processing Unit) - Handles display timing and rendering
///
/// The Game Boy PPU renders 144 visible scanlines (0-143), then enters
/// VBLANK for scanlines 144-153. Each scanline takes 456 T-cycles.
///
/// Key registers:
/// - LY (0xFF44): Current scanline (0-153, read-only)
/// - STAT (0xFF41): LCD status and mode
/// - LCDC (0xFF40): LCD control

/// Cycles per scanline
const CYCLES_PER_SCANLINE: u32 = 456;

/// Total scanlines per frame (0-153)
const SCANLINES_PER_FRAME: u8 = 154;

/// First VBLANK scanline
const VBLANK_START: u8 = 144;

/// Cycle at which OAM scan ends and drawing begins
const OAM_SCAN_END: u32 = 80;

/// Approximate cycle at which drawing ends and HBlank begins
const DRAWING_END: u32 = 252;

/// PPU modes (bits 0-1 of STAT register)
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Mode {
    HBlank = 0,    // Mode 0: Horizontal blank (CPU can access VRAM/OAM)
    VBlank = 1,    // Mode 1: Vertical blank (scanlines 144-153)
    OamScan = 2,   // Mode 2: Searching OAM (CPU cannot access OAM)
    Drawing = 3,   // Mode 3: Transferring data to LCD (CPU cannot access VRAM/OAM)
}

pub struct Ppu {
    /// Current scanline (0-153)
    ly: u8,
    /// Cycle counter within current scanline (0-455)
    scanline_cycles: u32,
    /// Current PPU mode
    mode: Mode,
    /// LYC register (0xFF45) - LY compare value
    lyc: u8,
    /// LCDC register (0xFF40) - LCD control
    lcdc: u8,
    /// STAT register (0xFF41) - LCD status (bits 2-6 are R/W, bits 0-1 are mode)
    stat: u8,
    /// SCY register (0xFF42) - Scroll Y
    scy: u8,
    /// SCX register (0xFF43) - Scroll X
    scx: u8,
    /// BGP register (0xFF47) - BG palette
    bgp: u8,
    /// OBP0 register (0xFF48) - Object palette 0
    obp0: u8,
    /// OBP1 register (0xFF49) - Object palette 1
    obp1: u8,
    /// WY register (0xFF4A) - Window Y
    wy: u8,
    /// WX register (0xFF4B) - Window X
    wx: u8,
}

impl Ppu {
    pub fn new() -> Self {
        Self {
            ly: 0,
            scanline_cycles: 0,
            mode: Mode::OamScan,
            lyc: 0,
            lcdc: 0x91, // LCD on, BG on by default (common boot state)
            stat: 0,
            scy: 0,
            scx: 0,
            bgp: 0xFC,  // Default palette
            obp0: 0xFF,
            obp1: 0xFF,
            wy: 0,
            wx: 0,
        }
    }

    /// Advance PPU state by the given number of T-cycles
    pub fn step(&mut self, cycles: u32) {
        // If LCD is off, do nothing
        if !self.lcd_enabled() {
            return;
        }

        self.scanline_cycles += cycles;

        // Check if we've completed a scanline
        while self.scanline_cycles >= CYCLES_PER_SCANLINE {
            self.scanline_cycles -= CYCLES_PER_SCANLINE;
            self.ly = (self.ly + 1) % SCANLINES_PER_FRAME;
        }

        // Update mode based on current state
        self.update_mode();
    }

    /// Update the PPU mode based on current scanline and cycle
    fn update_mode(&mut self) {
        self.mode = if self.ly >= VBLANK_START {
            Mode::VBlank
        } else if self.scanline_cycles < OAM_SCAN_END {
            Mode::OamScan
        } else if self.scanline_cycles < DRAWING_END {
            // Mode 3 duration varies, using approximate value
            Mode::Drawing
        } else {
            Mode::HBlank
        };
    }

    /// Check if LCD is enabled (bit 7 of LCDC)
    pub fn lcd_enabled(&self) -> bool {
        self.lcdc & 0x80 != 0
    }

    /// Read a PPU register
    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            0xFF40 => self.lcdc,
            0xFF41 => {
                // STAT: bits 0-1 are mode, bit 2 is LYC=LY flag
                let lyc_flag = if self.ly == self.lyc { 0x04 } else { 0 };
                (self.stat & 0xF8) | lyc_flag | (self.mode as u8)
            }
            0xFF42 => self.scy,
            0xFF43 => self.scx,
            0xFF44 => self.ly,
            0xFF45 => self.lyc,
            0xFF47 => self.bgp,
            0xFF48 => self.obp0,
            0xFF49 => self.obp1,
            0xFF4A => self.wy,
            0xFF4B => self.wx,
            _ => 0xFF, // Unmapped
        }
    }

    /// Write to a PPU register
    pub fn write(&mut self, addr: u16, val: u8) {
        match addr {
            0xFF40 => self.write_lcdc(val),
            0xFF41 => self.stat = (self.stat & 0x07) | (val & 0x78), // Only bits 3-6 writable
            0xFF42 => self.scy = val,
            0xFF43 => self.scx = val,
            0xFF44 => {} // LY is read-only
            0xFF45 => self.lyc = val,
            0xFF47 => self.bgp = val,
            0xFF48 => self.obp0 = val,
            0xFF49 => self.obp1 = val,
            0xFF4A => self.wy = val,
            0xFF4B => self.wx = val,
            _ => {} // Ignore unmapped
        }
    }

    /// Handle writes to LCDC register with LCD enable/disable logic
    fn write_lcdc(&mut self, val: u8) {
        let was_enabled = self.lcd_enabled();
        self.lcdc = val;

        // If LCD was just disabled, reset state
        if was_enabled && !self.lcd_enabled() {
            self.ly = 0;
            self.scanline_cycles = 0;
            self.mode = Mode::HBlank;
        }
    }

    /// Get current scanline (LY)
    pub fn ly(&self) -> u8 {
        self.ly
    }

    /// Get current mode
    pub fn mode(&self) -> Mode {
        self.mode
    }

    /// Check if in VBLANK period
    pub fn in_vblank(&self) -> bool {
        self.mode == Mode::VBlank
    }
}

impl Default for Ppu {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ppu_new() {
        let ppu = Ppu::new();
        assert_eq!(ppu.ly(), 0);
        assert_eq!(ppu.mode(), Mode::OamScan);
    }

    #[test]
    fn test_ly_increments_after_456_cycles() {
        let mut ppu = Ppu::new();
        assert_eq!(ppu.ly(), 0);

        // Step 455 cycles - should still be on scanline 0
        ppu.step(455);
        assert_eq!(ppu.ly(), 0);

        // Step 1 more cycle - should now be on scanline 1
        ppu.step(1);
        assert_eq!(ppu.ly(), 1);
    }

    #[test]
    fn test_ly_wraps_at_154() {
        let mut ppu = Ppu::new();

        // Advance through all 154 scanlines
        for _ in 0..154 {
            ppu.step(CYCLES_PER_SCANLINE);
        }

        // Should wrap back to 0
        assert_eq!(ppu.ly(), 0);
    }

    #[test]
    fn test_vblank_starts_at_144() {
        let mut ppu = Ppu::new();

        // Advance to scanline 143
        for _ in 0..143 {
            ppu.step(CYCLES_PER_SCANLINE);
        }
        assert_eq!(ppu.ly(), 143);
        assert!(!ppu.in_vblank());

        // Advance to scanline 144
        ppu.step(CYCLES_PER_SCANLINE);
        assert_eq!(ppu.ly(), 144);
        assert!(ppu.in_vblank());
        assert_eq!(ppu.mode(), Mode::VBlank);
    }

    #[test]
    fn test_read_ly_register() {
        let mut ppu = Ppu::new();
        assert_eq!(ppu.read(0xFF44), 0);

        ppu.step(CYCLES_PER_SCANLINE * 10);
        assert_eq!(ppu.read(0xFF44), 10);
    }

    #[test]
    fn test_ly_is_read_only() {
        let mut ppu = Ppu::new();
        ppu.step(CYCLES_PER_SCANLINE * 5);
        assert_eq!(ppu.ly(), 5);

        // Try to write to LY - should be ignored
        ppu.write(0xFF44, 100);
        assert_eq!(ppu.ly(), 5);
    }

    #[test]
    fn test_stat_mode_bits() {
        let mut ppu = Ppu::new();

        // In OAM scan initially (mode 2)
        let stat = ppu.read(0xFF41);
        assert_eq!(stat & 0x03, Mode::OamScan as u8);

        // Advance to drawing mode
        ppu.step(80);
        let stat = ppu.read(0xFF41);
        assert_eq!(stat & 0x03, Mode::Drawing as u8);

        // Advance to HBlank
        ppu.step(172);
        let stat = ppu.read(0xFF41);
        assert_eq!(stat & 0x03, Mode::HBlank as u8);
    }

    #[test]
    fn test_lyc_compare_flag() {
        let mut ppu = Ppu::new();
        ppu.write(0xFF45, 5); // Set LYC to 5

        // LY=0, LYC=5, flag should be 0
        assert_eq!(ppu.read(0xFF41) & 0x04, 0);

        // Advance to LY=5
        for _ in 0..5 {
            ppu.step(CYCLES_PER_SCANLINE);
        }
        assert_eq!(ppu.ly(), 5);

        // LY=LYC, flag should be set
        assert_eq!(ppu.read(0xFF41) & 0x04, 0x04);
    }

    #[test]
    fn test_lcd_disable_resets_ly() {
        let mut ppu = Ppu::new();

        // Advance to some scanline
        ppu.step(CYCLES_PER_SCANLINE * 50);
        assert_eq!(ppu.ly(), 50);

        // Disable LCD (clear bit 7 of LCDC)
        ppu.write(0xFF40, ppu.read(0xFF40) & 0x7F);
        assert_eq!(ppu.ly(), 0);
    }

    #[test]
    fn test_lcd_disabled_no_advance() {
        let mut ppu = Ppu::new();
        ppu.write(0xFF40, 0x00); // Disable LCD

        // Step should not advance LY when LCD is off
        ppu.step(CYCLES_PER_SCANLINE * 10);
        assert_eq!(ppu.ly(), 0);
    }

    #[test]
    fn test_multiple_scanlines_in_one_step() {
        let mut ppu = Ppu::new();

        // Step 3 full scanlines worth of cycles
        ppu.step(CYCLES_PER_SCANLINE * 3);
        assert_eq!(ppu.ly(), 3);
    }
}
