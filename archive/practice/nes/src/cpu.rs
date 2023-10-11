use crate::opcodes::OPCODES_MAP;

pub struct CPU {
    pub register_a: u8,
    pub register_x: u8,
    pub register_y: u8,
    // stack_pointer is the offset of the address of the stack.
    // The stack starts at address `STACK` + `stack_pointer` and grows downward.
    pub stack_pointer: u8,
    pub status: u8,
    pub program_counter: u16,
    memory: [u8; 0xFFFF],
    trace: bool,
}

// STACK is the starting address of the stack.
const STACK: u16 = 0x0100;
// STACK_RESET is the initial value of `stack_pointer`.
// I expect STACK_RESET could be 0xFF. https://github.com/bugzmanov/nes_ebook/blob/master/code/ch3.3/src/cpu.rs#L346 shows STACK_RESET as 0xFD.
const STACK_RESET: u8 = 0xFD;

#[derive(Debug)]
#[allow(non_camel_case_types)]
pub enum AddressingMode {
    Accumulator,
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
    Relative,
    NoneAddressing,
}

impl CPU {
    pub fn new() -> Self {
        CPU {
            register_a: 0,
            register_x: 0,
            register_y: 0,
            // stack_pointer is the stack pointer.
            stack_pointer: STACK_RESET,
            status: 0,
            program_counter: 0,
            memory: [0; 0xFFFF],
            trace: false,
        }
    }

    pub fn set_trace_mode(&mut self, val: bool) {
        self.trace = val;
    }

    pub fn stack_push(&mut self, val: u8) {
        if self.stack_pointer == 0 {
            panic!("stack overflow: attempting to push when stack_pointer == 0x00");
        }
        let addr = STACK.wrapping_add(self.stack_pointer as u16);
        self.mem_write(addr, val);
        self.stack_pointer -= 1;
    }

    pub fn stack_pop(&mut self) -> u8 {
        if self.stack_pointer == 0xFF {
            panic!("stack underflow: attempting to pop when stack_pointer == 0xFF");
        }
        self.stack_pointer += 1;
        let addr = STACK.wrapping_add(self.stack_pointer as u16);
        return self.mem_read(addr);
    }

    pub fn stack_push_u16(&mut self, val: u16) {
        let hi = (val >> 8) as u8;
        let lo = val as u8;
        self.stack_push(hi);
        self.stack_push(lo);
    }

    pub fn stack_pop_u16(&mut self) -> u16 {
        let lo = self.stack_pop() as u16;
        let hi = self.stack_pop() as u16;
        return (hi << 8) | lo;
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
            AddressingMode::Relative => {
                let signed_offset = self.mem_read(self.program_counter) as i8;
                let mut base = self.program_counter;
                (base, _) = base.overflowing_add_signed(signed_offset as i16);
                // Add one to account for increment to program_counter.
                base = base.wrapping_add(1);
                base
            }
            AddressingMode::NoneAddressing | AddressingMode::Accumulator => {
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

    fn ldx(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);
        self.register_x = value;
        self.set_zero_and_negative_flags(self.register_x);
    }

    fn ldy(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);
        self.register_y = value;
        self.set_zero_and_negative_flags(self.register_y);
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

        // TODO: set ONE_FLAG and INTERRUPT_DISABLE?
        // See: https://github.com/bugzmanov/nes_ebook/blob/c4f905346b27e3ab17277e9651d191ff310f480b/code/ch3.3/src/cpu.rs#L246
        self.status = 0;
        self.program_counter = self.mem_read_u16(0xFFFC);
        self.stack_pointer = STACK_RESET;
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
                todo!("opcode 0x{:02x} is not implemented", op);
            }
            let opcode = *opcode.unwrap();

            if self.trace {
                print!(
                    "\npc={:04x} {} ({:02x})",
                    self.program_counter, opcode.name, opcode.code
                );
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

                    if overflowed {
                        self.status = self.status | CPU::OVERFLOW_FLAG;
                    } else {
                        self.status = self.status & !CPU::OVERFLOW_FLAG;
                    }
                    self.set_zero_and_negative_flags(self.register_a);

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
                "ASL" => {
                    let val;
                    if let AddressingMode::Immediate = opcode.mode {
                        val = self.register_a;
                    } else {
                        let addr = self.get_operand_address(&opcode.mode);
                        val = self.mem_read(addr);
                    }

                    if val & 0b1000_0000 == 0b1000_0000 {
                        self.status = self.status | CPU::CARRY_FLAG;
                    }

                    let result = val << 1;

                    if result == 0 {
                        self.status = self.status | CPU::ZERO_FLAG;
                    }

                    if result & 0b1000_0000 == 0b1000_0000 {
                        self.status = self.status | CPU::NEGATIVE_FLAG;
                    }

                    self.register_a = result;
                }
                "BCC" => {
                    let addr = self.get_operand_address(&opcode.mode);
                    if self.status & CPU::CARRY_FLAG == 0b0000_0000 {
                        self.program_counter = addr;
                    } else {
                        self.program_counter += opcode.bytes - 1;
                    }
                }
                "BCS" => {
                    let addr = self.get_operand_address(&opcode.mode);
                    if self.status & CPU::CARRY_FLAG == CPU::CARRY_FLAG {
                        self.program_counter = addr;
                    } else {
                        self.program_counter += opcode.bytes - 1;
                    }
                }
                "BEQ" => {
                    let addr = self.get_operand_address(&opcode.mode);
                    if self.status & CPU::ZERO_FLAG == CPU::ZERO_FLAG {
                        self.program_counter = addr;
                    } else {
                        self.program_counter += opcode.bytes - 1;
                    }
                }
                "BNE" => {
                    let addr = self.get_operand_address(&opcode.mode);
                    if self.status & CPU::ZERO_FLAG == 0b0000_0000 {
                        self.program_counter = addr;
                    } else {
                        self.program_counter += opcode.bytes - 1;
                    }
                }

                "BIT" => {
                    let addr = self.get_operand_address(&opcode.mode);
                    let val = self.mem_read(addr);
                    if val & self.register_a == 0 {
                        self.status = self.status | CPU::ZERO_FLAG;
                    } else {
                        self.status = self.status & !CPU::ZERO_FLAG;
                    }

                    if val & CPU::OVERFLOW_FLAG != 0 {
                        self.status = self.status | CPU::OVERFLOW_FLAG;
                    } else {
                        self.status = self.status & !CPU::OVERFLOW_FLAG;
                    }

                    if val & CPU::NEGATIVE_FLAG != 0 {
                        self.status = self.status | CPU::NEGATIVE_FLAG;
                    } else {
                        self.status = self.status & !CPU::NEGATIVE_FLAG;
                    }
                    self.program_counter += opcode.bytes - 1;
                }
                "BMI" => {
                    let addr = self.get_operand_address(&opcode.mode);
                    if self.status & CPU::NEGATIVE_FLAG == CPU::NEGATIVE_FLAG {
                        self.program_counter = addr;
                    } else {
                        self.program_counter += opcode.bytes - 1;
                    }
                }
                "BPL" => {
                    let addr = self.get_operand_address(&opcode.mode);
                    if self.status & CPU::NEGATIVE_FLAG == CPU::NEGATIVE_FLAG {
                        // negative. Do not jump.
                        self.program_counter += opcode.bytes - 1;
                    } else {
                        self.program_counter = addr;
                    }
                }
                "BVC" => {
                    let addr = self.get_operand_address(&opcode.mode);
                    if self.status & CPU::OVERFLOW_FLAG == CPU::OVERFLOW_FLAG {
                        // Overflow set. Do not jump.
                        self.program_counter += opcode.bytes - 1;
                    } else {
                        self.program_counter = addr;
                    }
                }
                "BVS" => {
                    let addr = self.get_operand_address(&opcode.mode);
                    if self.status & CPU::OVERFLOW_FLAG == CPU::OVERFLOW_FLAG {
                        self.program_counter = addr;
                    } else {
                        self.program_counter += opcode.bytes - 1;
                    }
                }
                "CLC" => {
                    self.status = self.status & !CPU::CARRY_FLAG;
                    self.program_counter += opcode.bytes - 1;
                }
                "CLD" => {
                    self.status = self.status & !CPU::DECIMAL_FLAG;
                    self.program_counter += opcode.bytes - 1;
                }
                "CLI" => {
                    self.status = self.status & !CPU::INTERRUPT_DISABLE_FLAG;
                    self.program_counter += opcode.bytes - 1;
                }
                "CLV" => {
                    self.status = self.status & !CPU::OVERFLOW_FLAG;
                    self.program_counter += opcode.bytes - 1;
                }
                "CMP" => {
                    let addr = self.get_operand_address(&opcode.mode);
                    let val = self.mem_read(addr);
                    if self.trace {
                        println!("  CMP A={} with M={}", self.register_a, val);
                    }
                    if self.register_a >= val {
                        self.status = self.status | CPU::CARRY_FLAG;
                    } else {
                        self.status = self.status & !CPU::CARRY_FLAG;
                    }

                    if self.register_a == val {
                        self.status = self.status | CPU::ZERO_FLAG;
                    } else {
                        self.status = self.status & !CPU::ZERO_FLAG;
                    }

                    if self.register_a < val {
                        self.status = self.status | CPU::NEGATIVE_FLAG;
                    } else {
                        self.status = self.status & !CPU::NEGATIVE_FLAG;
                    }
                    self.program_counter += opcode.bytes - 1;
                }

                "CPX" => {
                    let addr = self.get_operand_address(&opcode.mode);
                    let val = self.mem_read(addr);
                    if self.trace {
                        println!("  CMP X={} with M={}", self.register_x, val);
                    }
                    if self.register_x >= val {
                        self.status = self.status | CPU::CARRY_FLAG;
                    } else {
                        self.status = self.status & !CPU::CARRY_FLAG;
                    }

                    if self.register_x == val {
                        self.status = self.status | CPU::ZERO_FLAG;
                    } else {
                        self.status = self.status & !CPU::ZERO_FLAG;
                    }

                    if self.register_x < val {
                        self.status = self.status | CPU::NEGATIVE_FLAG;
                    } else {
                        self.status = self.status & !CPU::NEGATIVE_FLAG;
                    }
                    self.program_counter += opcode.bytes - 1;
                }

                "CPY" => {
                    let addr = self.get_operand_address(&opcode.mode);
                    let val = self.mem_read(addr);
                    if self.trace {
                        println!("  CMP Y={} with M={}", self.register_y, val);
                    }
                    if self.register_y >= val {
                        self.status = self.status | CPU::CARRY_FLAG;
                    } else {
                        self.status = self.status & !CPU::CARRY_FLAG;
                    }

                    if self.register_y == val {
                        self.status = self.status | CPU::ZERO_FLAG;
                    } else {
                        self.status = self.status & !CPU::ZERO_FLAG;
                    }

                    if self.register_y < val {
                        self.status = self.status | CPU::NEGATIVE_FLAG;
                    } else {
                        self.status = self.status & !CPU::NEGATIVE_FLAG;
                    }
                    self.program_counter += opcode.bytes - 1;
                }

                "DEC" => {
                    let addr = self.get_operand_address(&opcode.mode);
                    let mut m = self.mem_read(addr);
                    if self.trace {
                        println!("  DEC M={}", m);
                    }
                    m = m.wrapping_sub(1);
                    self.set_zero_and_negative_flags(m);
                    self.mem_write(addr, m);
                    self.program_counter += opcode.bytes - 1;
                }

                "DEX" => {
                    let mut x = self.register_x;
                    if self.trace {
                        println!("  DEC X={}", x);
                    }
                    x = x.wrapping_sub(1);
                    self.set_zero_and_negative_flags(x);
                    self.register_x = x;
                    self.program_counter += opcode.bytes - 1;
                }
                "DEY" => {
                    let mut y = self.register_y;
                    if self.trace {
                        println!("  DEC Y={}", y);
                    }
                    y = y.wrapping_sub(1);
                    self.set_zero_and_negative_flags(y);
                    self.register_y = y;
                    self.program_counter += opcode.bytes - 1;
                }

                "EOR" => {
                    let addr = self.get_operand_address(&opcode.mode);
                    let m = self.mem_read(addr);
                    if self.trace {
                        println!("  EOR M={}", m);
                    }

                    self.register_a = m ^ self.register_a;
                    self.set_zero_and_negative_flags(self.register_a);
                    self.program_counter += opcode.bytes - 1;
                }

                "INC" => {
                    let addr = self.get_operand_address(&opcode.mode);
                    let mut m = self.mem_read(addr);
                    if self.trace {
                        println!("  INC M={}", m);
                    }
                    m = m.wrapping_add(1);
                    self.set_zero_and_negative_flags(m);
                    self.mem_write(addr, m);
                    self.program_counter += opcode.bytes - 1;
                }

                "INX" => {
                    let mut x = self.register_x;
                    if self.trace {
                        println!("  INC X={}", x);
                    }
                    x = x.wrapping_add(1);
                    self.set_zero_and_negative_flags(x);
                    self.register_x = x;
                    self.program_counter += opcode.bytes - 1;
                }
                "INY" => {
                    let mut y = self.register_y;
                    if self.trace {
                        println!("  INC Y={}", y);
                    }
                    y = y.wrapping_add(1);
                    self.set_zero_and_negative_flags(y);
                    self.register_y = y;
                    self.program_counter += opcode.bytes - 1;
                }

                "JSR" => {
                    let addr = self.get_operand_address(&opcode.mode);
                    if self.trace {
                        println!("  JSR to {}", addr);
                    }
                    let ret = self.program_counter + opcode.bytes - 1 - 1;
                    self.stack_push_u16(ret);
                    self.program_counter = addr;
                }

                "LDX" => {
                    self.ldx(&opcode.mode);
                    self.program_counter += opcode.bytes - 1;
                }

                "LDY" => {
                    self.ldy(&opcode.mode);
                    self.program_counter += opcode.bytes - 1;
                }
                "LSR" => {
                    let mut value;
                    let addr;
                    match opcode.mode {
                        AddressingMode::Accumulator => {
                            value = self.register_a;
                            addr = 0xFFFF;
                        }
                        _ => {
                            addr = self.get_operand_address(&opcode.mode);
                            value = self.mem_read(addr);
                        }
                    }

                    println!("Shifting value {value} one bit");
                    if value & 1 == 1 {
                        // Set carry.
                        self.status = self.status | CPU::CARRY_FLAG;
                    } else {
                        self.status = self.status & !CPU::CARRY_FLAG;
                    }

                    value >>= 1;

                    self.set_zero_and_negative_flags(value);

                    match opcode.mode {
                        AddressingMode::Accumulator => {
                            self.register_a = value;
                        }
                        _ => {
                            self.mem_write(addr, value);
                        }
                    }

                    self.program_counter += opcode.bytes - 1;
                }
                "NOP" => {
                    self.program_counter += opcode.bytes - 1;
                }

                "ORA" => {
                    let addr = self.get_operand_address(&opcode.mode);
                    let value = self.mem_read(addr);
                    if self.trace {
                        println!(
                            "  ORA with value: 0b{:08b}. Register A is: 0b{:08b}",
                            value, self.register_a
                        );
                    }

                    self.register_a = self.register_a | value;
                    self.set_zero_and_negative_flags(self.register_a);
                    self.program_counter += opcode.bytes - 1;
                }

                "PHA" => {
                    self.stack_push(self.register_a);
                    self.program_counter += opcode.bytes - 1;
                }

                "PHP" => {
                    self.stack_push(self.status | CPU::B_FLAG | CPU::ONE_FLAG);
                    self.program_counter += opcode.bytes - 1;
                }

                "PLA" => {
                    self.register_a = self.stack_pop();
                    self.set_zero_and_negative_flags(self.register_a);
                    self.program_counter += opcode.bytes - 1;
                }

                "PLP" => {
                    let mut val = self.stack_pop();
                    // Remove the B flag. See https://github.com/bugzmanov/nes_ebook/blob/c4f905346b27e3ab17277e9651d191ff310f480b/code/ch3.3/src/cpu.rs#L480
                    val = val & !CPU::B_FLAG;
                    self.status = val;
                    self.program_counter += opcode.bytes - 1;
                }

                "ROL" => {
                    let mut val;
                    let mut addr = 0;
                    match &opcode.mode {
                        AddressingMode::Accumulator => {
                            val = self.register_a;
                        }
                        mode => {
                            addr = self.get_operand_address(&mode);
                            val = self.mem_read(addr);
                        }
                    }
                    let bit7set = (val & 0b1000_0000) == 0b1000_0000;
                    let has_old_carry = (self.status & CPU::CARRY_FLAG) == CPU::CARRY_FLAG;
                    val <<= 1;
                    if has_old_carry {
                        val |= 0b1;
                    }
                    if bit7set {
                        self.status = self.status | CPU::CARRY_FLAG;
                    } else {
                        self.status = self.status & !CPU::CARRY_FLAG;
                    }
                    self.set_zero_and_negative_flags(val);

                    match &opcode.mode {
                        AddressingMode::Accumulator => {
                            self.register_a = val;
                        }
                        _ => {
                            self.mem_write(addr, val);
                        }
                    }
                    self.program_counter += opcode.bytes - 1;
                }

                "ROR" => {
                    let mut val;
                    let mut addr = 0;
                    match &opcode.mode {
                        AddressingMode::Accumulator => {
                            val = self.register_a;
                        }
                        mode => {
                            addr = self.get_operand_address(&mode);
                            val = self.mem_read(addr);
                        }
                    }
                    let bit0set = (val & 0b0000_0001) == 0b0000_0001;
                    let has_old_carry = (self.status & CPU::CARRY_FLAG) == CPU::CARRY_FLAG;
                    val >>= 1;
                    if has_old_carry {
                        val |= 0b1000_0000;
                    }
                    if bit0set {
                        self.status = self.status | CPU::CARRY_FLAG;
                    } else {
                        self.status = self.status & !CPU::CARRY_FLAG;
                    }
                    self.set_zero_and_negative_flags(val);

                    match &opcode.mode {
                        AddressingMode::Accumulator => {
                            self.register_a = val;
                        }
                        _ => {
                            self.mem_write(addr, val);
                        }
                    }
                    self.program_counter += opcode.bytes - 1;
                }

                "RTI" => {
                    self.status = self.stack_pop();
                    // Remove B flag following https://github.com/bugzmanov/nes_ebook/blob/c4f905346b27e3ab17277e9651d191ff310f480b/code/ch3.3/src/cpu.rs#L710C21-L710C57
                    self.status = self.status & !CPU::B_FLAG;
                    self.program_counter = self.stack_pop_u16();
                }

                "RTS" => {
                    self.program_counter = self.stack_pop_u16() + 1;
                }
                "SBC" => {
                    let addr = self.get_operand_address(&opcode.mode);
                    let value = self.mem_read(addr);
                    if self.trace {
                        println!(
                            "  SBC with value: 0x{:02x}. Carry is: {}",
                            value,
                            self.get_carry()
                        );
                    }

                    let carry = match self.get_carry() {
                        0 => 0,
                        _ => 1,
                    };
                    self.set_carry();
                    // Subtract 1 - carry.
                    let (res, underflowed) = self.register_a.overflowing_sub(1 - carry);
                    if underflowed {
                        self.clear_carry();
                    }
                    self.register_a = res;

                    // Subtract value.
                    let (res, underflowed) = self.register_a.overflowing_sub(value);
                    if underflowed {
                        self.clear_carry();
                    }
                    self.register_a = res;

                    if underflowed {
                        self.status = self.status | CPU::OVERFLOW_FLAG;
                    } else {
                        self.status = self.status & !CPU::OVERFLOW_FLAG;
                    }
                    self.set_zero_and_negative_flags(self.register_a);

                    self.program_counter += opcode.bytes - 1;
                }

                "SEC" => {
                    self.status = self.status | CPU::CARRY_FLAG;
                    self.program_counter += opcode.bytes - 1;
                }

                "SED" => {
                    self.status = self.status | CPU::DECIMAL_FLAG;
                    self.program_counter += opcode.bytes - 1;
                }

                "SEI" => {
                    self.status = self.status | CPU::INTERRUPT_DISABLE_FLAG;
                    self.program_counter += opcode.bytes - 1;
                }

                "STX" => {
                    let addr = self.get_operand_address(&opcode.mode);
                    self.mem_write(addr, self.register_x);
                    self.program_counter += opcode.bytes - 1;
                }

                "STY" => {
                    let addr = self.get_operand_address(&opcode.mode);
                    self.mem_write(addr, self.register_y);
                    self.program_counter += opcode.bytes - 1;
                }

                "TAX" => {
                    self.register_x = self.register_a;
                    self.set_zero_and_negative_flags(self.register_x);
                    self.program_counter += opcode.bytes - 1;
                }
                "TAY" => {
                    self.register_y = self.register_a;
                    self.set_zero_and_negative_flags(self.register_y);
                    self.program_counter += opcode.bytes - 1;
                }
                "TSX" => {
                    self.register_x = self.stack_pointer;
                    self.set_zero_and_negative_flags(self.register_x);
                    self.program_counter += opcode.bytes - 1;
                }
                "TXA" => {
                    self.register_a = self.register_x;
                    self.set_zero_and_negative_flags(self.register_a);
                    self.program_counter += opcode.bytes - 1;
                }
                "TXS" => {
                    self.stack_pointer = self.register_x;
                    // Flags are not set.
                    self.program_counter += opcode.bytes - 1;
                }
                "TYA" => {
                    self.register_a = self.register_y;
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
    pub const OVERFLOW_FLAG: u8 = 0b0100_0000;
    pub const DECIMAL_FLAG: u8 = 0b0000_1000;
    pub const INTERRUPT_DISABLE_FLAG: u8 = 0b0000_0100;
    pub const ONE_FLAG: u8 = 0b0010_0000; // Always pushed as 1.
    pub const B_FLAG: u8 = 0b0001_0000; // See https://www.nesdev.org/wiki/Status_flags#The_B_flag

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
            assert_eq!(cpu.status & CPU::ZERO_FLAG, 0);
            assert_eq!(cpu.status & CPU::CARRY_FLAG, 0);
            assert_eq!(cpu.status & CPU::NEGATIVE_FLAG, 0);
        }

        // Check that carry flag is set.
        {
            cpu.reset();
            cpu.load(vec![0x0A]);
            cpu.register_a = 0b1000_0001;
            cpu.program_counter = cpu.mem_read_u16(0xFFFC);
            cpu.run();
            assert_eq!(cpu.register_a, 0b0000_0010);
            assert_eq!(cpu.status & CPU::ZERO_FLAG, 0);
            assert_eq!(cpu.status & CPU::CARRY_FLAG, CPU::CARRY_FLAG);
            assert_eq!(cpu.status & CPU::NEGATIVE_FLAG, 0);
        }

        // Check that negative flag is set.
        {
            cpu.reset();
            cpu.load(vec![0x0A]);
            cpu.register_a = 0b0100_0001;
            cpu.program_counter = cpu.mem_read_u16(0xFFFC);
            cpu.run();
            assert_eq!(cpu.register_a, 0b1000_0010);
            assert_eq!(cpu.status & CPU::ZERO_FLAG, 0);
            assert_eq!(cpu.status & CPU::CARRY_FLAG, 0);
            assert_eq!(cpu.status & CPU::NEGATIVE_FLAG, CPU::NEGATIVE_FLAG);
        }

        // Check that zero flag is set.
        {
            cpu.reset();
            cpu.load(vec![0x0A]);
            cpu.register_a = 0b0000_0000;
            cpu.program_counter = cpu.mem_read_u16(0xFFFC);
            cpu.run();
            assert_eq!(cpu.register_a, 0b0000_0000);
            assert_eq!(cpu.status & CPU::ZERO_FLAG, CPU::ZERO_FLAG);
            assert_eq!(cpu.status & CPU::CARRY_FLAG, 0);
            assert_eq!(cpu.status & CPU::NEGATIVE_FLAG, 0);
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
            cpu.status = CPU::CARRY_FLAG;
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
            cpu.status = CPU::ZERO_FLAG;
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
            cpu.status = CPU::ZERO_FLAG;
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
            assert_eq!(cpu.status & CPU::ZERO_FLAG, 0b0000_0000);
            assert_eq!(cpu.status & CPU::NEGATIVE_FLAG, CPU::NEGATIVE_FLAG);
            assert_eq!(cpu.status & CPU::OVERFLOW_FLAG, CPU::OVERFLOW_FLAG);
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
            assert_eq!(cpu.status & CPU::ZERO_FLAG, CPU::ZERO_FLAG);
            assert_eq!(cpu.status & CPU::NEGATIVE_FLAG, CPU::NEGATIVE_FLAG);
            assert_eq!(cpu.status & CPU::OVERFLOW_FLAG, CPU::OVERFLOW_FLAG);
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
            assert_eq!(cpu.status & CPU::ZERO_FLAG, 0b0000_0000);
            assert_eq!(cpu.status & CPU::NEGATIVE_FLAG, CPU::NEGATIVE_FLAG);
            assert_eq!(cpu.status & CPU::OVERFLOW_FLAG, CPU::OVERFLOW_FLAG);
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
            cpu.status = CPU::NEGATIVE_FLAG;
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
            cpu.status = 0;
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
            cpu.status = CPU::NEGATIVE_FLAG;
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
            cpu.status = 0;
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
            cpu.status = CPU::OVERFLOW_FLAG;
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
            cpu.status = CPU::OVERFLOW_FLAG;
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
            cpu.status = 0;
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
        cpu.status = 0xFF; // Everything set.
        cpu.run();
        assert_eq!(cpu.status, 0xFF & !(CPU::CARRY_FLAG));
    }

    #[test]
    fn test_0xd8_cld() {
        let mut cpu = CPU::new();

        cpu.reset();
        cpu.load(vec![0xd8, 0x00]);
        cpu.program_counter = cpu.mem_read_u16(0xFFFC);
        cpu.status = 0xFF; // Everything set.
        cpu.run();
        assert_eq!(cpu.status, 0xFF & !(CPU::DECIMAL_FLAG));
    }

    #[test]
    fn test_0x58_cli() {
        let mut cpu = CPU::new();

        cpu.reset();
        cpu.load(vec![0x58, 0x00]);
        cpu.program_counter = cpu.mem_read_u16(0xFFFC);
        cpu.status = 0xFF; // Everything set.
        cpu.run();
        assert_eq!(cpu.status, 0xFF & !(CPU::INTERRUPT_DISABLE_FLAG));
    }

    #[test]
    fn test_0xb8_clv() {
        let mut cpu = CPU::new();

        cpu.reset();
        cpu.load(vec![0xb8, 0x00]);
        cpu.program_counter = cpu.mem_read_u16(0xFFFC);
        cpu.status = 0xFF; // Everything set.
        cpu.run();
        assert_eq!(cpu.status, 0xFF & !(CPU::OVERFLOW_FLAG));
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
            assert_eq!(cpu.status, CPU::CARRY_FLAG);
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
            assert_eq!(cpu.status, CPU::CARRY_FLAG | CPU::ZERO_FLAG);
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
            assert_eq!(cpu.status, CPU::NEGATIVE_FLAG);
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
            assert_eq!(cpu.status, CPU::CARRY_FLAG);
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
            assert_eq!(cpu.status, CPU::CARRY_FLAG | CPU::ZERO_FLAG);
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
            assert_eq!(cpu.status, CPU::NEGATIVE_FLAG);
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
            assert_eq!(cpu.status, CPU::CARRY_FLAG);
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
            assert_eq!(cpu.status, CPU::CARRY_FLAG | CPU::ZERO_FLAG);
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
            assert_eq!(cpu.status, CPU::NEGATIVE_FLAG);
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
            assert_eq!(cpu.status, 0);
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
            assert_eq!(cpu.status, CPU::ZERO_FLAG);
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
            assert_eq!(cpu.status, CPU::NEGATIVE_FLAG);
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
            assert_eq!(cpu.status, 0);
            assert_eq!(cpu.register_x, 122);
        }

        // Test zero flag is set.
        {
            cpu.reset();
            cpu.load(vec![0xca]);
            cpu.register_x = 1;
            cpu.program_counter = cpu.mem_read_u16(0xFFFC);
            cpu.run();
            assert_eq!(cpu.status, CPU::ZERO_FLAG);
            assert_eq!(cpu.register_x, 0);
        }
        // Test negative flag is set.
        {
            cpu.reset();
            cpu.load(vec![0xca]);
            cpu.register_x = 0;
            cpu.program_counter = cpu.mem_read_u16(0xFFFC);
            cpu.run();
            assert_eq!(cpu.status, CPU::NEGATIVE_FLAG);
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
            assert_eq!(cpu.status, 0);
            assert_eq!(cpu.register_y, 122);
        }

        // Test zero flag is set.
        {
            cpu.reset();
            cpu.load(vec![0x88]);
            cpu.register_y = 1;
            cpu.program_counter = cpu.mem_read_u16(0xFFFC);
            cpu.run();
            assert_eq!(cpu.status, CPU::ZERO_FLAG);
            assert_eq!(cpu.register_y, 0);
        }
        // Test negative flag is set.
        {
            cpu.reset();
            cpu.load(vec![0x88]);
            cpu.register_y = 0;
            cpu.program_counter = cpu.mem_read_u16(0xFFFC);
            cpu.run();
            assert_eq!(cpu.status, CPU::NEGATIVE_FLAG);
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
            assert_eq!(cpu.status, 0);
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
            assert_eq!(cpu.status, CPU::ZERO_FLAG);
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
            assert_eq!(cpu.status, CPU::NEGATIVE_FLAG);
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
            assert_eq!(cpu.status, 0);
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
            assert_eq!(cpu.status, CPU::ZERO_FLAG);
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
            assert_eq!(cpu.status, CPU::NEGATIVE_FLAG);
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
            assert_eq!(cpu.status, 0);
            assert_eq!(cpu.register_x, 124);
        }

        // Test zero flag is set.
        {
            cpu.reset();
            cpu.load(vec![0xe8]);
            cpu.register_x = 255;
            cpu.program_counter = cpu.mem_read_u16(0xFFFC);
            cpu.run();
            assert_eq!(cpu.status, CPU::ZERO_FLAG);
            assert_eq!(cpu.register_x, 0);
        }
        // Test negative flag is set.
        {
            cpu.reset();
            cpu.load(vec![0xe8]);
            cpu.register_x = 0b0111_1111;
            cpu.program_counter = cpu.mem_read_u16(0xFFFC);
            cpu.run();
            assert_eq!(cpu.status, CPU::NEGATIVE_FLAG);
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
            assert_eq!(cpu.status, 0);
            assert_eq!(cpu.register_y, 124);
        }

        // Test zero flag is set.
        {
            cpu.reset();
            cpu.load(vec![0xc8]);
            cpu.register_y = 255;
            cpu.program_counter = cpu.mem_read_u16(0xFFFC);
            cpu.run();
            assert_eq!(cpu.status, CPU::ZERO_FLAG);
            assert_eq!(cpu.register_y, 0);
        }
        // Test negative flag is set.
        {
            cpu.reset();
            cpu.load(vec![0xc8]);
            cpu.register_y = 0b0111_1111;
            cpu.program_counter = cpu.mem_read_u16(0xFFFC);
            cpu.run();
            assert_eq!(cpu.status, CPU::NEGATIVE_FLAG);
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
            assert_eq!(cpu.status, 0);
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
            assert_eq!(cpu.status, 0);
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
            assert_eq!(cpu.status, 0);
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
            assert_eq!(cpu.status, CPU::ZERO_FLAG | CPU::CARRY_FLAG);
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
            assert_eq!(cpu.status, 0);
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
            assert_eq!(cpu.status, 0);
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
            assert!(cpu.status == CPU::NEGATIVE_FLAG);
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
            assert!(cpu.status == CPU::ZERO_FLAG);
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
            assert_eq!(cpu.status, 0)
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
            assert_eq!(cpu.status, CPU::ZERO_FLAG | CPU::ONE_FLAG);
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
            assert_eq!(cpu.status, 0);
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
            assert_eq!(cpu.status, CPU::CARRY_FLAG);
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
            assert_eq!(cpu.status, 0);
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
            assert_eq!(cpu.status, 0);
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
            assert_eq!(cpu.status, CPU::CARRY_FLAG);
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
            assert_eq!(cpu.status, CPU::NEGATIVE_FLAG);
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
            assert_eq!(cpu.status, 0xF0 & !CPU::B_FLAG);
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
            cpu.status = CPU::CARRY_FLAG;
            cpu.run();
            // With the carry set, expect subtracted by 1.
            assert_eq!(cpu.register_a, 0x00);
            // No underflow occurred. Expect carry to be set.
            assert_eq!(cpu.get_carry(), 1);
            assert_eq!(cpu.status, CPU::ZERO_FLAG | CPU::CARRY_FLAG);
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
            assert_eq!(cpu.status, CPU::NEGATIVE_FLAG | CPU::OVERFLOW_FLAG);
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
        assert_eq!(cpu.status, CPU::CARRY_FLAG);
    }

    #[test]
    fn test_0xf8_sed() {
        let mut cpu = CPU::new();
        cpu.reset();
        cpu.load(vec![0xf8]);
        cpu.program_counter = cpu.mem_read_u16(0xFFFC);
        cpu.run();
        assert_eq!(cpu.status, CPU::DECIMAL_FLAG);
    }

    #[test]
    fn test_0x78_sei() {
        let mut cpu = CPU::new();
        cpu.reset();
        cpu.load(vec![0x78]);
        cpu.program_counter = cpu.mem_read_u16(0xFFFC);
        cpu.run();
        assert_eq!(cpu.status, CPU::INTERRUPT_DISABLE_FLAG);
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
}
