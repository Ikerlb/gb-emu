/// PPU (Picture Processing Unit) - Handles display timing and rendering
///
/// The Game Boy PPU renders 144 visible scanlines (0-143), then enters
/// VBLANK for scanlines 144-153. Each scanline takes 456 T-cycles.
///
/// Key registers:
/// - LY (0xFF44): Current scanline (0-153, read-only)
/// - STAT (0xFF41): LCD status and mode
/// - LCDC (0xFF40): LCD control

/// Screen dimensions
pub const SCREEN_WIDTH: usize = 160;
pub const SCREEN_HEIGHT: usize = 144;

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

/// Game Boy color palette (DMG green shades)
const COLORS: [u32; 4] = [
    0xFFE0F8D0, // Lightest (color 0) - light green
    0xFF88C070, // Light (color 1)
    0xFF346856, // Dark (color 2)
    0xFF081820, // Darkest (color 3) - near black
];

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
    /// Framebuffer (160x144 pixels as ARGB u32)
    pub framebuffer: Vec<u32>,
    /// Flag indicating a new frame is ready for display
    pub frame_ready: bool,
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
            framebuffer: vec![COLORS[0]; SCREEN_WIDTH * SCREEN_HEIGHT],
            frame_ready: false,
        }
    }

    /// Advance PPU state by the given number of T-cycles
    /// `vram` and `oam` are passed in for rendering
    /// Returns true if VBlank was just entered (for interrupt)
    pub fn step(&mut self, cycles: u32, vram: &[u8], oam: &[u8]) -> bool {
        // If LCD is off, do nothing but preserve frame_ready
        // (games often briefly disable LCD during VBlank to write VRAM)
        if !self.lcd_enabled() {
            return false;
        }

        let mut entered_vblank = false;
        self.scanline_cycles += cycles;

        // Check if we've completed a scanline
        while self.scanline_cycles >= CYCLES_PER_SCANLINE {
            self.scanline_cycles -= CYCLES_PER_SCANLINE;

            // Render the scanline before moving to the next one
            if self.ly < VBLANK_START {
                self.render_scanline(vram, oam);
            }

            let prev_ly = self.ly;
            self.ly = (self.ly + 1) % SCANLINES_PER_FRAME;

            // Check if we just entered VBlank (new frame is ready)
            if self.ly == VBLANK_START && prev_ly != VBLANK_START {
                self.frame_ready = true;
                entered_vblank = true;
            }
        }

        // Update mode based on current state
        self.update_mode();

        entered_vblank
    }

    /// Render a single scanline to the framebuffer
    fn render_scanline(&mut self, vram: &[u8], oam: &[u8]) {
        let ly = self.ly as usize;
        if ly >= SCREEN_HEIGHT {
            return;
        }

        // Render background if enabled
        if self.bg_enabled() {
            self.render_bg_scanline(vram, ly);
        } else {
            // Fill with color 0 if BG disabled
            for x in 0..SCREEN_WIDTH {
                self.framebuffer[ly * SCREEN_WIDTH + x] = COLORS[0];
            }
        }

        // Render window if enabled and visible
        if self.window_enabled() && ly >= self.wy as usize {
            self.render_window_scanline(vram, ly);
        }

        // Render sprites if enabled
        if self.sprites_enabled() {
            self.render_sprites_scanline(vram, oam, ly);
        }
    }

    /// Render background for one scanline
    fn render_bg_scanline(&mut self, vram: &[u8], ly: usize) {
        let scroll_y = self.scy as usize;
        let scroll_x = self.scx as usize;

        // Which tile map to use (bit 3 of LCDC)
        let tile_map_base: usize = if self.lcdc & 0x08 != 0 { 0x1C00 } else { 0x1800 };

        // Which tile data area to use (bit 4 of LCDC)
        let tile_data_base: usize = if self.lcdc & 0x10 != 0 { 0x0000 } else { 0x0800 };
        let signed_addressing = self.lcdc & 0x10 == 0;

        // Y position in the 256x256 background map
        let bg_y = (ly + scroll_y) % 256;
        let tile_row = bg_y / 8;
        let pixel_y_in_tile = bg_y % 8;

        for screen_x in 0..SCREEN_WIDTH {
            // X position in the 256x256 background map
            let bg_x = (screen_x + scroll_x) % 256;
            let tile_col = bg_x / 8;
            let pixel_x_in_tile = bg_x % 8;

            // Get tile index from tile map
            let tile_map_addr = tile_map_base + tile_row * 32 + tile_col;
            let tile_index = vram[tile_map_addr];

            // Get tile data address
            let tile_data_addr = if signed_addressing {
                // Signed addressing: tile 0 is at 0x9000, can go negative to 0x8800
                let signed_index = tile_index as i8 as i16;
                (tile_data_base as i16 + 0x800 + signed_index * 16) as usize
            } else {
                // Unsigned addressing: tile 0 is at 0x8000
                tile_data_base + (tile_index as usize) * 16
            };

            // Get the two bytes for this row of the tile
            let byte1 = vram[tile_data_addr + pixel_y_in_tile * 2];
            let byte2 = vram[tile_data_addr + pixel_y_in_tile * 2 + 1];

            // Get the color index (2 bits per pixel, MSB from byte2, LSB from byte1)
            let bit = 7 - pixel_x_in_tile;
            let color_low = (byte1 >> bit) & 1;
            let color_high = (byte2 >> bit) & 1;
            let color_index = (color_high << 1) | color_low;

            // Apply BGP palette
            let palette_color = (self.bgp >> (color_index * 2)) & 0x03;

            self.framebuffer[ly * SCREEN_WIDTH + screen_x] = COLORS[palette_color as usize];
        }
    }

    /// Render window layer for one scanline
    fn render_window_scanline(&mut self, vram: &[u8], ly: usize) {
        let window_y = ly as i32 - self.wy as i32;
        if window_y < 0 {
            return;
        }
        let window_y = window_y as usize;

        // Window X position (WX - 7)
        let window_x_start = (self.wx as i32 - 7).max(0) as usize;
        if window_x_start >= SCREEN_WIDTH {
            return;
        }

        // Which tile map to use for window (bit 6 of LCDC)
        let tile_map_base: usize = if self.lcdc & 0x40 != 0 { 0x1C00 } else { 0x1800 };

        // Tile data area (same as BG, bit 4)
        let tile_data_base: usize = if self.lcdc & 0x10 != 0 { 0x0000 } else { 0x0800 };
        let signed_addressing = self.lcdc & 0x10 == 0;

        let tile_row = window_y / 8;
        let pixel_y_in_tile = window_y % 8;

        for screen_x in window_x_start..SCREEN_WIDTH {
            let window_x = screen_x - window_x_start;
            let tile_col = window_x / 8;
            let pixel_x_in_tile = window_x % 8;

            let tile_map_addr = tile_map_base + tile_row * 32 + tile_col;
            let tile_index = vram[tile_map_addr];

            let tile_data_addr = if signed_addressing {
                let signed_index = tile_index as i8 as i16;
                (tile_data_base as i16 + 0x800 + signed_index * 16) as usize
            } else {
                tile_data_base + (tile_index as usize) * 16
            };

            let byte1 = vram[tile_data_addr + pixel_y_in_tile * 2];
            let byte2 = vram[tile_data_addr + pixel_y_in_tile * 2 + 1];

            let bit = 7 - pixel_x_in_tile;
            let color_low = (byte1 >> bit) & 1;
            let color_high = (byte2 >> bit) & 1;
            let color_index = (color_high << 1) | color_low;

            let palette_color = (self.bgp >> (color_index * 2)) & 0x03;

            self.framebuffer[ly * SCREEN_WIDTH + screen_x] = COLORS[palette_color as usize];
        }
    }

    /// Render sprites for one scanline
    fn render_sprites_scanline(&mut self, vram: &[u8], oam: &[u8], ly: usize) {
        let sprite_height = if self.lcdc & 0x04 != 0 { 16 } else { 8 };
        let ly_i32 = ly as i32;

        // Collect sprites on this scanline (max 10)
        let mut sprites_on_line: Vec<(usize, i32, i32)> = Vec::with_capacity(10);

        for sprite_idx in 0..40 {
            let oam_addr = sprite_idx * 4;
            let sprite_y = oam[oam_addr] as i32 - 16;
            let sprite_x = oam[oam_addr + 1] as i32 - 8;

            // Check if sprite is on this scanline
            if ly_i32 >= sprite_y && ly_i32 < sprite_y + sprite_height {
                sprites_on_line.push((sprite_idx, sprite_x, sprite_y));
                if sprites_on_line.len() >= 10 {
                    break;
                }
            }
        }

        // Sort by X coordinate (lower X = higher priority)
        // For same X, lower OAM index = higher priority (already sorted)
        sprites_on_line.sort_by(|a, b| a.1.cmp(&b.1));

        // Render sprites (in reverse order so higher priority draws last)
        for (sprite_idx, sprite_x, sprite_y) in sprites_on_line.iter().rev() {
            let oam_addr = sprite_idx * 4;
            let tile_index = oam[oam_addr + 2] as usize;
            let attributes = oam[oam_addr + 3];

            let flip_y = attributes & 0x40 != 0;
            let flip_x = attributes & 0x20 != 0;
            let bg_priority = attributes & 0x80 != 0;
            let palette = if attributes & 0x10 != 0 { self.obp1 } else { self.obp0 };

            // Which row of the sprite are we drawing?
            let mut sprite_row = ly_i32 - sprite_y;
            if flip_y {
                sprite_row = sprite_height - 1 - sprite_row;
            }

            // For 8x16 sprites, mask bit 0 of tile index
            let actual_tile = if sprite_height == 16 {
                if sprite_row >= 8 {
                    (tile_index | 1) as usize
                } else {
                    (tile_index & 0xFE) as usize
                }
            } else {
                tile_index
            };
            let row_in_tile = (sprite_row % 8) as usize;

            // Sprites always use 0x8000 addressing
            let tile_data_addr = actual_tile * 16 + row_in_tile * 2;
            let byte1 = vram[tile_data_addr];
            let byte2 = vram[tile_data_addr + 1];

            for pixel_x in 0..8i32 {
                let screen_x = sprite_x + pixel_x;
                if screen_x < 0 || screen_x >= SCREEN_WIDTH as i32 {
                    continue;
                }

                let bit = if flip_x { pixel_x } else { 7 - pixel_x } as usize;
                let color_low = (byte1 >> bit) & 1;
                let color_high = (byte2 >> bit) & 1;
                let color_index = (color_high << 1) | color_low;

                // Color 0 is transparent for sprites
                if color_index == 0 {
                    continue;
                }

                // BG priority: if set, sprite only shows over BG color 0
                if bg_priority {
                    let fb_idx = ly * SCREEN_WIDTH + screen_x as usize;
                    if self.framebuffer[fb_idx] != COLORS[0] {
                        continue;
                    }
                }

                let palette_color = (palette >> (color_index * 2)) & 0x03;
                self.framebuffer[ly * SCREEN_WIDTH + screen_x as usize] = COLORS[palette_color as usize];
            }
        }
    }

    /// Check if background is enabled (bit 0 of LCDC)
    fn bg_enabled(&self) -> bool {
        self.lcdc & 0x01 != 0
    }

    /// Check if window is enabled (bit 5 of LCDC)
    fn window_enabled(&self) -> bool {
        self.lcdc & 0x20 != 0
    }

    /// Check if sprites are enabled (bit 1 of LCDC)
    fn sprites_enabled(&self) -> bool {
        self.lcdc & 0x02 != 0
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

    // Empty VRAM and OAM for tests that don't need rendering
    const EMPTY_VRAM: &[u8] = &[0; 0x2000];
    const EMPTY_OAM: &[u8] = &[0; 160];

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
        ppu.step(455, EMPTY_VRAM, EMPTY_OAM);
        assert_eq!(ppu.ly(), 0);

        // Step 1 more cycle - should now be on scanline 1
        ppu.step(1, EMPTY_VRAM, EMPTY_OAM);
        assert_eq!(ppu.ly(), 1);
    }

    #[test]
    fn test_ly_wraps_at_154() {
        let mut ppu = Ppu::new();

        // Advance through all 154 scanlines
        for _ in 0..154 {
            ppu.step(CYCLES_PER_SCANLINE, EMPTY_VRAM, EMPTY_OAM);
        }

        // Should wrap back to 0
        assert_eq!(ppu.ly(), 0);
    }

    #[test]
    fn test_vblank_starts_at_144() {
        let mut ppu = Ppu::new();

        // Advance to scanline 143
        for _ in 0..143 {
            ppu.step(CYCLES_PER_SCANLINE, EMPTY_VRAM, EMPTY_OAM);
        }
        assert_eq!(ppu.ly(), 143);
        assert!(!ppu.in_vblank());

        // Advance to scanline 144
        ppu.step(CYCLES_PER_SCANLINE, EMPTY_VRAM, EMPTY_OAM);
        assert_eq!(ppu.ly(), 144);
        assert!(ppu.in_vblank());
        assert_eq!(ppu.mode(), Mode::VBlank);
    }

    #[test]
    fn test_read_ly_register() {
        let mut ppu = Ppu::new();
        assert_eq!(ppu.read(0xFF44), 0);

        ppu.step(CYCLES_PER_SCANLINE * 10, EMPTY_VRAM, EMPTY_OAM);
        assert_eq!(ppu.read(0xFF44), 10);
    }

    #[test]
    fn test_ly_is_read_only() {
        let mut ppu = Ppu::new();
        ppu.step(CYCLES_PER_SCANLINE * 5, EMPTY_VRAM, EMPTY_OAM);
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
        ppu.step(80, EMPTY_VRAM, EMPTY_OAM);
        let stat = ppu.read(0xFF41);
        assert_eq!(stat & 0x03, Mode::Drawing as u8);

        // Advance to HBlank
        ppu.step(172, EMPTY_VRAM, EMPTY_OAM);
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
            ppu.step(CYCLES_PER_SCANLINE, EMPTY_VRAM, EMPTY_OAM);
        }
        assert_eq!(ppu.ly(), 5);

        // LY=LYC, flag should be set
        assert_eq!(ppu.read(0xFF41) & 0x04, 0x04);
    }

    #[test]
    fn test_lcd_disable_resets_ly() {
        let mut ppu = Ppu::new();

        // Advance to some scanline
        ppu.step(CYCLES_PER_SCANLINE * 50, EMPTY_VRAM, EMPTY_OAM);
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
        ppu.step(CYCLES_PER_SCANLINE * 10, EMPTY_VRAM, EMPTY_OAM);
        assert_eq!(ppu.ly(), 0);
    }

    #[test]
    fn test_multiple_scanlines_in_one_step() {
        let mut ppu = Ppu::new();

        // Step 3 full scanlines worth of cycles
        ppu.step(CYCLES_PER_SCANLINE * 3, EMPTY_VRAM, EMPTY_OAM);
        assert_eq!(ppu.ly(), 3);
    }
}
