use super::*;
use crate::gb::interconnect::Interconnect;

fn create_test_interconnect() -> Interconnect {
    // Create a minimal ROM for testing
    let mut rom = vec![0; 0x8000];
    // Add some test data
    rom[0x0100] = 0x00; // NOP
    Interconnect::new(rom)
}

/// Create interconnect with specific ROM data at given addresses
fn create_test_interconnect_with_rom(data: &[(usize, u8)]) -> Interconnect {
    let mut rom = vec![0; 0x8000];
    for &(addr, val) in data {
        if addr < rom.len() {
            rom[addr] = val;
        }
    }
    Interconnect::new(rom)
}

#[test]
fn test_cpu_initialization() {
    let cpu = Cpu::new();
    // Test initial register values match Game Boy hardware
    assert_eq!(cpu.reg_pc, 0x0100);
    assert_eq!(cpu.reg_sp, 0xFFFE);
    assert_eq!(cpu.regs_af.get(), 0x01B0);
    assert_eq!(cpu.regs_bc.get(), 0x0013);
    assert_eq!(cpu.regs_de.get(), 0x00D8);
    assert_eq!(cpu.regs_hl.get(), 0x014D);
}

#[test]
fn test_nop_opcode() {
    let mut cpu = Cpu::new();
    let mut inter = create_test_interconnect();

    // Execute NOP (0x00)
    let cycles = cpu.execute_opcode(&mut inter, 0x00);

    assert_eq!(cycles, 4);
    // NOP shouldn't change any registers
}

#[test]
fn test_dec_bc_opcode() {
    let mut cpu = Cpu::new();
    let mut inter = create_test_interconnect();

    let initial_bc = cpu.regs_bc.get();

    // Execute DEC BC (0x0B)
    let cycles = cpu.execute_opcode(&mut inter, 0x0B);

    assert_eq!(cycles, 8);
    assert_eq!(cpu.regs_bc.get(), initial_bc.wrapping_sub(1));
}

#[test]
fn test_cpl_opcode() {
    let mut cpu = Cpu::new();
    let mut inter = create_test_interconnect();

    // Set A register to a known value
    cpu.set_reg_a(0b10101010);

    // Execute CPL (0x2F) - complement A register
    let cycles = cpu.execute_opcode(&mut inter, 0x2F);

    assert_eq!(cycles, 4);
    assert_eq!(cpu.get_reg_a(), 0b01010101);
    assert!(cpu.flags.n); // N flag should be set
    assert!(cpu.flags.h); // H flag should be set
}

#[test]
fn test_ld_c_b_opcode() {
    let mut cpu = Cpu::new();
    let mut inter = create_test_interconnect();

    // Set B register to a known value
    cpu.set_reg_b(0x42);

    // Execute LD C, B (0x48)
    let cycles = cpu.execute_opcode(&mut inter, 0x48);

    assert_eq!(cycles, 4);
    assert_eq!(cpu.get_reg_c(), 0x42);
    assert_eq!(cpu.get_reg_b(), 0x42); // B should remain unchanged
}

#[test]
fn test_ld_c_d_opcode() {
    let mut cpu = Cpu::new();
    let mut inter = create_test_interconnect();

    // Set D register to a known value
    cpu.set_reg_d(0x37);

    // Execute LD C, D (0x4A)
    let cycles = cpu.execute_opcode(&mut inter, 0x4A);

    assert_eq!(cycles, 4);
    assert_eq!(cpu.get_reg_c(), 0x37);
}

#[test]
fn test_register_getters_setters() {
    let mut cpu = Cpu::new();

    // Test A register
    cpu.set_reg_a(0xFF);
    assert_eq!(cpu.get_reg_a(), 0xFF);

    // Test B register
    cpu.set_reg_b(0xAB);
    assert_eq!(cpu.get_reg_b(), 0xAB);

    // Test D register
    cpu.set_reg_d(0xCD);
    assert_eq!(cpu.get_reg_d(), 0xCD);
}

#[test]
fn test_flag_setters() {
    let mut cpu = Cpu::new();

    cpu.set_zero_flag(true);
    assert!(cpu.flags.z);

    cpu.set_subtract_flag(true);
    assert!(cpu.flags.n);

    cpu.set_half_carry_flag(true);
    assert!(cpu.flags.h);

    cpu.set_carry_flag(true);
    assert!(cpu.flags.c);
}

// ========== 16-bit Load Tests ==========

#[test]
fn test_ld_bc_d16() {
    let mut cpu = Cpu::new();
    // Place immediate value 0x1234 at PC (little-endian: 0x34, 0x12)
    let mut inter = create_test_interconnect_with_rom(&[
        (0x0100, 0x34),
        (0x0101, 0x12),
    ]);
    cpu.reg_pc = 0x0100;

    let cycles = cpu.execute_opcode(&mut inter, 0x01); // LD BC,d16

    assert_eq!(cycles, 12);
    assert_eq!(cpu.regs_bc.get(), 0x1234);
    assert_eq!(cpu.reg_pc, 0x0102); // PC advanced by 2
}

#[test]
fn test_ld_de_d16() {
    let mut cpu = Cpu::new();
    let mut inter = create_test_interconnect_with_rom(&[
        (0x0100, 0xCD),
        (0x0101, 0xAB),
    ]);
    cpu.reg_pc = 0x0100;

    let cycles = cpu.execute_opcode(&mut inter, 0x11); // LD DE,d16

    assert_eq!(cycles, 12);
    assert_eq!(cpu.regs_de.get(), 0xABCD);
}

#[test]
fn test_ld_hl_d16() {
    let mut cpu = Cpu::new();
    let mut inter = create_test_interconnect_with_rom(&[
        (0x0100, 0xFF),
        (0x0101, 0xDF),
    ]);
    cpu.reg_pc = 0x0100;

    let cycles = cpu.execute_opcode(&mut inter, 0x21); // LD HL,d16

    assert_eq!(cycles, 12);
    assert_eq!(cpu.regs_hl.get(), 0xDFFF);
}

#[test]
fn test_ld_sp_d16() {
    let mut cpu = Cpu::new();
    let mut inter = create_test_interconnect_with_rom(&[
        (0x0100, 0x00),
        (0x0101, 0xC0),
    ]);
    cpu.reg_pc = 0x0100;

    let cycles = cpu.execute_opcode(&mut inter, 0x31); // LD SP,d16

    assert_eq!(cycles, 12);
    assert_eq!(cpu.reg_sp, 0xC000);
}

// ========== 8-bit Immediate Load Tests ==========

#[test]
fn test_ld_b_d8() {
    let mut cpu = Cpu::new();
    let mut inter = create_test_interconnect_with_rom(&[(0x0100, 0x42)]);
    cpu.reg_pc = 0x0100;

    let cycles = cpu.execute_opcode(&mut inter, 0x06); // LD B,d8

    assert_eq!(cycles, 8);
    assert_eq!(cpu.get_reg_b(), 0x42);
}

#[test]
fn test_ld_a_d8() {
    let mut cpu = Cpu::new();
    let mut inter = create_test_interconnect_with_rom(&[(0x0100, 0xFF)]);
    cpu.reg_pc = 0x0100;

    let cycles = cpu.execute_opcode(&mut inter, 0x3E); // LD A,d8

    assert_eq!(cycles, 8);
    assert_eq!(cpu.get_reg_a(), 0xFF);
}

// ========== Memory Load Tests ==========

#[test]
fn test_ld_bcref_a() {
    let mut cpu = Cpu::new();
    let mut inter = create_test_interconnect();
    cpu.set_reg_a(0x55);
    cpu.regs_bc.set(0xC000); // Point to WRAM

    let cycles = cpu.execute_opcode(&mut inter, 0x02); // LD (BC),A

    assert_eq!(cycles, 8);
    assert_eq!(inter.read(0xC000), 0x55);
}

#[test]
fn test_ld_a_bcref() {
    let mut cpu = Cpu::new();
    let mut inter = create_test_interconnect();
    inter.write(0xC000, 0xAA);
    cpu.regs_bc.set(0xC000);

    let cycles = cpu.execute_opcode(&mut inter, 0x0A); // LD A,(BC)

    assert_eq!(cycles, 8);
    assert_eq!(cpu.get_reg_a(), 0xAA);
}

#[test]
fn test_ld_hli_a() {
    let mut cpu = Cpu::new();
    let mut inter = create_test_interconnect();
    cpu.set_reg_a(0x77);
    cpu.regs_hl.set(0xC000);

    let cycles = cpu.execute_opcode(&mut inter, 0x22); // LD (HL+),A

    assert_eq!(cycles, 8);
    assert_eq!(inter.read(0xC000), 0x77);
    assert_eq!(cpu.regs_hl.get(), 0xC001); // HL incremented
}

#[test]
fn test_ld_hld_a() {
    let mut cpu = Cpu::new();
    let mut inter = create_test_interconnect();
    cpu.set_reg_a(0x88);
    cpu.regs_hl.set(0xC010);

    let cycles = cpu.execute_opcode(&mut inter, 0x32); // LD (HL-),A

    assert_eq!(cycles, 8);
    assert_eq!(inter.read(0xC010), 0x88);
    assert_eq!(cpu.regs_hl.get(), 0xC00F); // HL decremented
}

#[test]
fn test_ld_a_hli() {
    let mut cpu = Cpu::new();
    let mut inter = create_test_interconnect();
    inter.write(0xC000, 0x99);
    cpu.regs_hl.set(0xC000);

    let cycles = cpu.execute_opcode(&mut inter, 0x2A); // LD A,(HL+)

    assert_eq!(cycles, 8);
    assert_eq!(cpu.get_reg_a(), 0x99);
    assert_eq!(cpu.regs_hl.get(), 0xC001);
}

#[test]
fn test_ldh_a8_a() {
    let mut cpu = Cpu::new();
    // Use 0xFF01 (serial data) instead of 0xFF44 (LY) which is PPU register
    let mut inter = create_test_interconnect_with_rom(&[(0x0100, 0x01)]); // offset
    cpu.set_reg_a(0x42);
    cpu.reg_pc = 0x0100;

    let cycles = cpu.execute_opcode(&mut inter, 0xE0); // LDH (a8),A

    assert_eq!(cycles, 12);
    assert_eq!(inter.read(0xFF01), 0x42);
}

#[test]
fn test_ldh_a_a8() {
    let mut cpu = Cpu::new();
    // Use 0xFF01 (serial data) instead of 0xFF44 (LY) which is PPU register
    let mut inter = create_test_interconnect_with_rom(&[(0x0100, 0x01)]); // offset
    inter.write(0xFF01, 0x90);
    cpu.reg_pc = 0x0100;

    let cycles = cpu.execute_opcode(&mut inter, 0xF0); // LDH A,(a8)

    assert_eq!(cycles, 12);
    assert_eq!(cpu.get_reg_a(), 0x90);
}

// ========== Jump Tests ==========

#[test]
fn test_jp_a16() {
    let mut cpu = Cpu::new();
    let mut inter = create_test_interconnect_with_rom(&[
        (0x0100, 0x50),
        (0x0101, 0x01),
    ]);
    cpu.reg_pc = 0x0100;

    let cycles = cpu.execute_opcode(&mut inter, 0xC3); // JP a16

    assert_eq!(cycles, 16);
    assert_eq!(cpu.reg_pc, 0x0150);
}

#[test]
fn test_jp_hl() {
    let mut cpu = Cpu::new();
    let mut inter = create_test_interconnect();
    cpu.regs_hl.set(0x0200);

    let cycles = cpu.execute_opcode(&mut inter, 0xE9); // JP HL

    assert_eq!(cycles, 4);
    assert_eq!(cpu.reg_pc, 0x0200);
}

#[test]
fn test_jr_r8_forward() {
    let mut cpu = Cpu::new();
    let mut inter = create_test_interconnect_with_rom(&[(0x0100, 0x10)]); // offset +16
    cpu.reg_pc = 0x0100;

    let cycles = cpu.execute_opcode(&mut inter, 0x18); // JR r8

    assert_eq!(cycles, 12);
    assert_eq!(cpu.reg_pc, 0x0111); // 0x0101 + 0x10
}

#[test]
fn test_jr_r8_backward() {
    let mut cpu = Cpu::new();
    let mut inter = create_test_interconnect_with_rom(&[(0x0100, 0xFE)]); // offset -2 (signed)
    cpu.reg_pc = 0x0100;

    let cycles = cpu.execute_opcode(&mut inter, 0x18); // JR r8

    assert_eq!(cycles, 12);
    assert_eq!(cpu.reg_pc, 0x00FF); // 0x0101 + (-2)
}

#[test]
fn test_jr_nz_taken() {
    let mut cpu = Cpu::new();
    let mut inter = create_test_interconnect_with_rom(&[(0x0100, 0x05)]);
    cpu.reg_pc = 0x0100;
    cpu.flags.z = false;

    let cycles = cpu.execute_opcode(&mut inter, 0x20); // JR NZ,r8

    assert_eq!(cycles, 12);
    assert_eq!(cpu.reg_pc, 0x0106);
}

#[test]
fn test_jr_nz_not_taken() {
    let mut cpu = Cpu::new();
    let mut inter = create_test_interconnect_with_rom(&[(0x0100, 0x05)]);
    cpu.reg_pc = 0x0100;
    cpu.flags.z = true;

    let cycles = cpu.execute_opcode(&mut inter, 0x20); // JR NZ,r8

    assert_eq!(cycles, 8);
    assert_eq!(cpu.reg_pc, 0x0101); // Only advanced past immediate
}

#[test]
fn test_jp_z_taken() {
    let mut cpu = Cpu::new();
    let mut inter = create_test_interconnect_with_rom(&[
        (0x0100, 0x00),
        (0x0101, 0x02),
    ]);
    cpu.reg_pc = 0x0100;
    cpu.flags.z = true;

    let cycles = cpu.execute_opcode(&mut inter, 0xCA); // JP Z,a16

    assert_eq!(cycles, 16);
    assert_eq!(cpu.reg_pc, 0x0200);
}

#[test]
fn test_jp_z_not_taken() {
    let mut cpu = Cpu::new();
    let mut inter = create_test_interconnect_with_rom(&[
        (0x0100, 0x00),
        (0x0101, 0x02),
    ]);
    cpu.reg_pc = 0x0100;
    cpu.flags.z = false;

    let cycles = cpu.execute_opcode(&mut inter, 0xCA); // JP Z,a16

    assert_eq!(cycles, 12);
    assert_eq!(cpu.reg_pc, 0x0102); // Skipped the jump
}

// ========== Call/Return Tests ==========

#[test]
fn test_call_a16() {
    let mut cpu = Cpu::new();
    let mut inter = create_test_interconnect_with_rom(&[
        (0x0100, 0x00),
        (0x0101, 0x02),
    ]);
    cpu.reg_pc = 0x0100;
    cpu.reg_sp = 0xFFFE;

    let cycles = cpu.execute_opcode(&mut inter, 0xCD); // CALL a16

    assert_eq!(cycles, 24);
    assert_eq!(cpu.reg_pc, 0x0200);
    assert_eq!(cpu.reg_sp, 0xFFFC);
    // Return address should be on stack
    assert_eq!(inter.read(0xFFFC), 0x02); // Low byte
    assert_eq!(inter.read(0xFFFD), 0x01); // High byte
}

#[test]
fn test_ret() {
    let mut cpu = Cpu::new();
    let mut inter = create_test_interconnect();
    cpu.reg_sp = 0xFFFC;
    inter.write(0xFFFC, 0x50); // Return address low (HRAM is writable)
    inter.write(0xFFFD, 0x01); // Return address high

    let cycles = cpu.execute_opcode(&mut inter, 0xC9); // RET

    assert_eq!(cycles, 16);
    assert_eq!(cpu.reg_pc, 0x0150);
    assert_eq!(cpu.reg_sp, 0xFFFE);
}

#[test]
fn test_call_and_ret() {
    let mut cpu = Cpu::new();
    let mut inter = create_test_interconnect_with_rom(&[
        (0x0100, 0x00),
        (0x0101, 0x02),
    ]);
    cpu.reg_pc = 0x0100;
    cpu.reg_sp = 0xFFFE;

    // Execute CALL
    cpu.execute_opcode(&mut inter, 0xCD);
    assert_eq!(cpu.reg_pc, 0x0200);

    // Execute RET
    cpu.execute_opcode(&mut inter, 0xC9);
    assert_eq!(cpu.reg_pc, 0x0102); // Return to after CALL
}

#[test]
fn test_rst_00() {
    let mut cpu = Cpu::new();
    let mut inter = create_test_interconnect();
    cpu.reg_pc = 0x0150;
    cpu.reg_sp = 0xFFFE;

    let cycles = cpu.execute_opcode(&mut inter, 0xC7); // RST 00H

    assert_eq!(cycles, 16);
    assert_eq!(cpu.reg_pc, 0x0000);
    assert_eq!(cpu.reg_sp, 0xFFFC);
}

#[test]
fn test_rst_38() {
    let mut cpu = Cpu::new();
    let mut inter = create_test_interconnect();
    cpu.reg_pc = 0x0150;
    cpu.reg_sp = 0xFFFE;

    let cycles = cpu.execute_opcode(&mut inter, 0xFF); // RST 38H

    assert_eq!(cycles, 16);
    assert_eq!(cpu.reg_pc, 0x0038);
}

// ========== Stack Tests ==========

#[test]
fn test_push_bc() {
    let mut cpu = Cpu::new();
    let mut inter = create_test_interconnect();
    cpu.regs_bc.set(0x1234);
    cpu.reg_sp = 0xFFFE;

    let cycles = cpu.execute_opcode(&mut inter, 0xC5); // PUSH BC

    assert_eq!(cycles, 16);
    assert_eq!(cpu.reg_sp, 0xFFFC);
    assert_eq!(inter.read(0xFFFC), 0x34); // Low byte
    assert_eq!(inter.read(0xFFFD), 0x12); // High byte
}

#[test]
fn test_pop_bc() {
    let mut cpu = Cpu::new();
    let mut inter = create_test_interconnect();
    inter.write(0xFFFC, 0xCD);
    inter.write(0xFFFD, 0xAB);
    cpu.reg_sp = 0xFFFC;

    let cycles = cpu.execute_opcode(&mut inter, 0xC1); // POP BC

    assert_eq!(cycles, 12);
    assert_eq!(cpu.regs_bc.get(), 0xABCD);
    assert_eq!(cpu.reg_sp, 0xFFFE);
}

#[test]
fn test_push_pop_af() {
    let mut cpu = Cpu::new();
    let mut inter = create_test_interconnect();
    cpu.set_reg_a(0x12);
    cpu.flags.z = true;
    cpu.flags.n = false;
    cpu.flags.h = true;
    cpu.flags.c = true;
    cpu.reg_sp = 0xFFFE;

    // PUSH AF
    cpu.execute_opcode(&mut inter, 0xF5);
    assert_eq!(cpu.reg_sp, 0xFFFC);

    // Modify registers
    cpu.set_reg_a(0x00);
    cpu.flags.z = false;
    cpu.flags.h = false;
    cpu.flags.c = false;

    // POP AF - should restore
    cpu.execute_opcode(&mut inter, 0xF1);
    assert_eq!(cpu.get_reg_a(), 0x12);
    assert!(cpu.flags.z);
    assert!(!cpu.flags.n);
    assert!(cpu.flags.h);
    assert!(cpu.flags.c);
}

#[test]
fn test_pop_af_lower_bits_zero() {
    let mut cpu = Cpu::new();
    let mut inter = create_test_interconnect();
    // Set F with lower 4 bits set (should be masked)
    inter.write(0xFFFC, 0xFF); // F = 0xFF, but lower 4 bits ignored
    inter.write(0xFFFD, 0x12); // A = 0x12
    cpu.reg_sp = 0xFFFC;

    cpu.execute_opcode(&mut inter, 0xF1); // POP AF

    // Lower 4 bits should be 0
    assert_eq!(cpu.regs_af.get() & 0x000F, 0);
}

// ========== ADD Tests ==========

#[test]
fn test_add_a_b() {
    let mut cpu = Cpu::new();
    let mut inter = create_test_interconnect();
    cpu.set_reg_a(0x10);
    cpu.set_reg_b(0x20);

    let cycles = cpu.execute_opcode(&mut inter, 0x80); // ADD A,B

    assert_eq!(cycles, 4);
    assert_eq!(cpu.get_reg_a(), 0x30);
    assert!(!cpu.flags.z);
    assert!(!cpu.flags.n);
    assert!(!cpu.flags.h);
    assert!(!cpu.flags.c);
}

#[test]
fn test_add_a_zero_flag() {
    let mut cpu = Cpu::new();
    let mut inter = create_test_interconnect();
    cpu.set_reg_a(0x00);
    cpu.set_reg_b(0x00);

    cpu.execute_opcode(&mut inter, 0x80); // ADD A,B

    assert_eq!(cpu.get_reg_a(), 0x00);
    assert!(cpu.flags.z);
}

#[test]
fn test_add_a_half_carry() {
    let mut cpu = Cpu::new();
    let mut inter = create_test_interconnect();
    cpu.set_reg_a(0x0F);
    cpu.set_reg_b(0x01);

    cpu.execute_opcode(&mut inter, 0x80); // ADD A,B

    assert_eq!(cpu.get_reg_a(), 0x10);
    assert!(cpu.flags.h); // Half carry from bit 3 to 4
}

#[test]
fn test_add_a_carry() {
    let mut cpu = Cpu::new();
    let mut inter = create_test_interconnect();
    cpu.set_reg_a(0xFF);
    cpu.set_reg_b(0x01);

    cpu.execute_opcode(&mut inter, 0x80); // ADD A,B

    assert_eq!(cpu.get_reg_a(), 0x00);
    assert!(cpu.flags.z);
    assert!(cpu.flags.c);
    assert!(cpu.flags.h);
}

#[test]
fn test_add_a_d8() {
    let mut cpu = Cpu::new();
    let mut inter = create_test_interconnect_with_rom(&[(0x0100, 0x25)]);
    cpu.set_reg_a(0x10);
    cpu.reg_pc = 0x0100;

    let cycles = cpu.execute_opcode(&mut inter, 0xC6); // ADD A,d8

    assert_eq!(cycles, 8);
    assert_eq!(cpu.get_reg_a(), 0x35);
}

// ========== ADC Tests ==========

#[test]
fn test_adc_without_carry() {
    let mut cpu = Cpu::new();
    let mut inter = create_test_interconnect();
    cpu.set_reg_a(0x10);
    cpu.set_reg_b(0x20);
    cpu.flags.c = false;

    cpu.execute_opcode(&mut inter, 0x88); // ADC A,B

    assert_eq!(cpu.get_reg_a(), 0x30);
}

#[test]
fn test_adc_with_carry() {
    let mut cpu = Cpu::new();
    let mut inter = create_test_interconnect();
    cpu.set_reg_a(0x10);
    cpu.set_reg_b(0x20);
    cpu.flags.c = true;

    cpu.execute_opcode(&mut inter, 0x88); // ADC A,B

    assert_eq!(cpu.get_reg_a(), 0x31); // 0x10 + 0x20 + 1
}

// ========== SUB Tests ==========

#[test]
fn test_sub_a_b() {
    let mut cpu = Cpu::new();
    let mut inter = create_test_interconnect();
    cpu.set_reg_a(0x30);
    cpu.set_reg_b(0x10);

    let cycles = cpu.execute_opcode(&mut inter, 0x90); // SUB A,B

    assert_eq!(cycles, 4);
    assert_eq!(cpu.get_reg_a(), 0x20);
    assert!(!cpu.flags.z);
    assert!(cpu.flags.n); // N always set for SUB
    assert!(!cpu.flags.h);
    assert!(!cpu.flags.c);
}

#[test]
fn test_sub_a_borrow() {
    let mut cpu = Cpu::new();
    let mut inter = create_test_interconnect();
    cpu.set_reg_a(0x10);
    cpu.set_reg_b(0x20);

    cpu.execute_opcode(&mut inter, 0x90); // SUB A,B

    assert_eq!(cpu.get_reg_a(), 0xF0); // Wraps around
    assert!(cpu.flags.c); // Borrow occurred
}

#[test]
fn test_sub_half_borrow() {
    let mut cpu = Cpu::new();
    let mut inter = create_test_interconnect();
    cpu.set_reg_a(0x10);
    cpu.set_reg_b(0x01);

    cpu.execute_opcode(&mut inter, 0x90); // SUB A,B

    assert_eq!(cpu.get_reg_a(), 0x0F);
    assert!(cpu.flags.h); // Half borrow from bit 4 to 3
}

// ========== SBC Tests ==========

#[test]
fn test_sbc_without_carry() {
    let mut cpu = Cpu::new();
    let mut inter = create_test_interconnect();
    cpu.set_reg_a(0x30);
    cpu.set_reg_b(0x10);
    cpu.flags.c = false;

    cpu.execute_opcode(&mut inter, 0x98); // SBC A,B

    assert_eq!(cpu.get_reg_a(), 0x20);
}

#[test]
fn test_sbc_with_carry() {
    let mut cpu = Cpu::new();
    let mut inter = create_test_interconnect();
    cpu.set_reg_a(0x30);
    cpu.set_reg_b(0x10);
    cpu.flags.c = true;

    cpu.execute_opcode(&mut inter, 0x98); // SBC A,B

    assert_eq!(cpu.get_reg_a(), 0x1F); // 0x30 - 0x10 - 1
}

// ========== AND Tests ==========

#[test]
fn test_and_a_b() {
    let mut cpu = Cpu::new();
    let mut inter = create_test_interconnect();
    cpu.set_reg_a(0b11110000);
    cpu.set_reg_b(0b10101010);

    let cycles = cpu.execute_opcode(&mut inter, 0xA0); // AND A,B

    assert_eq!(cycles, 4);
    assert_eq!(cpu.get_reg_a(), 0b10100000);
    assert!(!cpu.flags.z);
    assert!(!cpu.flags.n);
    assert!(cpu.flags.h); // H always set for AND
    assert!(!cpu.flags.c);
}

#[test]
fn test_and_zero_result() {
    let mut cpu = Cpu::new();
    let mut inter = create_test_interconnect();
    cpu.set_reg_a(0b11110000);
    cpu.set_reg_b(0b00001111);

    cpu.execute_opcode(&mut inter, 0xA0); // AND A,B

    assert_eq!(cpu.get_reg_a(), 0x00);
    assert!(cpu.flags.z);
}

// ========== OR Tests ==========

#[test]
fn test_or_a_b() {
    let mut cpu = Cpu::new();
    let mut inter = create_test_interconnect();
    cpu.set_reg_a(0b11110000);
    cpu.set_reg_b(0b00001111);

    let cycles = cpu.execute_opcode(&mut inter, 0xB0); // OR A,B

    assert_eq!(cycles, 4);
    assert_eq!(cpu.get_reg_a(), 0xFF);
    assert!(!cpu.flags.z);
    assert!(!cpu.flags.n);
    assert!(!cpu.flags.h);
    assert!(!cpu.flags.c);
}

// ========== XOR Tests ==========

#[test]
fn test_xor_a_a() {
    let mut cpu = Cpu::new();
    let mut inter = create_test_interconnect();
    cpu.set_reg_a(0xFF);

    let cycles = cpu.execute_opcode(&mut inter, 0xAF); // XOR A,A

    assert_eq!(cycles, 4);
    assert_eq!(cpu.get_reg_a(), 0x00);
    assert!(cpu.flags.z);
    assert!(!cpu.flags.n);
    assert!(!cpu.flags.h);
    assert!(!cpu.flags.c);
}

#[test]
fn test_xor_a_b() {
    let mut cpu = Cpu::new();
    let mut inter = create_test_interconnect();
    cpu.set_reg_a(0b11110000);
    cpu.set_reg_b(0b10101010);

    cpu.execute_opcode(&mut inter, 0xA8); // XOR A,B

    assert_eq!(cpu.get_reg_a(), 0b01011010);
}

// ========== CP Tests ==========

#[test]
fn test_cp_equal() {
    let mut cpu = Cpu::new();
    let mut inter = create_test_interconnect();
    cpu.set_reg_a(0x42);
    cpu.set_reg_b(0x42);

    let cycles = cpu.execute_opcode(&mut inter, 0xB8); // CP A,B

    assert_eq!(cycles, 4);
    assert_eq!(cpu.get_reg_a(), 0x42); // A unchanged
    assert!(cpu.flags.z); // Equal
    assert!(cpu.flags.n);
    assert!(!cpu.flags.c);
}

#[test]
fn test_cp_less_than() {
    let mut cpu = Cpu::new();
    let mut inter = create_test_interconnect();
    cpu.set_reg_a(0x10);
    cpu.set_reg_b(0x20);

    cpu.execute_opcode(&mut inter, 0xB8); // CP A,B

    assert_eq!(cpu.get_reg_a(), 0x10); // A unchanged
    assert!(!cpu.flags.z);
    assert!(cpu.flags.c); // Borrow occurred (A < B)
}

#[test]
fn test_cp_greater_than() {
    let mut cpu = Cpu::new();
    let mut inter = create_test_interconnect();
    cpu.set_reg_a(0x30);
    cpu.set_reg_b(0x10);

    cpu.execute_opcode(&mut inter, 0xB8); // CP A,B

    assert!(!cpu.flags.z);
    assert!(!cpu.flags.c);
}

// ========== INC/DEC Tests ==========

#[test]
fn test_inc_b() {
    let mut cpu = Cpu::new();
    let mut inter = create_test_interconnect();
    cpu.set_reg_b(0x0F);

    let cycles = cpu.execute_opcode(&mut inter, 0x04); // INC B

    assert_eq!(cycles, 4);
    assert_eq!(cpu.get_reg_b(), 0x10);
    assert!(!cpu.flags.z);
    assert!(!cpu.flags.n);
    assert!(cpu.flags.h); // Half carry from 0x0F to 0x10
}

#[test]
fn test_inc_overflow() {
    let mut cpu = Cpu::new();
    let mut inter = create_test_interconnect();
    cpu.set_reg_b(0xFF);

    cpu.execute_opcode(&mut inter, 0x04); // INC B

    assert_eq!(cpu.get_reg_b(), 0x00);
    assert!(cpu.flags.z);
    assert!(cpu.flags.h);
}

#[test]
fn test_dec_b() {
    let mut cpu = Cpu::new();
    let mut inter = create_test_interconnect();
    cpu.set_reg_b(0x10);

    let cycles = cpu.execute_opcode(&mut inter, 0x05); // DEC B

    assert_eq!(cycles, 4);
    assert_eq!(cpu.get_reg_b(), 0x0F);
    assert!(!cpu.flags.z);
    assert!(cpu.flags.n);
    assert!(cpu.flags.h); // Half borrow from 0x10 to 0x0F
}

#[test]
fn test_dec_to_zero() {
    let mut cpu = Cpu::new();
    let mut inter = create_test_interconnect();
    cpu.set_reg_b(0x01);

    cpu.execute_opcode(&mut inter, 0x05); // DEC B

    assert_eq!(cpu.get_reg_b(), 0x00);
    assert!(cpu.flags.z);
}

#[test]
fn test_inc_bc() {
    let mut cpu = Cpu::new();
    let mut inter = create_test_interconnect();
    cpu.regs_bc.set(0x00FF);

    let cycles = cpu.execute_opcode(&mut inter, 0x03); // INC BC

    assert_eq!(cycles, 8);
    assert_eq!(cpu.regs_bc.get(), 0x0100);
    // 16-bit INC doesn't affect flags
}

#[test]
fn test_inc_hl_ref() {
    let mut cpu = Cpu::new();
    let mut inter = create_test_interconnect();
    cpu.regs_hl.set(0xC000);
    inter.write(0xC000, 0x0F);

    let cycles = cpu.execute_opcode(&mut inter, 0x34); // INC (HL)

    assert_eq!(cycles, 12);
    assert_eq!(inter.read(0xC000), 0x10);
    assert!(cpu.flags.h);
}

// ========== Rotate Tests ==========

#[test]
fn test_rlca() {
    let mut cpu = Cpu::new();
    let mut inter = create_test_interconnect();
    cpu.set_reg_a(0b10000001);

    let cycles = cpu.execute_opcode(&mut inter, 0x07); // RLCA

    assert_eq!(cycles, 4);
    assert_eq!(cpu.get_reg_a(), 0b00000011);
    assert!(!cpu.flags.z); // Z always 0 for RLCA
    assert!(!cpu.flags.n);
    assert!(!cpu.flags.h);
    assert!(cpu.flags.c); // Bit 7 was 1
}

#[test]
fn test_rrca() {
    let mut cpu = Cpu::new();
    let mut inter = create_test_interconnect();
    cpu.set_reg_a(0b00000011);

    let cycles = cpu.execute_opcode(&mut inter, 0x0F); // RRCA

    assert_eq!(cycles, 4);
    assert_eq!(cpu.get_reg_a(), 0b10000001);
    assert!(cpu.flags.c); // Bit 0 was 1
}

#[test]
fn test_rla() {
    let mut cpu = Cpu::new();
    let mut inter = create_test_interconnect();
    cpu.set_reg_a(0b10000000);
    cpu.flags.c = true;

    let cycles = cpu.execute_opcode(&mut inter, 0x17); // RLA

    assert_eq!(cycles, 4);
    assert_eq!(cpu.get_reg_a(), 0b00000001); // Carry rotated in
    assert!(cpu.flags.c); // Old bit 7
}

#[test]
fn test_rra() {
    let mut cpu = Cpu::new();
    let mut inter = create_test_interconnect();
    cpu.set_reg_a(0b00000001);
    cpu.flags.c = true;

    let cycles = cpu.execute_opcode(&mut inter, 0x1F); // RRA

    assert_eq!(cycles, 4);
    assert_eq!(cpu.get_reg_a(), 0b10000000); // Carry rotated in
    assert!(cpu.flags.c); // Old bit 0
}

// ========== Misc Tests ==========

#[test]
fn test_scf() {
    let mut cpu = Cpu::new();
    let mut inter = create_test_interconnect();
    cpu.flags.c = false;
    cpu.flags.n = true;
    cpu.flags.h = true;

    let cycles = cpu.execute_opcode(&mut inter, 0x37); // SCF

    assert_eq!(cycles, 4);
    assert!(cpu.flags.c);
    assert!(!cpu.flags.n);
    assert!(!cpu.flags.h);
}

#[test]
fn test_ccf() {
    let mut cpu = Cpu::new();
    let mut inter = create_test_interconnect();
    cpu.flags.c = true;

    cpu.execute_opcode(&mut inter, 0x3F); // CCF

    assert!(!cpu.flags.c);

    cpu.execute_opcode(&mut inter, 0x3F); // CCF again

    assert!(cpu.flags.c);
}

#[test]
fn test_ld_sp_hl() {
    let mut cpu = Cpu::new();
    let mut inter = create_test_interconnect();
    cpu.regs_hl.set(0xDFFF);

    let cycles = cpu.execute_opcode(&mut inter, 0xF9); // LD SP,HL

    assert_eq!(cycles, 8);
    assert_eq!(cpu.reg_sp, 0xDFFF);
}

#[test]
fn test_add_hl_bc() {
    let mut cpu = Cpu::new();
    let mut inter = create_test_interconnect();
    cpu.regs_hl.set(0x0FFF);
    cpu.regs_bc.set(0x0001);

    let cycles = cpu.execute_opcode(&mut inter, 0x09); // ADD HL,BC

    assert_eq!(cycles, 8);
    assert_eq!(cpu.regs_hl.get(), 0x1000);
    assert!(!cpu.flags.n);
    assert!(cpu.flags.h); // Half carry from bit 11 to 12
    assert!(!cpu.flags.c);
}

#[test]
fn test_add_hl_bc_carry() {
    let mut cpu = Cpu::new();
    let mut inter = create_test_interconnect();
    cpu.regs_hl.set(0xFFFF);
    cpu.regs_bc.set(0x0001);

    cpu.execute_opcode(&mut inter, 0x09); // ADD HL,BC

    assert_eq!(cpu.regs_hl.get(), 0x0000);
    assert!(cpu.flags.c);
}

#[test]
fn test_ld_a16_sp() {
    let mut cpu = Cpu::new();
    let mut inter = create_test_interconnect_with_rom(&[
        (0x0100, 0x00),
        (0x0101, 0xC0),
    ]);
    cpu.reg_sp = 0x1234;
    cpu.reg_pc = 0x0100;

    let cycles = cpu.execute_opcode(&mut inter, 0x08); // LD (a16),SP

    assert_eq!(cycles, 20);
    assert_eq!(inter.read(0xC000), 0x34); // Low byte
    assert_eq!(inter.read(0xC001), 0x12); // High byte
}

// ========== CB-Prefix Opcode Tests ==========

#[test]
fn test_cb_rlc_b() {
    let mut cpu = Cpu::new();
    let mut inter = create_test_interconnect_with_rom(&[(0x0100, 0x00)]); // CB opcode for RLC B
    cpu.reg_pc = 0x0100;
    cpu.set_reg_b(0x85); // 1000_0101

    let cycles = cpu.execute_opcode(&mut inter, 0xCB); // PREFIX CB

    assert_eq!(cycles, 8);
    assert_eq!(cpu.get_reg_b(), 0x0B); // 0000_1011 (rotated left, bit 7 -> bit 0)
    assert!(cpu.flags.c); // Old bit 7 was 1
    assert!(!cpu.flags.z);
    assert!(!cpu.flags.n);
    assert!(!cpu.flags.h);
}

#[test]
fn test_cb_rlc_zero() {
    let mut cpu = Cpu::new();
    let mut inter = create_test_interconnect_with_rom(&[(0x0100, 0x00)]); // RLC B
    cpu.reg_pc = 0x0100;
    cpu.set_reg_b(0x00);

    cpu.execute_opcode(&mut inter, 0xCB);

    assert_eq!(cpu.get_reg_b(), 0x00);
    assert!(cpu.flags.z);
    assert!(!cpu.flags.c);
}

#[test]
fn test_cb_rrc_a() {
    let mut cpu = Cpu::new();
    let mut inter = create_test_interconnect_with_rom(&[(0x0100, 0x0F)]); // RRC A
    cpu.reg_pc = 0x0100;
    cpu.set_reg_a(0x81); // 1000_0001

    let cycles = cpu.execute_opcode(&mut inter, 0xCB);

    assert_eq!(cycles, 8);
    assert_eq!(cpu.get_reg_a(), 0xC0); // 1100_0000 (rotated right, bit 0 -> bit 7)
    assert!(cpu.flags.c); // Old bit 0 was 1
}

#[test]
fn test_cb_rl_c() {
    let mut cpu = Cpu::new();
    let mut inter = create_test_interconnect_with_rom(&[(0x0100, 0x11)]); // RL C
    cpu.reg_pc = 0x0100;
    cpu.set_reg_c(0x80); // 1000_0000
    cpu.flags.c = true; // Carry set

    let cycles = cpu.execute_opcode(&mut inter, 0xCB);

    assert_eq!(cycles, 8);
    assert_eq!(cpu.get_reg_c(), 0x01); // 0000_0001 (shifted left, old carry -> bit 0)
    assert!(cpu.flags.c); // Old bit 7 was 1
}

#[test]
fn test_cb_rr_d() {
    let mut cpu = Cpu::new();
    let mut inter = create_test_interconnect_with_rom(&[(0x0100, 0x1A)]); // RR D
    cpu.reg_pc = 0x0100;
    cpu.set_reg_d(0x01); // 0000_0001
    cpu.flags.c = true; // Carry set

    let cycles = cpu.execute_opcode(&mut inter, 0xCB);

    assert_eq!(cycles, 8);
    assert_eq!(cpu.get_reg_d(), 0x80); // 1000_0000 (shifted right, old carry -> bit 7)
    assert!(cpu.flags.c); // Old bit 0 was 1
}

#[test]
fn test_cb_sla_e() {
    let mut cpu = Cpu::new();
    let mut inter = create_test_interconnect_with_rom(&[(0x0100, 0x23)]); // SLA E
    cpu.reg_pc = 0x0100;
    cpu.set_reg_e(0xC1); // 1100_0001

    let cycles = cpu.execute_opcode(&mut inter, 0xCB);

    assert_eq!(cycles, 8);
    assert_eq!(cpu.get_reg_e(), 0x82); // 1000_0010 (shifted left, bit 0 = 0)
    assert!(cpu.flags.c); // Old bit 7 was 1
}

#[test]
fn test_cb_sra_h() {
    let mut cpu = Cpu::new();
    let mut inter = create_test_interconnect_with_rom(&[(0x0100, 0x2C)]); // SRA H
    cpu.reg_pc = 0x0100;
    cpu.set_reg_h(0x81); // 1000_0001

    let cycles = cpu.execute_opcode(&mut inter, 0xCB);

    assert_eq!(cycles, 8);
    assert_eq!(cpu.get_reg_h(), 0xC0); // 1100_0000 (shifted right, bit 7 preserved)
    assert!(cpu.flags.c); // Old bit 0 was 1
}

#[test]
fn test_cb_swap_l() {
    let mut cpu = Cpu::new();
    let mut inter = create_test_interconnect_with_rom(&[(0x0100, 0x35)]); // SWAP L
    cpu.reg_pc = 0x0100;
    cpu.set_reg_l(0x12); // 0001_0010

    let cycles = cpu.execute_opcode(&mut inter, 0xCB);

    assert_eq!(cycles, 8);
    assert_eq!(cpu.get_reg_l(), 0x21); // 0010_0001 (nibbles swapped)
    assert!(!cpu.flags.c);
    assert!(!cpu.flags.z);
}

#[test]
fn test_cb_swap_zero() {
    let mut cpu = Cpu::new();
    let mut inter = create_test_interconnect_with_rom(&[(0x0100, 0x37)]); // SWAP A
    cpu.reg_pc = 0x0100;
    cpu.set_reg_a(0x00);

    cpu.execute_opcode(&mut inter, 0xCB);

    assert_eq!(cpu.get_reg_a(), 0x00);
    assert!(cpu.flags.z);
}

#[test]
fn test_cb_srl_a() {
    let mut cpu = Cpu::new();
    let mut inter = create_test_interconnect_with_rom(&[(0x0100, 0x3F)]); // SRL A
    cpu.reg_pc = 0x0100;
    cpu.set_reg_a(0x81); // 1000_0001

    let cycles = cpu.execute_opcode(&mut inter, 0xCB);

    assert_eq!(cycles, 8);
    assert_eq!(cpu.get_reg_a(), 0x40); // 0100_0000 (shifted right, bit 7 = 0)
    assert!(cpu.flags.c); // Old bit 0 was 1
}

#[test]
fn test_cb_bit_0_b() {
    let mut cpu = Cpu::new();
    let mut inter = create_test_interconnect_with_rom(&[(0x0100, 0x40)]); // BIT 0,B
    cpu.reg_pc = 0x0100;
    cpu.set_reg_b(0xFE); // 1111_1110 (bit 0 is 0)
    cpu.flags.c = true; // Should be unchanged

    let cycles = cpu.execute_opcode(&mut inter, 0xCB);

    assert_eq!(cycles, 8);
    assert!(cpu.flags.z); // Bit 0 is 0, so Z is set
    assert!(!cpu.flags.n);
    assert!(cpu.flags.h);
    assert!(cpu.flags.c); // Unchanged
}

#[test]
fn test_cb_bit_7_a() {
    let mut cpu = Cpu::new();
    let mut inter = create_test_interconnect_with_rom(&[(0x0100, 0x7F)]); // BIT 7,A
    cpu.reg_pc = 0x0100;
    cpu.set_reg_a(0x80); // 1000_0000 (bit 7 is 1)

    let cycles = cpu.execute_opcode(&mut inter, 0xCB);

    assert_eq!(cycles, 8);
    assert!(!cpu.flags.z); // Bit 7 is 1, so Z is clear
}

#[test]
fn test_cb_res_3_c() {
    let mut cpu = Cpu::new();
    let mut inter = create_test_interconnect_with_rom(&[(0x0100, 0x99)]); // RES 3,C
    cpu.reg_pc = 0x0100;
    cpu.set_reg_c(0xFF); // All bits set

    let cycles = cpu.execute_opcode(&mut inter, 0xCB);

    assert_eq!(cycles, 8);
    assert_eq!(cpu.get_reg_c(), 0xF7); // 1111_0111 (bit 3 cleared)
}

#[test]
fn test_cb_set_5_d() {
    let mut cpu = Cpu::new();
    let mut inter = create_test_interconnect_with_rom(&[(0x0100, 0xEA)]); // SET 5,D
    cpu.reg_pc = 0x0100;
    cpu.set_reg_d(0x00); // All bits clear

    let cycles = cpu.execute_opcode(&mut inter, 0xCB);

    assert_eq!(cycles, 8);
    assert_eq!(cpu.get_reg_d(), 0x20); // 0010_0000 (bit 5 set)
}

#[test]
fn test_cb_rlc_hl_ref() {
    let mut cpu = Cpu::new();
    let mut inter = create_test_interconnect_with_rom(&[(0x0100, 0x06)]); // RLC (HL)
    cpu.reg_pc = 0x0100;
    cpu.regs_hl.set(0xC000); // Point to WRAM
    inter.write(0xC000, 0x85); // 1000_0101

    let cycles = cpu.execute_opcode(&mut inter, 0xCB);

    assert_eq!(cycles, 16); // (HL) takes 16 cycles
    assert_eq!(inter.read(0xC000), 0x0B); // 0000_1011
    assert!(cpu.flags.c);
}

#[test]
fn test_cb_bit_hl_ref() {
    let mut cpu = Cpu::new();
    let mut inter = create_test_interconnect_with_rom(&[(0x0100, 0x46)]); // BIT 0,(HL)
    cpu.reg_pc = 0x0100;
    cpu.regs_hl.set(0xC000);
    inter.write(0xC000, 0x01); // Bit 0 is set

    let cycles = cpu.execute_opcode(&mut inter, 0xCB);

    assert_eq!(cycles, 12); // BIT (HL) takes 12 cycles
    assert!(!cpu.flags.z); // Bit 0 is 1
}

#[test]
fn test_cb_set_hl_ref() {
    let mut cpu = Cpu::new();
    let mut inter = create_test_interconnect_with_rom(&[(0x0100, 0xFE)]); // SET 7,(HL)
    cpu.reg_pc = 0x0100;
    cpu.regs_hl.set(0xC000);
    inter.write(0xC000, 0x00);

    let cycles = cpu.execute_opcode(&mut inter, 0xCB);

    assert_eq!(cycles, 16);
    assert_eq!(inter.read(0xC000), 0x80); // Bit 7 set
}
