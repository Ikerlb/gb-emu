/// Joypad input handling
///
/// The joypad register (0xFF00/P1) selects which button group to read:
/// - Bit 4 = 0: Select D-pad (Down, Up, Left, Right)
/// - Bit 5 = 0: Select buttons (Start, Select, B, A)
/// - Bits 0-3: Button states (0 = pressed, active low)

/// Joypad buttons
#[derive(Debug, Clone, Copy)]
pub enum Button {
    Right,
    Left,
    Up,
    Down,
    A,
    B,
    Select,
    Start,
}

pub struct Joypad {
    /// Currently pressed buttons (bitfield)
    /// Bit 0: Right/A
    /// Bit 1: Left/B
    /// Bit 2: Up/Select
    /// Bit 3: Down/Start
    /// Bits 4-5: Select lines (active low)
    direction_buttons: u8, // Right, Left, Up, Down (bits 0-3)
    action_buttons: u8,    // A, B, Select, Start (bits 0-3)
    select: u8,            // Bits 4-5: selection mode
}

impl Joypad {
    pub fn new() -> Self {
        Self {
            direction_buttons: 0x0F, // All released (1 = not pressed)
            action_buttons: 0x0F,    // All released
            select: 0x30,            // Both select bits high (nothing selected)
        }
    }

    /// Press a button
    pub fn press(&mut self, button: Button) {
        match button {
            Button::Right => self.direction_buttons &= !0x01,
            Button::Left => self.direction_buttons &= !0x02,
            Button::Up => self.direction_buttons &= !0x04,
            Button::Down => self.direction_buttons &= !0x08,
            Button::A => self.action_buttons &= !0x01,
            Button::B => self.action_buttons &= !0x02,
            Button::Select => self.action_buttons &= !0x04,
            Button::Start => self.action_buttons &= !0x08,
        }
    }

    /// Release a button
    pub fn release(&mut self, button: Button) {
        match button {
            Button::Right => self.direction_buttons |= 0x01,
            Button::Left => self.direction_buttons |= 0x02,
            Button::Up => self.direction_buttons |= 0x04,
            Button::Down => self.direction_buttons |= 0x08,
            Button::A => self.action_buttons |= 0x01,
            Button::B => self.action_buttons |= 0x02,
            Button::Select => self.action_buttons |= 0x04,
            Button::Start => self.action_buttons |= 0x08,
        }
    }

    /// Read the joypad register
    pub fn read(&self) -> u8 {
        let mut result = 0xCF; // Bits 6-7 always 1, bits 0-3 default high

        // Check which button group is selected (active low)
        let select_direction = self.select & 0x10 == 0;
        let select_action = self.select & 0x20 == 0;

        // Apply select bits
        result = (result & 0xCF) | (self.select & 0x30);

        // Get button states (OR together if both selected)
        let mut buttons = 0x0F;
        if select_direction {
            buttons &= self.direction_buttons;
        }
        if select_action {
            buttons &= self.action_buttons;
        }

        // Set button bits (low nibble)
        result = (result & 0xF0) | (buttons & 0x0F);

        result
    }

    /// Write to the joypad register (only bits 4-5 are writable)
    pub fn write(&mut self, value: u8) {
        // Only bits 4-5 can be written (select lines)
        self.select = value & 0x30;
    }

    /// Check if any button is currently pressed (for interrupt)
    #[allow(dead_code)] // May be used for joypad interrupt support
    pub fn any_pressed(&self) -> bool {
        self.direction_buttons != 0x0F || self.action_buttons != 0x0F
    }
}

impl Default for Joypad {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_state() {
        let joypad = Joypad::new();
        // Initially both select bits high, no buttons readable
        assert_eq!(joypad.read() & 0x0F, 0x0F);
    }

    #[test]
    fn test_direction_buttons() {
        let mut joypad = Joypad::new();

        // Select direction buttons
        joypad.write(0x20); // Bit 4 = 0 (select directions), bit 5 = 1

        // All released
        assert_eq!(joypad.read() & 0x0F, 0x0F);

        // Press Right
        joypad.press(Button::Right);
        assert_eq!(joypad.read() & 0x0F, 0x0E); // Bit 0 = 0

        // Press Up as well
        joypad.press(Button::Up);
        assert_eq!(joypad.read() & 0x0F, 0x0A); // Bits 0 and 2 = 0

        // Release Right
        joypad.release(Button::Right);
        assert_eq!(joypad.read() & 0x0F, 0x0B); // Bit 2 = 0
    }

    #[test]
    fn test_action_buttons() {
        let mut joypad = Joypad::new();

        // Select action buttons
        joypad.write(0x10); // Bit 4 = 1, bit 5 = 0 (select actions)

        // All released
        assert_eq!(joypad.read() & 0x0F, 0x0F);

        // Press A
        joypad.press(Button::A);
        assert_eq!(joypad.read() & 0x0F, 0x0E); // Bit 0 = 0

        // Press Start
        joypad.press(Button::Start);
        assert_eq!(joypad.read() & 0x0F, 0x06); // Bits 0 and 3 = 0
    }

    #[test]
    fn test_select_bits_preserved() {
        let mut joypad = Joypad::new();

        joypad.write(0x10);
        assert_eq!(joypad.read() & 0x30, 0x10);

        joypad.write(0x20);
        assert_eq!(joypad.read() & 0x30, 0x20);
    }

    #[test]
    fn test_both_selected() {
        let mut joypad = Joypad::new();

        // Select both (both bits low)
        joypad.write(0x00);

        // Press Right (direction) and A (action)
        joypad.press(Button::Right);
        joypad.press(Button::A);

        // Both should show as pressed (bits ANDed together)
        assert_eq!(joypad.read() & 0x0F, 0x0E); // Bit 0 = 0 (both pressed)
    }
}
