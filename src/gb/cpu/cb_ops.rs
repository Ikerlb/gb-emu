use super::{Cpu, Reg8};
use crate::gb::interconnect::Interconnect;

impl Cpu {
    /// RLC - Rotate left, old bit 7 to carry. Z=*, N=0, H=0, C=*
    pub(super) fn cb_rlc(&mut self, value: u8) -> u8 {
        let carry = (value >> 7) & 1;
        let result = (value << 1) | carry;
        self.flags.z = result == 0;
        self.flags.n = false;
        self.flags.h = false;
        self.flags.c = carry == 1;
        result
    }

    /// RRC - Rotate right, old bit 0 to carry. Z=*, N=0, H=0, C=*
    pub(super) fn cb_rrc(&mut self, value: u8) -> u8 {
        let carry = value & 1;
        let result = (value >> 1) | (carry << 7);
        self.flags.z = result == 0;
        self.flags.n = false;
        self.flags.h = false;
        self.flags.c = carry == 1;
        result
    }

    /// RL - Rotate left through carry. Z=*, N=0, H=0, C=*
    pub(super) fn cb_rl(&mut self, value: u8) -> u8 {
        let old_carry = if self.flags.c { 1 } else { 0 };
        let new_carry = (value >> 7) & 1;
        let result = (value << 1) | old_carry;
        self.flags.z = result == 0;
        self.flags.n = false;
        self.flags.h = false;
        self.flags.c = new_carry == 1;
        result
    }

    /// RR - Rotate right through carry. Z=*, N=0, H=0, C=*
    pub(super) fn cb_rr(&mut self, value: u8) -> u8 {
        let old_carry = if self.flags.c { 1u8 } else { 0 };
        let new_carry = value & 1;
        let result = (value >> 1) | (old_carry << 7);
        self.flags.z = result == 0;
        self.flags.n = false;
        self.flags.h = false;
        self.flags.c = new_carry == 1;
        result
    }

    /// SLA - Shift left arithmetic (bit 0 = 0). Z=*, N=0, H=0, C=*
    pub(super) fn cb_sla(&mut self, value: u8) -> u8 {
        let carry = (value >> 7) & 1;
        let result = value << 1;
        self.flags.z = result == 0;
        self.flags.n = false;
        self.flags.h = false;
        self.flags.c = carry == 1;
        result
    }

    /// SRA - Shift right arithmetic (bit 7 unchanged). Z=*, N=0, H=0, C=*
    pub(super) fn cb_sra(&mut self, value: u8) -> u8 {
        let carry = value & 1;
        let result = (value >> 1) | (value & 0x80); // Preserve bit 7
        self.flags.z = result == 0;
        self.flags.n = false;
        self.flags.h = false;
        self.flags.c = carry == 1;
        result
    }

    /// SWAP - Swap upper and lower nibbles. Z=*, N=0, H=0, C=0
    pub(super) fn cb_swap(&mut self, value: u8) -> u8 {
        let result = (value >> 4) | (value << 4);
        self.flags.z = result == 0;
        self.flags.n = false;
        self.flags.h = false;
        self.flags.c = false;
        result
    }

    /// SRL - Shift right logical (bit 7 = 0). Z=*, N=0, H=0, C=*
    pub(super) fn cb_srl(&mut self, value: u8) -> u8 {
        let carry = value & 1;
        let result = value >> 1;
        self.flags.z = result == 0;
        self.flags.n = false;
        self.flags.h = false;
        self.flags.c = carry == 1;
        result
    }

    /// BIT - Test bit n. Z=*, N=0, H=1, C=-
    pub(super) fn cb_bit(&mut self, bit: u8, value: u8) {
        self.flags.z = (value & (1 << bit)) == 0;
        self.flags.n = false;
        self.flags.h = true;
        // C flag unchanged
    }

    /// RES - Reset bit n. No flags affected.
    pub(super) fn cb_res(&self, bit: u8, value: u8) -> u8 {
        value & !(1 << bit)
    }

    /// SET - Set bit n. No flags affected.
    pub(super) fn cb_set(&self, bit: u8, value: u8) -> u8 {
        value | (1 << bit)
    }

    /// Execute CB-prefixed opcode, returns cycle count
    pub(super) fn execute_cb(&mut self, inter: &mut Interconnect) -> usize {
        let cb_op = self.read_imm8(inter);
        let reg = Reg8::from_bits(cb_op & 0x07);
        let is_hl_ref = reg == Reg8::HLRef;

        // Operations 0x40-0xFF use bits 5-3 as bit number
        let bit = (cb_op >> 3) & 0x07;

        match cb_op {
            // RLC r (0x00-0x07)
            0x00..=0x07 => {
                let val = self.read_r8(inter, reg);
                let result = self.cb_rlc(val);
                self.write_r8(inter, reg, result);
                if is_hl_ref { 16 } else { 8 }
            }
            // RRC r (0x08-0x0F)
            0x08..=0x0F => {
                let val = self.read_r8(inter, reg);
                let result = self.cb_rrc(val);
                self.write_r8(inter, reg, result);
                if is_hl_ref { 16 } else { 8 }
            }
            // RL r (0x10-0x17)
            0x10..=0x17 => {
                let val = self.read_r8(inter, reg);
                let result = self.cb_rl(val);
                self.write_r8(inter, reg, result);
                if is_hl_ref { 16 } else { 8 }
            }
            // RR r (0x18-0x1F)
            0x18..=0x1F => {
                let val = self.read_r8(inter, reg);
                let result = self.cb_rr(val);
                self.write_r8(inter, reg, result);
                if is_hl_ref { 16 } else { 8 }
            }
            // SLA r (0x20-0x27)
            0x20..=0x27 => {
                let val = self.read_r8(inter, reg);
                let result = self.cb_sla(val);
                self.write_r8(inter, reg, result);
                if is_hl_ref { 16 } else { 8 }
            }
            // SRA r (0x28-0x2F)
            0x28..=0x2F => {
                let val = self.read_r8(inter, reg);
                let result = self.cb_sra(val);
                self.write_r8(inter, reg, result);
                if is_hl_ref { 16 } else { 8 }
            }
            // SWAP r (0x30-0x37)
            0x30..=0x37 => {
                let val = self.read_r8(inter, reg);
                let result = self.cb_swap(val);
                self.write_r8(inter, reg, result);
                if is_hl_ref { 16 } else { 8 }
            }
            // SRL r (0x38-0x3F)
            0x38..=0x3F => {
                let val = self.read_r8(inter, reg);
                let result = self.cb_srl(val);
                self.write_r8(inter, reg, result);
                if is_hl_ref { 16 } else { 8 }
            }
            // BIT n, r (0x40-0x7F)
            0x40..=0x7F => {
                let val = self.read_r8(inter, reg);
                self.cb_bit(bit, val);
                if is_hl_ref { 12 } else { 8 }
            }
            // RES n, r (0x80-0xBF)
            0x80..=0xBF => {
                let val = self.read_r8(inter, reg);
                let result = self.cb_res(bit, val);
                self.write_r8(inter, reg, result);
                if is_hl_ref { 16 } else { 8 }
            }
            // SET n, r (0xC0-0xFF)
            0xC0..=0xFF => {
                let val = self.read_r8(inter, reg);
                let result = self.cb_set(bit, val);
                self.write_r8(inter, reg, result);
                if is_hl_ref { 16 } else { 8 }
            }
        }
    }
}
