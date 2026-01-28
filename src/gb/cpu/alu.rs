use super::Cpu;

impl Cpu {
    /// ADD A, value - Sets Z=*, N=0, H=*, C=*
    pub(super) fn alu_add(&mut self, value: u8) {
        let a = self.get_reg_a();
        let result = a.wrapping_add(value);
        self.flags.z = result == 0;
        self.flags.n = false;
        self.flags.h = (a & 0x0F) + (value & 0x0F) > 0x0F;
        self.flags.c = (a as u16) + (value as u16) > 0xFF;
        self.set_reg_a(result);
    }

    /// ADC A, value - Add with carry. Sets Z=*, N=0, H=*, C=*
    pub(super) fn alu_adc(&mut self, value: u8) {
        let a = self.get_reg_a();
        let carry = if self.flags.c { 1u8 } else { 0u8 };
        let result = a.wrapping_add(value).wrapping_add(carry);
        self.flags.z = result == 0;
        self.flags.n = false;
        self.flags.h = (a & 0x0F) + (value & 0x0F) + carry > 0x0F;
        self.flags.c = (a as u16) + (value as u16) + (carry as u16) > 0xFF;
        self.set_reg_a(result);
    }

    /// SUB A, value - Sets Z=*, N=1, H=*, C=*
    pub(super) fn alu_sub(&mut self, value: u8) {
        let a = self.get_reg_a();
        let result = a.wrapping_sub(value);
        self.flags.z = result == 0;
        self.flags.n = true;
        self.flags.h = (a & 0x0F) < (value & 0x0F);
        self.flags.c = a < value;
        self.set_reg_a(result);
    }

    /// SBC A, value - Subtract with carry. Sets Z=*, N=1, H=*, C=*
    pub(super) fn alu_sbc(&mut self, value: u8) {
        let a = self.get_reg_a();
        let carry = if self.flags.c { 1u8 } else { 0u8 };
        let result = a.wrapping_sub(value).wrapping_sub(carry);
        self.flags.z = result == 0;
        self.flags.n = true;
        self.flags.h = (a & 0x0F) < (value & 0x0F) + carry;
        self.flags.c = (a as u16) < (value as u16) + (carry as u16);
        self.set_reg_a(result);
    }

    /// AND A, value - Sets Z=*, N=0, H=1, C=0
    pub(super) fn alu_and(&mut self, value: u8) {
        let result = self.get_reg_a() & value;
        self.flags.z = result == 0;
        self.flags.n = false;
        self.flags.h = true;
        self.flags.c = false;
        self.set_reg_a(result);
    }

    /// XOR A, value - Sets Z=*, N=0, H=0, C=0
    pub(super) fn alu_xor(&mut self, value: u8) {
        let result = self.get_reg_a() ^ value;
        self.flags.z = result == 0;
        self.flags.n = false;
        self.flags.h = false;
        self.flags.c = false;
        self.set_reg_a(result);
    }

    /// OR A, value - Sets Z=*, N=0, H=0, C=0
    pub(super) fn alu_or(&mut self, value: u8) {
        let result = self.get_reg_a() | value;
        self.flags.z = result == 0;
        self.flags.n = false;
        self.flags.h = false;
        self.flags.c = false;
        self.set_reg_a(result);
    }

    /// CP A, value - Compare (like SUB but discard result). Sets Z=*, N=1, H=*, C=*
    pub(super) fn alu_cp(&mut self, value: u8) {
        let a = self.get_reg_a();
        self.flags.z = a == value;
        self.flags.n = true;
        self.flags.h = (a & 0x0F) < (value & 0x0F);
        self.flags.c = a < value;
    }

    /// 8-bit INC helper - sets Z, N=0, H, C unchanged
    pub(super) fn alu_inc(&mut self, val: u8) -> u8 {
        let result = val.wrapping_add(1);
        self.flags.z = result == 0;
        self.flags.n = false;
        self.flags.h = (val & 0x0F) == 0x0F;
        result
    }

    /// 8-bit DEC helper - sets Z, N=1, H, C unchanged
    pub(super) fn alu_dec(&mut self, val: u8) -> u8 {
        let result = val.wrapping_sub(1);
        self.flags.z = result == 0;
        self.flags.n = true;
        self.flags.h = (val & 0x0F) == 0x00;
        result
    }

    /// 16-bit ADD HL,rr helper - sets N=0, H, C (Z unchanged)
    pub(super) fn alu_add_hl(&mut self, val: u16) {
        let hl = self.regs_hl.get();
        let result = hl.wrapping_add(val);
        self.flags.n = false;
        self.flags.h = (hl & 0x0FFF) + (val & 0x0FFF) > 0x0FFF;
        self.flags.c = hl > 0xFFFF - val;
        self.regs_hl.set(result);
    }
}
