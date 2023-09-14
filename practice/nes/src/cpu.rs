use crate::opcodes::OPCODES_MAP;

pub struct CPU {
    pub register_a: u8,
    pub register_x: u8,
    pub register_y: u8,
    pub status: u8,
    pub program_counter: u16,
    memory: [u8; 0xFFFF],
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
    Indirect_X,
    Indirect_Y,
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
        }
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
                _ => {
                    todo!();
                }
            }
        }
    }

    fn set_zero_and_negative_flags(&mut self, val: u8) {
        if val == 0 {
            self.status = self.status | 0b0000_0010;
        } else {
            self.status = self.status & !0b0000_0010;
        }

        if val & 0b1000_0000 != 0 {
            self.status = self.status | 0b1000_0000;
        } else {
            self.status = self.status & !0b1000_0000;
        }
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

    // TODO: test LDA with indirect addressing mode.
}
