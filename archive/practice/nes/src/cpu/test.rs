use super::*;

#[test]
fn test_0xa9_sets_zero_flag() {
    let mut cpu = CPU::new();
    cpu.load_and_run(vec![0xA9, 0x00, 0x00]);
    assert!(cpu.status2.get(StatusFlag::Zero));
}

#[test]
fn test_0xaa_sets_x() {
    let mut cpu = CPU::new();
    cpu.load_and_run(vec![0xA9, 0xFF, 0xAA, 0x00]);
    assert_eq!(cpu.register_a, 0xFF);
    assert_eq!(cpu.register_x, 0xFF);
}

#[test]
fn test_0xaa_sets_zero_flag() {
    let mut cpu = CPU::new();
    cpu.load_and_run(vec![0xA9, 0x00, 0xAA, 0x00]);
    assert!(cpu.status2.get(StatusFlag::Zero));
}

#[test]
fn test_5_ops_working_together() {
    let mut cpu = CPU::new();
    cpu.load_and_run(vec![0xa9, 0xc0, 0xaa, 0xe8, 0x00]);

    assert_eq!(cpu.register_x, 0xc1)
}

#[test]
fn test_inx_overflow() {
    let mut cpu = CPU::new();
    cpu.register_x = 0xff;
    // Do not call `load_and_run` to avoid resetting `register_x`.
    cpu.load(vec![0xe8, 0xe8, 0x00]);
    cpu.program_counter = 0x8000;
    cpu.run();

    assert_eq!(cpu.register_x, 1)
}

#[test]
fn test_0xa5() {
    // Load A from Zero Page.
    let mut cpu = CPU::new();
    cpu.mem_write(0x01, 123);
    cpu.load_and_run(vec![0xA5, 0x01]);
    assert_eq!(cpu.register_a, 123);
}

#[test]
fn test_0xb5() {
    // Load A from Zero Page,X.
    let mut cpu = CPU::new();
    cpu.load(vec![0xB5, 0x01]);
    cpu.program_counter = cpu.mem_read_u16(0xFFFC);
    cpu.mem_write(0x03, 123);
    cpu.register_x = 2;
    cpu.run();
    assert_eq!(cpu.register_a, 123);
}

#[test]
fn test_0xad() {
    // Load A from Absolute.
    let mut cpu = CPU::new();
    cpu.load(vec![0xAD, 0x01, 0x00]);
    cpu.program_counter = cpu.mem_read_u16(0xFFFC);
    cpu.mem_write(0x01, 123);
    cpu.run();
    assert_eq!(cpu.register_a, 123);
}

#[test]
fn test_0x85() {
    // Store A from Zero Page.
    let mut cpu = CPU::new();
    cpu.load(vec![0x85, 0x01]);
    cpu.register_a = 123;
    cpu.program_counter = cpu.mem_read_u16(0xFFFC);
    cpu.run();
    assert_eq!(cpu.mem_read(1), 123);
}

#[test]
fn test_0x95() {
    // Store A from Zero Page, X.
    let mut cpu = CPU::new();
    cpu.load(vec![0x95, 0x01]);
    cpu.register_a = 123;
    cpu.register_x = 2;
    cpu.program_counter = cpu.mem_read_u16(0xFFFC);
    cpu.run();
    assert_eq!(cpu.mem_read(3), 123);
}

#[test]
fn test_0x8d() {
    // Store A from Absolute.
    let mut cpu = CPU::new();
    cpu.load(vec![0x8d, 0x01, 0x00]);
    cpu.register_a = 123;
    cpu.program_counter = cpu.mem_read_u16(0xFFFC);
    cpu.run();
    assert_eq!(cpu.mem_read(1), 123);
}

#[test]
fn test_0x9d() {
    // Store A from Absolute, X.
    let mut cpu = CPU::new();
    cpu.load(vec![0x9d, 0x01, 0x00]);
    cpu.register_a = 123;
    cpu.register_x = 2;
    cpu.program_counter = cpu.mem_read_u16(0xFFFC);
    cpu.run();
    assert_eq!(cpu.mem_read(3), 123);
}

#[test]
fn test_0x99() {
    // Store A from Absolute, Y.
    let mut cpu = CPU::new();
    cpu.load(vec![0x99, 0x01, 0x00]);
    cpu.register_a = 123;
    cpu.register_y = 2;
    cpu.program_counter = cpu.mem_read_u16(0xFFFC);
    cpu.run();
    assert_eq!(cpu.mem_read(3), 123);
}

#[test]
fn test_jmp_absolute() {
    let mut cpu = CPU::new();
    // Program is loaded at address 0x8000. Jump to 0x8004
    cpu.load(vec![
        0x4C, 0x04, 0x80, // Jump to 0x8004.
        0xFF, // Invalid instruction.
        0xa9, // LDA Immediate.
        0x01,
    ]);
    cpu.program_counter = cpu.mem_read_u16(0xFFFC);
    cpu.run();
    assert_eq!(cpu.register_a, 0x01);
}

#[test]
fn test_jmp_indirect() {
    let mut cpu = CPU::new();
    // Program is loaded at address 0x8000.
    cpu.load(vec![
        0x6C, 0x03, 0x80, // Jump to address stored at 0x8003.
        0x06, 0x80, // Address 0x8006
        0xFF, // Invalid instruction.
        0xa9, // LDA Immediate.
        0x01,
    ]);
    cpu.program_counter = cpu.mem_read_u16(0xFFFC);
    cpu.run();
    assert_eq!(cpu.register_a, 0x01);
}

// TODO: test LDA with indirect addressing mode.

#[test]
fn test_0x69_adc_immediate() {
    let mut cpu = CPU::new();
    // Adds 1.
    {
        cpu.reset();
        cpu.load(vec![
            0x69, 0x01, // Add 1.
        ]);
        cpu.program_counter = cpu.mem_read_u16(0xFFFC);
        cpu.run();
        assert_eq!(cpu.register_a, 0x01);
        assert_eq!(cpu.get_carry(), 0);
    }
    // Sets carry bit on overflow.
    {
        cpu.reset();
        cpu.load(vec![
            0x69, 0xFF, // Add 255.
            0x69, 0x01, // Add 1.
        ]);
        cpu.program_counter = cpu.mem_read_u16(0xFFFC);
        cpu.run();
        assert_eq!(cpu.register_a, 0x00);
        assert_eq!(cpu.get_carry(), 1);
    }
    // Uses carry bit after overflow.
    {
        cpu.reset();
        cpu.load(vec![
            0x69, 0xFF, // Add 255.
            0x69, 0x01, // Add 1.
            0x69, 0x00, // Add 0.
        ]);
        cpu.program_counter = cpu.mem_read_u16(0xFFFC);
        cpu.run();
        assert_eq!(cpu.register_a, 0x01);
        assert_eq!(cpu.get_carry(), 0);
    }
}

#[test]
fn test_0x65_adc_zeropage() {
    let mut cpu = CPU::new();
    // Adds 123.
    {
        cpu.reset();
        cpu.load(vec![
            0x65, 0x01, // Add from memory location 1.
        ]);
        cpu.mem_write(0x01, 123);
        cpu.program_counter = cpu.mem_read_u16(0xFFFC);
        cpu.run();
        assert_eq!(cpu.register_a, 123);
        assert_eq!(cpu.get_carry(), 0);
    }
}

#[test]
fn test_0x75_adc_zeropage_x() {
    let mut cpu = CPU::new();
    // Adds 123.
    {
        cpu.reset();
        cpu.load(vec![
            0x75, 0x01, // Add from memory location 0x01 + x.
        ]);
        cpu.register_x = 2;
        cpu.mem_write(0x03, 123);
        cpu.program_counter = cpu.mem_read_u16(0xFFFC);
        cpu.run();
        assert_eq!(cpu.register_a, 123);
        assert_eq!(cpu.get_carry(), 0);
    }
}

#[test]
fn test_0x6d_adc_absolute() {
    let mut cpu = CPU::new();
    // Adds 123.
    {
        cpu.reset();
        cpu.load(vec![
            0x6D, 0x03, 0x00, // Add from memory location 0x03.
        ]);
        cpu.mem_write(0x03, 123);
        cpu.program_counter = cpu.mem_read_u16(0xFFFC);
        cpu.run();
        assert_eq!(cpu.register_a, 123);
        assert_eq!(cpu.get_carry(), 0);
    }
}

#[test]
fn test_0x7d_adc_absolute_x() {
    let mut cpu = CPU::new();
    // Adds 123.
    {
        cpu.reset();
        cpu.load(vec![
            0x7d, 0x01, 0x00, // Add from memory location 0x01 + x.
        ]);
        cpu.register_x = 2;
        cpu.mem_write(0x03, 123);
        cpu.program_counter = cpu.mem_read_u16(0xFFFC);
        cpu.run();
        assert_eq!(cpu.register_a, 123);
        assert_eq!(cpu.get_carry(), 0);
    }
}

#[test]
fn test_0x79_adc_absolute_y() {
    let mut cpu = CPU::new();
    // Adds 123.
    {
        cpu.reset();
        cpu.load(vec![
            0x79, 0x01, 0x00, // Add from memory location 0x01 + y.
        ]);
        cpu.register_y = 2;
        cpu.mem_write(0x03, 123);
        cpu.program_counter = cpu.mem_read_u16(0xFFFC);
        cpu.run();
        assert_eq!(cpu.register_a, 123);
        assert_eq!(cpu.get_carry(), 0);
    }
}

#[test]
fn test_0x61_adc_indirect_x() {
    let mut cpu = CPU::new();
    // Adds 123.
    {
        cpu.reset();
        cpu.load(vec![
            0x61, 0x01, // Add from memory location *(0x0001 + x).
        ]);
        cpu.register_x = 2;
        cpu.mem_write_u16(0x0003, 0x0005); // Value at 0x0003 is 0x0005
        cpu.mem_write(0x0005, 123);
        cpu.program_counter = cpu.mem_read_u16(0xFFFC);
        cpu.run();
        assert_eq!(cpu.register_a, 123);
        assert_eq!(cpu.get_carry(), 0);
    }
}

#[test]
fn test_0x71_adc_indirect_y() {
    let mut cpu = CPU::new();
    // Adds 123.
    {
        cpu.reset();
        cpu.load(vec![
            0x71, 0x01, // Add from memory location *(0x0001) + y.
        ]);
        cpu.register_y = 2;
        cpu.mem_write_u16(0x0001, 0x0003);
        cpu.mem_write(0x0005, 123);
        cpu.program_counter = cpu.mem_read_u16(0xFFFC);
        cpu.run();
        assert_eq!(cpu.register_a, 123);
        assert_eq!(cpu.get_carry(), 0);
    }
}

#[test]
fn test_0x29_and_immediate() {
    let mut cpu = CPU::new();
    // AND 1001 and 1101
    {
        cpu.reset();
        cpu.load(vec![0x29, 0b1101]);
        cpu.register_a = 0b1001;
        cpu.program_counter = cpu.mem_read_u16(0xFFFC);
        cpu.run();
        assert_eq!(cpu.register_a, 0b1001);
    }

    // Check that negative flag is set.
    {
        cpu.reset();
        cpu.load(vec![0x29, 0b1000_0000]);
        cpu.register_a = 0b1000_0000;
        cpu.program_counter = cpu.mem_read_u16(0xFFFC);
        cpu.run();
        assert_eq!(cpu.register_a, 0b1000_0000);
        // Check that negative flag is set.
        assert!(cpu.status2.get(StatusFlag::Negative));
    }

    // Check that zero flag is set.
    {
        cpu.reset();
        cpu.load(vec![0x29, 0b1]);
        cpu.register_a = 0b0;
        cpu.program_counter = cpu.mem_read_u16(0xFFFC);
        cpu.run();
        assert_eq!(cpu.register_a, 0b0);
        // Check that zero flag is set.
        assert!(cpu.status2.get(StatusFlag::Zero));
    }
}

// AND operations with other AddressingMode values are not tested.
// Assuming testing ADC with Immediate AddressingMode is sufficient.

#[test]
fn test_0x0a_asl_immediate() {
    let mut cpu = CPU::new();
    {
        cpu.reset();
        cpu.load(vec![0x0A]);
        cpu.register_a = 0b0000_0001;
        cpu.program_counter = cpu.mem_read_u16(0xFFFC);
        cpu.run();
        assert_eq!(cpu.register_a, 0b0000_0010);
        assert!(!cpu.status2.get(StatusFlag::Zero));
        assert!(!cpu.status2.get(StatusFlag::Carry));
        assert!(!cpu.status2.get(StatusFlag::Negative));
    }

    // Check that carry flag is set.
    {
        cpu.reset();
        cpu.load(vec![0x0A]);
        cpu.register_a = 0b1000_0001;
        cpu.program_counter = cpu.mem_read_u16(0xFFFC);
        cpu.run();
        assert_eq!(cpu.register_a, 0b0000_0010);
        assert!(!cpu.status2.get(StatusFlag::Zero));
        assert!(cpu.status2.get(StatusFlag::Carry));
        assert!(!cpu.status2.get(StatusFlag::Negative));
    }

    // Check that negative flag is set.
    {
        cpu.reset();
        cpu.load(vec![0x0A]);
        cpu.register_a = 0b0100_0001;
        cpu.program_counter = cpu.mem_read_u16(0xFFFC);
        cpu.run();
        assert_eq!(cpu.register_a, 0b1000_0010);
        assert!(!cpu.status2.get(StatusFlag::Zero));
        assert!(!cpu.status2.get(StatusFlag::Carry));
        assert!(cpu.status2.get(StatusFlag::Negative));
    }

    // Check that zero flag is set.
    {
        cpu.reset();
        cpu.load(vec![0x0A]);
        cpu.register_a = 0b0000_0000;
        cpu.program_counter = cpu.mem_read_u16(0xFFFC);
        cpu.run();
        assert_eq!(cpu.register_a, 0b0000_0000);
        assert!(cpu.status2.get(StatusFlag::Zero));
        assert!(!cpu.status2.get(StatusFlag::Carry));
        assert!(!cpu.status2.get(StatusFlag::Negative));
    }
}

#[test]
fn test_0x90_bcc() {
    let mut cpu = CPU::new();
    {
        cpu.reset();
        cpu.load(vec![
            0x90, 0x01, // Jump one instruction ahead.
            0xFF, // Invalid instruction.
            0xA9, 123, 0x00, // LDA value 123.
        ]);
        cpu.program_counter = cpu.mem_read_u16(0xFFFC);
        cpu.run();
        assert_eq!(cpu.register_a, 123);
    }
    // Test jumping negative.
    {
        cpu.reset();
        cpu.load(vec![
            0x90,
            0x04, // Jump four instructions ahead.
            0xFF, // Invalid instruction.
            0xA9,
            123,
            0x00, // LDA value 123.
            0x90,
            (-5i8 as u8), // Jump five instructions behind.
        ]);
        cpu.program_counter = cpu.mem_read_u16(0xFFFC);
        cpu.run();
        assert_eq!(cpu.register_a, 123);
    }
}

#[test]
fn test_0xb0_bcs() {
    let mut cpu = CPU::new();
    {
        cpu.reset();
        cpu.load(vec![
            0xB0, 0x01, // Jump one instruction ahead.
            0xFF, // Invalid instruction.
            0xA9, 123, 0x00, // LDA value 123.
        ]);
        cpu.program_counter = cpu.mem_read_u16(0xFFFC);
        cpu.status2.set(StatusFlag::Carry, true);
        cpu.run();
        assert_eq!(cpu.register_a, 123);
    }
    // Does not jump if carry not set.
    {
        cpu.reset();
        cpu.load(vec![
            0xB0, 0x01, // Jump one instruction ahead.
            0x00, // Break.
            0xA9, 123, 0x00, // LDA value 123.
        ]);
        cpu.program_counter = cpu.mem_read_u16(0xFFFC);
        cpu.run();
        assert_eq!(cpu.register_a, 0);
    }
}

#[test]
fn test_0xfo_beq() {
    let mut cpu = CPU::new();
    {
        cpu.reset();
        cpu.load(vec![
            0xF0, 0x01, // Jump one instruction ahead.
            0xFF, // Invalid instruction.
            0xA9, 123, 0x00, // LDA value 123.
        ]);
        cpu.program_counter = cpu.mem_read_u16(0xFFFC);
        cpu.status2.set(StatusFlag::Zero, true);
        cpu.run();
        assert_eq!(cpu.register_a, 123);
    }
    // Does not jump if zero not set.
    {
        cpu.reset();
        cpu.load(vec![
            0xF0, 0x01, // Jump one instruction ahead.
            0x00, // Break.
            0xA9, 123, 0x00, // LDA value 123.
        ]);
        cpu.program_counter = cpu.mem_read_u16(0xFFFC);
        cpu.run();
        assert_eq!(cpu.register_a, 0);
    }
}

#[test]
fn test_0xfo_bne() {
    let mut cpu = CPU::new();
    {
        cpu.reset();
        cpu.load(vec![
            0xD0, 0x01, // Jump one instruction ahead.
            0xFF, // Invalid instruction.
            0xA9, 123, 0x00, // LDA value 123.
        ]);
        cpu.program_counter = cpu.mem_read_u16(0xFFFC);
        cpu.run();
        assert_eq!(cpu.register_a, 123);
    }
    // Does not jump if zero is set.
    {
        cpu.reset();
        cpu.load(vec![
            0xD0, 0x01, // Jump one instruction ahead.
            0x00, // Break.
            0xA9, 123, 0x00, // LDA value 123.
        ]);
        cpu.program_counter = cpu.mem_read_u16(0xFFFC);
        cpu.status2.set(StatusFlag::Zero, true);
        cpu.run();
        assert_eq!(cpu.register_a, 0);
    }
}

#[test]
fn test_0x24_bit_zeropage() {
    let mut cpu = CPU::new();

    // Test with ZeroPage. Result is 1.
    {
        cpu.reset();
        cpu.load(vec![
            0xA9, 0xF0, // LDA value 0xF0.
            0x85, 0x01, // STA to address 0x01.
            0xA9, 0xFF, // LDA value 0xFF.
            0x24, 0x01, // BIT with address 0x01.
        ]);
        cpu.program_counter = cpu.mem_read_u16(0xFFFC);
        cpu.run();
        assert_eq!(cpu.register_a, 0xFF);
        assert!(!cpu.status2.get(StatusFlag::Zero));
        assert!(cpu.status2.get(StatusFlag::Negative));
        assert!(cpu.status2.get(StatusFlag::Overflow));
    }

    // Test with ZeroPage. Result is 0.
    {
        cpu.reset();
        cpu.load(vec![
            0xA9, 0xF0, // LDA value 0xF0.
            0x85, 0x01, // STA to address 0x01.
            0xA9, 0x0F, // LDA value 0x0F.
            0x24, 0x01, // BIT with address 0x01.
        ]);
        cpu.program_counter = cpu.mem_read_u16(0xFFFC);
        cpu.run();
        assert_eq!(cpu.register_a, 0x0F);
        assert!(cpu.status2.get(StatusFlag::Zero));
        assert!(cpu.status2.get(StatusFlag::Negative));
        assert!(cpu.status2.get(StatusFlag::Overflow));
    }
}

#[test]
fn test_0x2c_bit_absolute() {
    let mut cpu = CPU::new();

    // Test with ZeroPage. Result is 1.
    {
        cpu.reset();
        cpu.load(vec![
            0xA9, 0xF0, // LDA value 0xF0.
            0x85, 0x01, // STA to address 0x01.
            0xA9, 0xFF, // LDA value 0xFF.
            0x2c, 0x01, 0x00, // BIT with address 0x01.
        ]);
        cpu.program_counter = cpu.mem_read_u16(0xFFFC);
        cpu.run();
        assert_eq!(cpu.register_a, 0xFF);
        assert!(!cpu.status2.get(StatusFlag::Zero));
        assert!(cpu.status2.get(StatusFlag::Negative));
        assert!(cpu.status2.get(StatusFlag::Overflow));
    }
}

#[test]
fn test_0x30_bmi() {
    let mut cpu = CPU::new();
    {
        cpu.reset();
        cpu.load(vec![
            0x30, 0x01, // Jump one instruction ahead.
            0xFF, // Invalid instruction.
            0xA9, 123, 0x00, // LDA value 123.
        ]);
        cpu.program_counter = cpu.mem_read_u16(0xFFFC);
        cpu.status2.set(StatusFlag::Negative, true);
        cpu.run();
        assert_eq!(cpu.register_a, 123);
    }
    // Does not jump if negative not set.
    {
        cpu.reset();
        cpu.load(vec![
            0x30, 0x01, // Jump one instruction ahead.
            0x00, // Break.
            0xA9, 123, 0x00, // LDA value 123.
        ]);
        cpu.program_counter = cpu.mem_read_u16(0xFFFC);
        cpu.run();
        assert_eq!(cpu.register_a, 0);
    }
}

#[test]
fn test_0x10_bpl() {
    let mut cpu = CPU::new();
    {
        cpu.reset();
        cpu.load(vec![
            0x10, 0x01, // Jump one instruction ahead.
            0xFF, // Invalid instruction.
            0xA9, 123, 0x00, // LDA value 123.
        ]);
        cpu.program_counter = cpu.mem_read_u16(0xFFFC);
        cpu.status2.reset();
        cpu.run();
        assert_eq!(cpu.register_a, 123);
    }
    // Does not jump if negative set.
    {
        cpu.reset();
        cpu.load(vec![
            0x10, 0x01, // Jump one instruction ahead.
            0x00, // Break.
            0xA9, 123, 0x00, // LDA value 123.
        ]);
        cpu.program_counter = cpu.mem_read_u16(0xFFFC);
        cpu.status2.set(StatusFlag::Negative, true);
        cpu.run();
        assert_eq!(cpu.register_a, 0);
    }
}

#[test]
fn test_0x50_bvc() {
    let mut cpu = CPU::new();
    {
        cpu.reset();
        cpu.load(vec![
            0x50, 0x01, // Jump one instruction ahead.
            0xFF, // Invalid instruction.
            0xA9, 123, 0x00, // LDA value 123.
        ]);
        cpu.program_counter = cpu.mem_read_u16(0xFFFC);
        cpu.status2.reset();
        cpu.run();
        assert_eq!(cpu.register_a, 123);
    }
    // Does not jump if overflow set.
    {
        cpu.reset();
        cpu.load(vec![
            0x50, 0x01, // Jump one instruction ahead.
            0x00, // Break.
            0xA9, 123, 0x00, // LDA value 123.
        ]);
        cpu.program_counter = cpu.mem_read_u16(0xFFFC);
        cpu.status2.set(StatusFlag::Overflow, true);
        cpu.run();
        assert_eq!(cpu.register_a, 0);
    }
}

#[test]
fn test_0x70_bvs() {
    let mut cpu = CPU::new();
    {
        cpu.reset();
        cpu.load(vec![
            0x70, 0x01, // Jump one instruction ahead.
            0xFF, // Invalid instruction.
            0xA9, 123, 0x00, // LDA value 123.
        ]);
        cpu.program_counter = cpu.mem_read_u16(0xFFFC);
        cpu.status2.set(StatusFlag::Overflow, true);
        cpu.run();
        assert_eq!(cpu.register_a, 123);
    }
    // Does not jump if overflow not set.
    {
        cpu.reset();
        cpu.load(vec![
            0x70, 0x01, // Jump one instruction ahead.
            0x00, // Break.
            0xA9, 123, 0x00, // LDA value 123.
        ]);
        cpu.program_counter = cpu.mem_read_u16(0xFFFC);
        cpu.status2.reset();
        cpu.run();
        assert_eq!(cpu.register_a, 0);
    }
}

#[test]
fn test_0x18_clc() {
    let mut cpu = CPU::new();

    cpu.reset();
    cpu.load(vec![0x18, 0x00]);
    cpu.program_counter = cpu.mem_read_u16(0xFFFC);
    cpu.status2.set_all(0xFF);
    cpu.run();
    assert_eq!(cpu.status2.get_all(), 0xFF & !(StatusFlag::Carry as u8));
}

#[test]
fn test_0xd8_cld() {
    let mut cpu = CPU::new();

    cpu.reset();
    cpu.load(vec![0xd8, 0x00]);
    cpu.program_counter = cpu.mem_read_u16(0xFFFC);
    cpu.status2.set_all(0xFF);
    cpu.run();
    assert_eq!(cpu.status2.get_all(), 0xFF & !(StatusFlag::Decimal as u8));
}

#[test]
fn test_0x58_cli() {
    let mut cpu = CPU::new();

    cpu.reset();
    cpu.load(vec![0x58, 0x00]);
    cpu.program_counter = cpu.mem_read_u16(0xFFFC);
    cpu.status2.set_all(0xFF);
    cpu.run();
    assert_eq!(
        cpu.status2.get_all(),
        0xFF & !(StatusFlag::InterruptDisable as u8)
    );
}

#[test]
fn test_0xb8_clv() {
    let mut cpu = CPU::new();

    cpu.reset();
    cpu.load(vec![0xb8, 0x00]);
    cpu.program_counter = cpu.mem_read_u16(0xFFFC);
    cpu.status2.set_all(0xFF);
    cpu.run();
    assert_eq!(cpu.status2.get_all(), 0xFF & !(StatusFlag::Overflow as u8));
}

#[test]
fn test_cmp() {
    let mut cpu = CPU::new();

    // A > M
    {
        cpu.reset();
        cpu.load(vec![
            0xa9, 0x02, // LDA Immediate of 2.
            0xc9, 0x01, // CMP with 1.
        ]);
        cpu.program_counter = cpu.mem_read_u16(0xFFFC);
        cpu.run();
        assert!(cpu.status2.get(StatusFlag::Carry));
    }

    // A == M
    {
        cpu.reset();
        cpu.load(vec![
            0xa9, 0x01, // LDA Immediate of 1.
            0xc9, 0x01, // CMP with 1.
        ]);
        cpu.program_counter = cpu.mem_read_u16(0xFFFC);
        cpu.run();
        assert!(cpu.status2.get(StatusFlag::Carry));
        assert!(cpu.status2.get(StatusFlag::Zero));
    }

    // A < M
    {
        cpu.reset();
        cpu.load(vec![
            0xa9, 0x01, // LDA Immediate of 1.
            0xc9, 0x02, // CMP with 2.
        ]);
        cpu.program_counter = cpu.mem_read_u16(0xFFFC);
        cpu.run();
        assert!(cpu.status2.get(StatusFlag::Negative));
    }
}

// CMP operations with other AddressingMode values are not tested.
// Assuming testing CMP with Immediate AddressingMode is sufficient.

#[test]
fn test_cpx() {
    let mut cpu = CPU::new();

    // X > M
    {
        cpu.reset();
        cpu.load(vec![
            0xe0, 0x01, // CMP with 1.
        ]);
        cpu.register_x = 2;
        cpu.program_counter = cpu.mem_read_u16(0xFFFC);
        cpu.run();
        assert!(cpu.status2.get(StatusFlag::Carry));
    }

    // X == M
    {
        cpu.reset();
        cpu.load(vec![
            0xe0, 0x01, // CMP with 1.
        ]);
        cpu.register_x = 1;
        cpu.program_counter = cpu.mem_read_u16(0xFFFC);
        cpu.run();
        assert!(cpu.status2.get(StatusFlag::Carry));
        assert!(cpu.status2.get(StatusFlag::Zero));
    }

    // X < M
    {
        cpu.reset();
        cpu.load(vec![
            0xc9, 0x02, // CMP with 2.
        ]);
        cpu.register_x = 1;
        cpu.program_counter = cpu.mem_read_u16(0xFFFC);
        cpu.run();
        assert!(cpu.status2.get(StatusFlag::Negative));
    }
}

// CPX operations with other AddressingMode values are not tested.
// Assuming testing CPX with Immediate AddressingMode is sufficient.

#[test]
fn test_cpy() {
    let mut cpu = CPU::new();

    // Y > M
    {
        cpu.reset();
        cpu.load(vec![
            0xc0, 0x01, // CMP with 1.
        ]);
        cpu.register_y = 2;
        cpu.program_counter = cpu.mem_read_u16(0xFFFC);
        cpu.run();
        assert!(cpu.status2.get(StatusFlag::Carry));
    }

    // Y == M
    {
        cpu.reset();
        cpu.load(vec![
            0xc0, 0x01, // CMP with 1.
        ]);
        cpu.register_y = 1;
        cpu.program_counter = cpu.mem_read_u16(0xFFFC);
        cpu.run();
        assert!(cpu.status2.get(StatusFlag::Carry));
        assert!(cpu.status2.get(StatusFlag::Zero));
    }

    // Y < M
    {
        cpu.reset();
        cpu.load(vec![
            0xc0, 0x02, // CMP with 2.
        ]);
        cpu.register_y = 1;
        cpu.program_counter = cpu.mem_read_u16(0xFFFC);
        cpu.run();
        assert!(cpu.status2.get(StatusFlag::Negative));
    }
}

// CPY operations with other AddressingMode values are not tested.
// Assuming testing CPY with Immediate AddressingMode is sufficient.

#[test]
fn test_0xc6_dec() {
    let mut cpu = CPU::new();

    {
        cpu.reset();
        cpu.load(vec![
            0xA9, 123, // LDA 123.
            0x85, 0x01, // STA at address 0x0001.
            0xc6, 0x01, // DEC.
        ]);
        cpu.program_counter = cpu.mem_read_u16(0xFFFC);
        cpu.run();
        assert!(!cpu.status2.get(StatusFlag::Zero));
        assert!(!cpu.status2.get(StatusFlag::Negative));
        let got = cpu.mem_read(0x0001);
        assert_eq!(got, 122);
    }

    // Test zero flag is set.
    {
        cpu.reset();
        cpu.load(vec![
            0xA9, 1, // LDA 1.
            0x85, 0x01, // STA at address 0x0001.
            0xc6, 0x01, // DEC.
        ]);
        cpu.program_counter = cpu.mem_read_u16(0xFFFC);
        cpu.run();
        assert!(cpu.status2.get(StatusFlag::Zero));
        assert!(!cpu.status2.get(StatusFlag::Negative));
        let got = cpu.mem_read(0x0001);
        assert_eq!(got, 0);
    }

    // Test negative flag is set.
    {
        cpu.reset();
        cpu.load(vec![
            0xA9, 0, // LDA 0.
            0x85, 0x01, // STA at address 0x0001.
            0xc6, 0x01, // DEC.
        ]);
        cpu.program_counter = cpu.mem_read_u16(0xFFFC);
        cpu.run();
        assert!(!cpu.status2.get(StatusFlag::Zero));
        assert!(cpu.status2.get(StatusFlag::Negative));
        let got = cpu.mem_read(0x0001);
        assert_eq!(got, 255);
    }
}

// DEC operations with other AddressingMode values are not tested.
// Assuming testing DEC with Immediate AddressingMode is sufficient.

#[test]
fn test_0xca_dex() {
    let mut cpu = CPU::new();

    {
        cpu.reset();
        cpu.load(vec![0xca]);
        cpu.register_x = 123;
        cpu.program_counter = cpu.mem_read_u16(0xFFFC);
        cpu.run();
        assert!(!cpu.status2.get(StatusFlag::Zero));
        assert!(!cpu.status2.get(StatusFlag::Negative));
        assert_eq!(cpu.register_x, 122);
    }

    // Test zero flag is set.
    {
        cpu.reset();
        cpu.load(vec![0xca]);
        cpu.register_x = 1;
        cpu.program_counter = cpu.mem_read_u16(0xFFFC);
        cpu.run();
        assert!(cpu.status2.get(StatusFlag::Zero));
        assert!(!cpu.status2.get(StatusFlag::Negative));
        assert_eq!(cpu.register_x, 0);
    }
    // Test negative flag is set.
    {
        cpu.reset();
        cpu.load(vec![0xca]);
        cpu.register_x = 0;
        cpu.program_counter = cpu.mem_read_u16(0xFFFC);
        cpu.run();
        assert!(!cpu.status2.get(StatusFlag::Zero));
        assert!(cpu.status2.get(StatusFlag::Negative));
        assert_eq!(cpu.register_x, 255);
    }
}

#[test]
fn test_0x88_dey() {
    let mut cpu = CPU::new();

    {
        cpu.reset();
        cpu.load(vec![0x88]);
        cpu.register_y = 123;
        cpu.program_counter = cpu.mem_read_u16(0xFFFC);
        cpu.run();
        assert!(!cpu.status2.get(StatusFlag::Zero));
        assert!(!cpu.status2.get(StatusFlag::Negative));
        assert_eq!(cpu.register_y, 122);
    }

    // Test zero flag is set.
    {
        cpu.reset();
        cpu.load(vec![0x88]);
        cpu.register_y = 1;
        cpu.program_counter = cpu.mem_read_u16(0xFFFC);
        cpu.run();
        assert!(cpu.status2.get(StatusFlag::Zero));
        assert!(!cpu.status2.get(StatusFlag::Negative));
        assert_eq!(cpu.register_y, 0);
    }
    // Test negative flag is set.
    {
        cpu.reset();
        cpu.load(vec![0x88]);
        cpu.register_y = 0;
        cpu.program_counter = cpu.mem_read_u16(0xFFFC);
        cpu.run();
        assert!(!cpu.status2.get(StatusFlag::Zero));
        assert!(cpu.status2.get(StatusFlag::Negative));
        assert_eq!(cpu.register_y, 255);
    }
}

#[test]
fn test_0x49_eor() {
    let mut cpu = CPU::new();

    {
        cpu.reset();
        cpu.load(vec![
            0xa9,
            0b0000_0011, // LDA.
            0x49,
            0b0000_0101, // XOR.
        ]);
        cpu.program_counter = cpu.mem_read_u16(0xFFFC);
        cpu.run();
        assert!(!cpu.status2.get(StatusFlag::Zero));
        assert!(!cpu.status2.get(StatusFlag::Negative));
        assert_eq!(cpu.register_a, 0b0000_0110);
    }

    // Sets zero flag.
    {
        cpu.reset();
        cpu.load(vec![
            0xa9,
            0b0000_0011, // LDA.
            0x49,
            0b0000_0011, // XOR.
        ]);
        cpu.program_counter = cpu.mem_read_u16(0xFFFC);
        cpu.run();
        assert!(cpu.status2.get(StatusFlag::Zero));
        assert!(!cpu.status2.get(StatusFlag::Negative));
        assert_eq!(cpu.register_a, 0b0000_0000);
    }

    // Sets negative flag.
    {
        cpu.reset();
        cpu.load(vec![
            0xa9,
            0b1000_0000, // LDA.
            0x49,
            0b0000_0000, // XOR.
        ]);
        cpu.program_counter = cpu.mem_read_u16(0xFFFC);
        cpu.run();
        assert!(!cpu.status2.get(StatusFlag::Zero));
        assert!(cpu.status2.get(StatusFlag::Negative));
        assert_eq!(cpu.register_a, 0b1000_0000);
    }
}
// EOR operations with other AddressingMode values are not tested.
// Assuming testing EOR with Immediate AddressingMode is sufficient.

#[test]
fn test_0xe6_inc() {
    let mut cpu = CPU::new();

    {
        cpu.reset();
        cpu.load(vec![
            0xA9, 123, // LDA 123.
            0x85, 0x01, // STA at address 0x0001.
            0xe6, 0x01, // INC.
        ]);
        cpu.program_counter = cpu.mem_read_u16(0xFFFC);
        cpu.run();
        assert!(!cpu.status2.get(StatusFlag::Zero));
        assert!(!cpu.status2.get(StatusFlag::Negative));
        let got = cpu.mem_read(0x0001);
        assert_eq!(got, 124);
    }

    // Test zero flag is set.
    {
        cpu.reset();
        cpu.trace = true;
        cpu.load(vec![
            0xA9, 255, // LDA 1.
            0x85, 0x01, // STA at address 0x0001.
            0xe6, 0x01, // INC.
        ]);
        cpu.program_counter = cpu.mem_read_u16(0xFFFC);
        cpu.run();
        assert!(cpu.status2.get(StatusFlag::Zero));
        assert!(!cpu.status2.get(StatusFlag::Negative));
        let got = cpu.mem_read(0x0001);
        assert_eq!(got, 0);
    }

    // Test negative flag is set.
    {
        cpu.reset();
        cpu.load(vec![
            0xA9,
            0b0111_1111, // LDA 0.
            0x85,
            0x01, // STA at address 0x0001.
            0xe6,
            0x01, // INC.
        ]);
        cpu.program_counter = cpu.mem_read_u16(0xFFFC);
        cpu.run();
        assert!(!cpu.status2.get(StatusFlag::Zero));
        assert!(cpu.status2.get(StatusFlag::Negative));
        let got = cpu.mem_read(0x0001);
        assert_eq!(got, 0b1000_0000);
    }
}

// INC operations with other AddressingMode values are not tested.
// Assuming testing INC with Immediate AddressingMode is sufficient.

#[test]
fn test_0xe8_inx() {
    let mut cpu = CPU::new();

    {
        cpu.reset();
        cpu.load(vec![0xe8]);
        cpu.register_x = 123;
        cpu.program_counter = cpu.mem_read_u16(0xFFFC);
        cpu.run();
        assert!(!cpu.status2.get(StatusFlag::Zero));
        assert!(!cpu.status2.get(StatusFlag::Negative));
        assert_eq!(cpu.register_x, 124);
    }

    // Test zero flag is set.
    {
        cpu.reset();
        cpu.load(vec![0xe8]);
        cpu.register_x = 255;
        cpu.program_counter = cpu.mem_read_u16(0xFFFC);
        cpu.run();
        assert!(cpu.status2.get(StatusFlag::Zero));
        assert!(!cpu.status2.get(StatusFlag::Negative));
        assert_eq!(cpu.register_x, 0);
    }
    // Test negative flag is set.
    {
        cpu.reset();
        cpu.load(vec![0xe8]);
        cpu.register_x = 0b0111_1111;
        cpu.program_counter = cpu.mem_read_u16(0xFFFC);
        cpu.run();
        assert!(!cpu.status2.get(StatusFlag::Zero));
        assert!(cpu.status2.get(StatusFlag::Negative));
        assert_eq!(cpu.register_x, 0b1000_0000);
    }
}

#[test]
fn test_0xc8_iny() {
    let mut cpu = CPU::new();

    {
        cpu.reset();
        cpu.load(vec![0xc8]);
        cpu.register_y = 123;
        cpu.program_counter = cpu.mem_read_u16(0xFFFC);
        cpu.run();
        assert!(!cpu.status2.get(StatusFlag::Zero));
        assert!(!cpu.status2.get(StatusFlag::Negative));
        assert_eq!(cpu.register_y, 124);
    }

    // Test zero flag is set.
    {
        cpu.reset();
        cpu.load(vec![0xc8]);
        cpu.register_y = 255;
        cpu.program_counter = cpu.mem_read_u16(0xFFFC);
        cpu.run();
        assert!(cpu.status2.get(StatusFlag::Zero));
        assert!(!cpu.status2.get(StatusFlag::Negative));
        assert_eq!(cpu.register_y, 0);
    }
    // Test negative flag is set.
    {
        cpu.reset();
        cpu.load(vec![0xc8]);
        cpu.register_y = 0b0111_1111;
        cpu.program_counter = cpu.mem_read_u16(0xFFFC);
        cpu.run();
        assert!(!cpu.status2.get(StatusFlag::Zero));
        assert!(cpu.status2.get(StatusFlag::Negative));
        assert_eq!(cpu.register_y, 0b1000_0000);
    }
}

#[test]
fn test_push_pop() {
    let mut cpu = CPU::new();

    // Test push / pop of u8.
    {
        cpu.reset();
        cpu.stack_push(1);
        cpu.stack_push(2);
        assert_eq!(cpu.stack_pop(), 2);
        assert_eq!(cpu.stack_pop(), 1);
    }
    // Q: is it possible to assert something panics?
    // A:
    {
        // Test push / pop of u16.
        cpu.stack_push_u16(0x1234);
        cpu.stack_push_u16(0xABCD);
        assert_eq!(cpu.stack_pop_u16(), 0xABCD);
        assert_eq!(cpu.stack_pop_u16(), 0x1234);
    }
}

#[test]
fn test_0x20_jsr() {
    let mut cpu = CPU::new();

    {
        cpu.reset();
        // 0x8000 is starting address of instructions.
        cpu.load(vec![
            0x20, 0x04, 0x80, // JSR to 0x8004
            0xFF, // Invalid instruction.
            0xa9, 123, // LDA 123.
        ]);
        cpu.program_counter = cpu.mem_read_u16(0xFFFC);
        cpu.run();
        assert_eq!(cpu.register_a, 123);
        let got = cpu.stack_pop_u16();
        let expect = 0x8002;
        assert_eq!(got, expect, "testing {:02x} and {:02x}", got, expect);
    }
}

#[test]
fn test_0xa2_ldx() {
    let mut cpu = CPU::new();

    {
        cpu.reset();
        cpu.load(vec![0xa2, 123]);
        cpu.program_counter = cpu.mem_read_u16(0xFFFC);
        cpu.run();
        assert_eq!(cpu.register_x, 123);
        assert!(!cpu.status2.get(StatusFlag::Zero));
        assert!(!cpu.status2.get(StatusFlag::Negative));
    }
}

// LDX operations with other AddressingMode values are not tested.
// Assuming testing LDX with Immediate AddressingMode is sufficient.

#[test]
fn test_0xa0_ldy() {
    let mut cpu = CPU::new();

    {
        cpu.reset();
        cpu.load(vec![0xa0, 123]);
        cpu.program_counter = cpu.mem_read_u16(0xFFFC);
        cpu.run();
        assert_eq!(cpu.register_y, 123);
        assert!(!cpu.status2.get(StatusFlag::Zero));
        assert!(!cpu.status2.get(StatusFlag::Negative));
    }
}

// LDY operations with other AddressingMode values are not tested.
// Assuming testing LDY with Immediate AddressingMode is sufficient.

#[test]
fn test_0x4a_lsr() {
    let mut cpu = CPU::new();

    {
        cpu.reset();
        cpu.load(vec![
            0xA9, 0b10, // LDA
            0x4A,
        ]);
        cpu.program_counter = cpu.mem_read_u16(0xFFFC);
        cpu.run();
        assert_eq!(cpu.register_a, 0b01);
        assert!(!cpu.status2.get(StatusFlag::Carry));
        assert!(!cpu.status2.get(StatusFlag::Zero));
        assert!(!cpu.status2.get(StatusFlag::Negative));
    }

    // Check zero flag.
    {
        cpu.reset();
        cpu.load(vec![
            0xA9, 0b01, // LDA
            0x4A,
        ]);
        cpu.program_counter = cpu.mem_read_u16(0xFFFC);
        cpu.run();
        assert_eq!(cpu.register_a, 0b00);
        assert!(cpu.status2.get(StatusFlag::Carry));
        assert!(cpu.status2.get(StatusFlag::Zero));
        assert!(!cpu.status2.get(StatusFlag::Negative));
    }
}

#[test]
fn test_0x46_lsr() {
    let mut cpu = CPU::new();

    {
        cpu.reset();
        cpu.load(vec![
            0xA9, 0b10, // LDA
            0x85, 0x02, // STA ZeroPage
            0x46, 0x02, // LSR
        ]);
        cpu.program_counter = cpu.mem_read_u16(0xFFFC);
        cpu.run();
        assert_eq!(cpu.mem_read(0x0002), 0b01);
        assert!(!cpu.status2.get(StatusFlag::Carry));
        assert!(!cpu.status2.get(StatusFlag::Zero));
        assert!(!cpu.status2.get(StatusFlag::Negative));
    }
}

// LSR operations with other AddressingMode values are not tested.
// Assuming testing LSR with Accumulator and ZeroPage AddressingMode is sufficient.

#[test]
fn test_0xea_nop() {
    let mut cpu = CPU::new();

    {
        cpu.reset();
        cpu.load(vec![0xEA]);
        cpu.program_counter = cpu.mem_read_u16(0xFFFC);
        cpu.run();
    }
}

#[test]
fn test_0x09_ora_immediate() {
    let mut cpu = CPU::new();
    // OR 1001 and 1101
    {
        cpu.reset();
        cpu.load(vec![0x09, 0b1101]);
        cpu.register_a = 0b1001;
        cpu.program_counter = cpu.mem_read_u16(0xFFFC);
        cpu.run();
        assert!(!cpu.status2.get(StatusFlag::Zero));
        assert!(!cpu.status2.get(StatusFlag::Negative));
        assert_eq!(cpu.register_a, 0b1101);
    }

    // Check that negative flag is set.
    {
        cpu.reset();
        cpu.load(vec![0x09, 0b1000_0000]);
        cpu.register_a = 0b0000_0000;
        cpu.program_counter = cpu.mem_read_u16(0xFFFC);
        cpu.run();
        assert_eq!(cpu.register_a, 0b1000_0000);
        // Check that negative flag is set.
        assert!(!cpu.status2.get(StatusFlag::Zero));
        assert!(cpu.status2.get(StatusFlag::Negative));
    }

    // Check that zero flag is set.
    {
        cpu.reset();
        cpu.load(vec![0x09, 0b0]);
        cpu.register_a = 0b0;
        cpu.program_counter = cpu.mem_read_u16(0xFFFC);
        cpu.run();
        assert_eq!(cpu.register_a, 0b0);
        // Check that zero flag is set.
        assert!(cpu.status2.get(StatusFlag::Zero));
        assert!(!cpu.status2.get(StatusFlag::Negative));
    }
}

// ORA operations with other AddressingMode values are not tested.
// Assuming testing ORA with Immediate AddressingMode is sufficient.

#[test]
fn test_0x48_pha() {
    let mut cpu = CPU::new();

    {
        cpu.reset();
        // 0x8000 is starting address of instructions.
        cpu.load(vec![
            0xa9, 123,  // LDA Immediate
            0x48, // PHA
        ]);
        cpu.program_counter = cpu.mem_read_u16(0xFFFC);
        cpu.run();
        assert_eq!(cpu.register_a, 123);
        let got = cpu.stack_pop();
        let expect = 123;
        assert_eq!(got, expect);
    }
}

#[test]
fn test_0x08_php() {
    let mut cpu = CPU::new();

    {
        cpu.reset();
        // 0x8000 is starting address of instructions.
        cpu.load(vec![
            0xa9, 0x00, // LDA Immediate (sets Zero Flag)
            0x08, // PHP
        ]);
        cpu.program_counter = cpu.mem_read_u16(0xFFFC);
        cpu.run();
        let got = cpu.stack_pop();
        let expect = CPU::ZERO_FLAG | CPU::B_FLAG | CPU::ONE_FLAG;
        assert_eq!(got, expect);
    }
}

#[test]
fn test_0x68_pla() {
    let mut cpu = CPU::new();

    {
        cpu.reset();
        // 0x8000 is starting address of instructions.
        cpu.load(vec![
            0xa9, 123,  // LDA Immediate
            0x48, // PHA
            0xa9, 124,  // LDA Immediate
            0x68, // PLA
        ]);
        cpu.program_counter = cpu.mem_read_u16(0xFFFC);
        cpu.run();
        assert_eq!(cpu.register_a, 123);
        assert!(!cpu.status2.get(StatusFlag::Zero));
        assert!(!cpu.status2.get(StatusFlag::Negative));
    }
}

#[test]
fn test_0x28_plp() {
    let mut cpu = CPU::new();

    {
        cpu.reset();
        // 0x8000 is starting address of instructions.
        cpu.load(vec![
            0xa9, 0x00, // LDA Immediate (sets Zero Flag)
            0x08, // PHP
            0xa9, 0x01, // Unsets the zero flag.
            0x28, // PLP
        ]);
        cpu.program_counter = cpu.mem_read_u16(0xFFFC);
        cpu.run();
        assert!(cpu.status2.get(StatusFlag::Zero));
        assert!(cpu.status2.get(StatusFlag::One));
    }
}

#[test]
fn test_0x2a_rol() {
    let mut cpu = CPU::new();

    {
        cpu.reset();
        cpu.load(vec![
            0xA9,
            0b0000_0001, // LDA
            0x2A,        // ROL
        ]);
        cpu.program_counter = cpu.mem_read_u16(0xFFFC);
        cpu.run();
        assert_eq!(cpu.register_a, 0b0000_0010);
        assert!(!cpu.status2.get(StatusFlag::Carry));
        assert!(!cpu.status2.get(StatusFlag::Zero));
        assert!(!cpu.status2.get(StatusFlag::Negative));
    }

    // Stores carry.
    {
        cpu.reset();
        cpu.load(vec![
            0xA9,
            0b1000_0001, // LDA
            0x2A,        // ROL
        ]);
        cpu.program_counter = cpu.mem_read_u16(0xFFFC);
        cpu.run();
        assert_eq!(cpu.register_a, 0b0000_0010);
        assert!(cpu.status2.get(StatusFlag::Carry));
        assert!(!cpu.status2.get(StatusFlag::Zero));
        assert!(!cpu.status2.get(StatusFlag::Negative));
    }

    // Applies carry.
    {
        cpu.reset();
        cpu.load(vec![
            0xA9,
            0b1000_0001, // LDA
            0x2A,        // ROL
            0x2A,        // ROL
        ]);
        cpu.program_counter = cpu.mem_read_u16(0xFFFC);
        cpu.run();
        assert_eq!(cpu.register_a, 0b0000_0101);
        assert!(!cpu.status2.get(StatusFlag::Carry));
        assert!(!cpu.status2.get(StatusFlag::Zero));
        assert!(!cpu.status2.get(StatusFlag::Negative));
    }
}

// ROL operations with other AddressingMode values are not tested.
// Assuming testing ROL with Accumulator and ZeroPage AddressingMode is sufficient.

#[test]
fn test_0x6a_ror() {
    let mut cpu = CPU::new();

    {
        cpu.reset();
        cpu.load(vec![
            0xA9,
            0b0000_0010, // LDA
            0x6A,        // ROR
        ]);
        cpu.program_counter = cpu.mem_read_u16(0xFFFC);
        cpu.run();
        assert_eq!(cpu.register_a, 0b0000_0001);
        assert!(!cpu.status2.get(StatusFlag::Carry));
        assert!(!cpu.status2.get(StatusFlag::Zero));
        assert!(!cpu.status2.get(StatusFlag::Negative));
    }

    // Stores carry.
    {
        cpu.reset();
        cpu.load(vec![
            0xA9,
            0b1000_0001, // LDA
            0x6A,        // ROR
        ]);
        cpu.program_counter = cpu.mem_read_u16(0xFFFC);
        cpu.run();
        assert_eq!(cpu.register_a, 0b0100_0000);
        assert!(cpu.status2.get(StatusFlag::Carry));
        assert!(!cpu.status2.get(StatusFlag::Zero));
        assert!(!cpu.status2.get(StatusFlag::Negative));
    }

    // Applies carry.
    {
        cpu.reset();
        cpu.load(vec![
            0xA9,
            0b1000_0001, // LDA
            0x6A,        // ROR
            0x6A,        // ROR
        ]);
        cpu.program_counter = cpu.mem_read_u16(0xFFFC);
        cpu.run();
        assert_eq!(cpu.register_a, 0b1010_0000);
        assert!(!cpu.status2.get(StatusFlag::Carry));
        assert!(!cpu.status2.get(StatusFlag::Zero));
        assert!(cpu.status2.get(StatusFlag::Negative));
    }
}

// ROR operations with other AddressingMode values are not tested.
// Assuming testing ROR with Accumulator and ZeroPage AddressingMode is sufficient.

#[test]
fn test_0x40_rti() {
    let mut cpu = CPU::new();

    {
        cpu.reset();
        cpu.load(vec![
            0x40, // RTI.
            0xFF, // Invalid.
            0x00,
        ]);
        // Push return address 0x8002
        cpu.stack_push_u16(0x8002);
        // Push status flags.
        cpu.stack_push(0xF0);
        cpu.program_counter = cpu.mem_read_u16(0xFFFC);
        cpu.run();
        // Assert that B flag is removed.
        assert!(!cpu.status2.get(StatusFlag::B));
    }
}

#[test]
fn test_0x60_rts() {
    let mut cpu = CPU::new();

    {
        cpu.reset();
        cpu.load(vec![
            0x60, // RTS.
            0xFF, // Invalid.
            0x00, // Break.
            0xFF, // Invalid.
        ]);
        // Push return address 0x8001
        cpu.stack_push_u16(0x8001);
        cpu.program_counter = cpu.mem_read_u16(0xFFFC);
        cpu.run();
    }
}

#[test]
fn test_0xe9_sbc_immediate() {
    let mut cpu = CPU::new();
    // Subtracts 1 with carry set.
    {
        cpu.reset();
        cpu.load(vec![
            0xA9, 0x01, // LDA 1.
            0xE9, 0x01, // SBC 1.
        ]);
        cpu.program_counter = cpu.mem_read_u16(0xFFFC);
        cpu.status2.set(StatusFlag::Carry, true);
        cpu.run();
        // With the carry set, expect subtracted by 1.
        assert_eq!(cpu.register_a, 0x00);
        // No underflow occurred. Expect carry to be set.
        assert_eq!(cpu.get_carry(), 1);
        assert!(cpu.status2.get(StatusFlag::Carry));
        assert!(cpu.status2.get(StatusFlag::Zero));
    }

    // Subtracts 1 without carry set.
    {
        cpu.reset();
        cpu.load(vec![
            0xA9, 0x01, // LDA 1.
            0xE9, 0x01, // SBC 1.
        ]);
        cpu.program_counter = cpu.mem_read_u16(0xFFFC);
        cpu.run();
        // With the carry set, expect subtracted by 2.
        assert_eq!(cpu.register_a, 0xFF);
        assert_eq!(cpu.get_carry(), 0);
        assert!(cpu.status2.get(StatusFlag::Negative));
        assert!(cpu.status2.get(StatusFlag::Overflow));
    }
}

// SBC operations with other AddressingMode values are not tested.
// Assuming testing SBC with Accumulator and ZeroPage AddressingMode is sufficient.

#[test]
fn test_0x38_sec() {
    let mut cpu = CPU::new();
    cpu.reset();
    cpu.load(vec![0x38]);
    cpu.program_counter = cpu.mem_read_u16(0xFFFC);
    cpu.run();
    assert!(cpu.status2.get(StatusFlag::Carry));
}

#[test]
fn test_0xf8_sed() {
    let mut cpu = CPU::new();
    cpu.reset();
    cpu.load(vec![0xf8]);
    cpu.program_counter = cpu.mem_read_u16(0xFFFC);
    cpu.run();
    assert!(cpu.status2.get(StatusFlag::Decimal));
}

#[test]
fn test_0x78_sei() {
    let mut cpu = CPU::new();
    cpu.reset();
    cpu.load(vec![0x78]);
    cpu.program_counter = cpu.mem_read_u16(0xFFFC);
    cpu.run();
    assert!(cpu.status2.get(StatusFlag::InterruptDisable));
}

#[test]
fn test_0x86_stx_immediate() {
    let mut cpu = CPU::new();
    cpu.reset();
    cpu.load(vec![
        0xA2, 123, // LDX
        0x86, 0x01, // STX
    ]);
    cpu.program_counter = cpu.mem_read_u16(0xFFFC);
    cpu.run();
    assert_eq!(cpu.mem_read(0x0001), 123)
}

// STX operations with other AddressingMode values are not tested.
// Assuming testing with one mode is sufficient.

#[test]
fn test_0x86_sty_immediate() {
    let mut cpu = CPU::new();
    cpu.reset();
    cpu.load(vec![
        0xA0, 123, // LDY
        0x84, 0x01, // STY
    ]);
    cpu.program_counter = cpu.mem_read_u16(0xFFFC);
    cpu.run();
    assert_eq!(cpu.mem_read(0x0001), 123)
}

// STY operations with other AddressingMode values are not tested.
// Assuming testing with one mode is sufficient.

#[test]
fn test_0xaa_tax() {
    let mut cpu = CPU::new();
    cpu.reset();
    cpu.load(vec![0xaa]);
    cpu.program_counter = cpu.mem_read_u16(0xFFFC);
    cpu.register_a = 123;
    cpu.run();
    assert_eq!(cpu.register_x, 123);
}
#[test]
fn test_0xa8_tay() {
    let mut cpu = CPU::new();
    cpu.reset();
    cpu.load(vec![0xa8]);
    cpu.program_counter = cpu.mem_read_u16(0xFFFC);
    cpu.register_a = 123;
    cpu.run();
    assert_eq!(cpu.register_y, 123);
}
#[test]
fn test_0xba_tsx() {
    let mut cpu = CPU::new();
    cpu.reset();
    cpu.load(vec![0xba]);
    cpu.program_counter = cpu.mem_read_u16(0xFFFC);
    cpu.stack_pointer = 123;
    cpu.run();
    assert_eq!(cpu.register_x, 123);
}
#[test]
fn test_0x8a_txa() {
    let mut cpu = CPU::new();
    cpu.reset();
    cpu.load(vec![0x8a]);
    cpu.program_counter = cpu.mem_read_u16(0xFFFC);
    cpu.register_x = 123;
    cpu.run();
    assert_eq!(cpu.register_a, 123);
}
#[test]
fn test_0x9a_txs() {
    let mut cpu = CPU::new();
    cpu.reset();
    cpu.load(vec![0x9a]);
    cpu.program_counter = cpu.mem_read_u16(0xFFFC);
    cpu.register_x = 123;
    cpu.run();
    assert_eq!(cpu.stack_pointer, 123);
}
#[test]
fn test_0x98_tya() {
    let mut cpu = CPU::new();
    cpu.reset();
    cpu.load(vec![0x98]);
    cpu.program_counter = cpu.mem_read_u16(0xFFFC);
    cpu.register_y = 123;
    cpu.run();
    assert_eq!(cpu.register_a, 123);
}
