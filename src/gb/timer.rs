/// Timer - Handles Game Boy timer subsystem
///
/// Registers:
/// - DIV (0xFF04): Divider register, increments at 16384 Hz (every 256 T-cycles)
/// - TIMA (0xFF05): Timer counter, increments at rate set by TAC
/// - TMA (0xFF06): Timer modulo, value loaded into TIMA on overflow
/// - TAC (0xFF07): Timer control (bit 2 = enable, bits 0-1 = clock select)

/// Clock dividers for TIMA based on TAC bits 0-1
/// These are in T-cycles (not M-cycles)
const TIMER_CLOCKS: [u32; 4] = [
    1024, // 00: 4096 Hz (CPU clock / 1024)
    16,   // 01: 262144 Hz (CPU clock / 16)
    64,   // 10: 65536 Hz (CPU clock / 64)
    256,  // 11: 16384 Hz (CPU clock / 256)
];

pub struct Timer {
    /// DIV register - upper 8 bits of internal 16-bit counter
    div_counter: u16,
    /// TIMA register - timer counter
    tima: u8,
    /// TMA register - timer modulo (reload value)
    tma: u8,
    /// TAC register - timer control
    tac: u8,
    /// Internal counter for TIMA increments
    tima_counter: u32,
    /// Flag indicating TIMA overflow (triggers interrupt)
    pub interrupt_requested: bool,
}

impl Timer {
    pub fn new() -> Self {
        Self {
            div_counter: 0,
            tima: 0,
            tma: 0,
            tac: 0,
            tima_counter: 0,
            interrupt_requested: false,
        }
    }

    /// Advance timer by given number of T-cycles
    pub fn step(&mut self, cycles: u32) {
        // DIV always increments (upper 8 bits of 16-bit counter)
        self.div_counter = self.div_counter.wrapping_add(cycles as u16);

        // TIMA only increments if timer is enabled (TAC bit 2)
        if self.tac & 0x04 != 0 {
            self.tima_counter += cycles;

            let clock_select = (self.tac & 0x03) as usize;
            let threshold = TIMER_CLOCKS[clock_select];

            while self.tima_counter >= threshold {
                self.tima_counter -= threshold;

                // Increment TIMA, check for overflow
                let (new_tima, overflow) = self.tima.overflowing_add(1);
                if overflow {
                    self.tima = self.tma; // Reload from TMA
                    self.interrupt_requested = true;
                } else {
                    self.tima = new_tima;
                }
            }
        }
    }

    /// Read timer register
    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            0xFF04 => (self.div_counter >> 8) as u8, // DIV is upper 8 bits
            0xFF05 => self.tima,
            0xFF06 => self.tma,
            0xFF07 => self.tac | 0xF8, // Upper 5 bits read as 1
            _ => 0xFF,
        }
    }

    /// Write timer register
    pub fn write(&mut self, addr: u16, val: u8) {
        match addr {
            0xFF04 => {
                // Writing any value to DIV resets it to 0
                self.div_counter = 0;
            }
            0xFF05 => self.tima = val,
            0xFF06 => self.tma = val,
            0xFF07 => {
                // Changing clock select resets the counter
                let old_clock = self.tac & 0x03;
                let new_clock = val & 0x03;
                if old_clock != new_clock {
                    self.tima_counter = 0;
                }
                self.tac = val & 0x07; // Only bits 0-2 are used
            }
            _ => {}
        }
    }

    /// Clear the interrupt request flag (called after interrupt is handled)
    pub fn clear_interrupt(&mut self) {
        self.interrupt_requested = false;
    }
}

impl Default for Timer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timer_new() {
        let timer = Timer::new();
        assert_eq!(timer.read(0xFF04), 0); // DIV
        assert_eq!(timer.read(0xFF05), 0); // TIMA
        assert_eq!(timer.read(0xFF06), 0); // TMA
        assert_eq!(timer.read(0xFF07) & 0x07, 0); // TAC
    }

    #[test]
    fn test_div_increments() {
        let mut timer = Timer::new();

        // DIV increments every 256 T-cycles (upper 8 bits of 16-bit counter)
        timer.step(255);
        assert_eq!(timer.read(0xFF04), 0);

        timer.step(1);
        assert_eq!(timer.read(0xFF04), 1);
    }

    #[test]
    fn test_div_reset_on_write() {
        let mut timer = Timer::new();

        timer.step(1024); // DIV = 4
        assert_eq!(timer.read(0xFF04), 4);

        // Writing any value resets DIV
        timer.write(0xFF04, 0x42);
        assert_eq!(timer.read(0xFF04), 0);
    }

    #[test]
    fn test_tima_disabled_by_default() {
        let mut timer = Timer::new();

        // TIMA shouldn't increment when timer is disabled
        timer.step(10000);
        assert_eq!(timer.read(0xFF05), 0);
    }

    #[test]
    fn test_tima_increments_when_enabled() {
        let mut timer = Timer::new();

        // Enable timer with clock 00 (1024 cycles)
        timer.write(0xFF07, 0x04);

        timer.step(1023);
        assert_eq!(timer.read(0xFF05), 0);

        timer.step(1);
        assert_eq!(timer.read(0xFF05), 1);
    }

    #[test]
    fn test_tima_overflow_reloads_tma() {
        let mut timer = Timer::new();

        // Set TMA to 0x80
        timer.write(0xFF06, 0x80);
        // Set TIMA to 0xFF
        timer.write(0xFF05, 0xFF);
        // Enable timer with fastest clock (16 cycles)
        timer.write(0xFF07, 0x05);

        // Step enough to overflow
        timer.step(16);

        // TIMA should be reloaded from TMA
        assert_eq!(timer.read(0xFF05), 0x80);
        assert!(timer.interrupt_requested);
    }

    #[test]
    fn test_timer_clock_select() {
        // Test clock 01 (16 cycles)
        let mut timer = Timer::new();
        timer.write(0xFF07, 0x05); // Enable + clock 01

        timer.step(15);
        assert_eq!(timer.read(0xFF05), 0);

        timer.step(1);
        assert_eq!(timer.read(0xFF05), 1);
    }
}
