use crate::opcodes::OPCODES_MAP;

pub struct CPU {
    pub register_a: u8,
    pub register_x: u8,
    pub register_y: u8,
    pub status: u8,
    pub program_counter: u16,
    memory: [u8; 0xFFFF],
    trace: bool,
}

#[derive(Debug)]
#[allow(non_camel_case_types)]
pub enum AddressingMode {
    Immediate,
    ZeroPage,
    ZeroPage_X,
    ZeroPage_Y,
    Absolute,
    Absolute_X,
    Absolute_Y,
    Indirect,
    Indirect_X, // Deref, Shift, Deref
    Indirect_Y, // Deref, Deref, Shift
    NoneAddressing,
}

impl CPU {
    pub fn new() -> Self {
        CPU {
            register_a: 0,
            register_x: 0,
            register_y: 0,
            status: 0,
            program_counter: 0,
            memory: [0; 0xFFFF],
            trace: false,
        }
    }

    pub fn set_trace_mode(&mut self, val: bool) {
        self.trace = val;
    }

    fn get_operand_address(&self, mode: &AddressingMode) -> u16 {
        match mode {
            AddressingMode::Immediate => self.program_counter,
            AddressingMode::ZeroPage => self.mem_read(self.program_counter) as u16,
            AddressingMode::Absolute => self.mem_read_u16(self.program_counter),
            AddressingMode::ZeroPage_X => {
                let pos = self.mem_read(self.program_counter) as u16;
                let addr = pos.wrapping_add(self.register_x as u16);
                return addr;
            }
            AddressingMode::ZeroPage_Y => {
                let pos = self.mem_read(self.program_counter) as u16;
                let addr = pos.wrapping_add(self.register_y as u16);
                return addr;
            }
            AddressingMode::Absolute_X => {
                let pos = self.mem_read_u16(self.program_counter);
                let addr = pos.wrapping_add(self.register_x as u16);
                return addr;
            }
            AddressingMode::Absolute_Y => {
                let pos = self.mem_read_u16(self.program_counter);
                let addr = pos.wrapping_add(self.register_y as u16);
                return addr;
            }
            AddressingMode::Indirect_X => {
                let base = self.mem_read(self.program_counter);
                let ptr = base.wrapping_add(self.register_x);
                let lo = self.mem_read(ptr as u16);
                let hi = self.mem_read(ptr.wrapping_add(1) as u16);
                return (hi as u16) << 8 | (lo as u16);
            }
            AddressingMode::Indirect_Y => {
                let base = self.mem_read(self.program_counter);

                let lo = self.mem_read(base as u16);
                let hi = self.mem_read((base as u8).wrapping_add(1) as u16);
                let deref_base = (hi as u16) << 8 | (lo as u16);
                let deref = deref_base.wrapping_add(self.register_y as u16);
                deref
            }
            AddressingMode::Indirect => {
                let base = self.mem_read_u16(self.program_counter);

                let lo = self.mem_read(base);
                let hi = self.mem_read(base.wrapping_add(1));
                let deref = (hi as u16) << 8 | (lo as u16);
                deref
            }
            AddressingMode::NoneAddressing => {
                panic!("mode {:?} is not supported", mode);
            }
        }
    }

    fn lda(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);
        self.register_a = value;
        self.set_zero_and_negative_flags(self.register_a);
    }

    fn sta(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        self.mem_write(addr, self.register_a);
    }

    fn mem_read(&self, addr: u16) -> u8 {
        return self.memory[addr as usize];
    }

    fn mem_write(&mut self, addr: u16, val: u8) {
        self.memory[addr as usize] = val;
    }

    fn mem_read_u16(&self, addr: u16) -> u16 {
        let lo: u16 = self.mem_read(addr) as u16;
        let hi: u16 = self.mem_read(addr + 1) as u16;
        return lo + (hi << 8);
    }

    fn mem_write_u16(&mut self, addr: u16, val: u16) {
        let lo: u8 = (val & 0xFF) as u8;
        let hi: u8 = ((val & 0xFF00) >> 8) as u8;
        self.mem_write(addr, lo);
        self.mem_write(addr + 1, hi);
    }

    pub fn reset(&mut self) {
        self.register_a = 0;
        self.register_x = 0;
        self.status = 0;
        self.program_counter = self.mem_read_u16(0xFFFC);
    }

    pub fn load(&mut self, program: Vec<u8>) {
        assert!(program.len() <= 0x8000);
        self.memory[0x8000..(0x8000 + program.len())].copy_from_slice(&program[..]);
        self.mem_write_u16(0xFFFC, 0x8000);
    }

    pub fn load_and_run(&mut self, program: Vec<u8>) {
        self.load(program);
        self.reset();
        self.run();
    }

    pub fn run(&mut self) {
        loop {
            // TODO: return error with context if self.program_counter >= len(program)?
            let op = self.mem_read(self.program_counter);
            self.program_counter += 1;

            let opcode = OPCODES_MAP.get(&op);
            if opcode.is_none() {
                todo!();
            }
            let opcode = *opcode.unwrap();

            if self.trace {
                print!("\n{} ({:02x})", opcode.name, opcode.code);
                for i in 0..(opcode.bytes - 1) {
                    print!(" {:02x}", self.mem_read(self.program_counter + i));
                }
                println!();
            }

            match opcode.name {
                "LDA" => {
                    // Load A.
                    self.lda(&opcode.mode);
                    // TODO: return error with context if self.program_counter >= len(program)?
                    self.program_counter += opcode.bytes - 1;
                }
                "STA" => {
                    // Store A.
                    self.sta(&opcode.mode);
                    // TODO: return error with context if self.program_counter >= len(program)?
                    self.program_counter += opcode.bytes - 1;
                }
                "TAX" => {
                    // Transfer A to X.
                    self.register_x = self.register_a;
                    self.set_zero_and_negative_flags(self.register_x);
                }
                "INX" => {
                    // Increment X.
                    // Use `wrapping_add`. Overflow with `+` results in panic.
                    self.register_x = self.register_x.wrapping_add(1);
                    self.set_zero_and_negative_flags(self.register_x)
                }
                "BRK" => {
                    // Break.
                    return;
                }
                "JMP" => {
                    let addr = self.get_operand_address(&opcode.mode);
                    if self.trace {
                        println!("  JMP to address: 0x{:02x}", addr);
                    }
                    self.program_counter = addr;
                }
                "ADC" => {
                    let addr = self.get_operand_address(&opcode.mode);
                    let value = self.mem_read(addr);
                    if self.trace {
                        println!(
                            "  ADC with value: 0x{:02x}. Carry is: {}",
                            value,
                            self.get_carry()
                        );
                    }

                    let carry = self.get_carry();
                    self.clear_carry();
                    // Add carry.
                    let (res, overflowed) = self.register_a.overflowing_add(carry);
                    if overflowed {
                        self.set_carry();
                    }
                    self.register_a = res;

                    // Add value.
                    let (res, overflowed) = self.register_a.overflowing_add(value);
                    if overflowed {
                        self.set_carry();
                    }
                    self.register_a = res;

                    self.program_counter += opcode.bytes - 1;
                }
                "AND" => {
                    let addr = self.get_operand_address(&opcode.mode);
                    let value = self.mem_read(addr);
                    if self.trace {
                        println!(
                            "  AND with value: 0b{:08b}. Register A is: 0b{:08b}",
                            value, self.register_a
                        );
                    }

                    self.register_a = self.register_a & value;
                    self.set_zero_and_negative_flags(self.register_a);
                    self.program_counter += opcode.bytes - 1;
                }
                _ => {
                    todo!();
                }
            }
        }
    }

    pub const ZERO_FLAG: u8 = 0b0000_0010;
    pub const NEGATIVE_FLAG: u8 = 0b1000_0000;
    pub const CARRY_FLAG: u8 = 0b0000_0001;

    fn set_zero_and_negative_flags(&mut self, val: u8) {
        if val == 0 {
            self.status = self.status | CPU::ZERO_FLAG;
        } else {
            self.status = self.status & !CPU::ZERO_FLAG;
        }

        if val & 0b1000_0000 != 0 {
            self.status = self.status | CPU::NEGATIVE_FLAG;
        } else {
            self.status = self.status & !CPU::NEGATIVE_FLAG;
        }
    }

    fn get_carry(&self) -> u8 {
        return self.status & CPU::CARRY_FLAG;
    }

    fn set_carry(&mut self) {
        self.status = self.status | CPU::CARRY_FLAG;
    }

    fn clear_carry(&mut self) {
        self.status = self.status & !(CPU::CARRY_FLAG);
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_0xa9_loads_a() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xA9, 0xFF, 0x00]);
        assert_eq!(cpu.register_a, 0xFF);
    }

    #[test]
    fn test_0xa9_sets_zero_flag() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xA9, 0x00, 0x00]);
        assert!(cpu.status & 0b0000_0010 == 0b10);
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
        assert!(cpu.status & 0b0000_0010 == 0b10);
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
            assert!(cpu.status & 0b1000_0000 == 0b1000_0000);
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
            assert!(cpu.status & 0b0000_0010 == 0b0000_0010);
        }
    }

    // AND operations with other AddressingMode values are not tested.
    // Assuming testing ADC with all AddressingMode is sufficient.
}
