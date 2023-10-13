use crate::opcodes::OPCODES_MAP;

enum StatusFlag {
    Zero = 0b0000_0010,
    Negative = 0b1000_0000,
    Carry = 0b0000_0001,
    Overflow = 0b0100_0000,
    Decimal = 0b0000_1000,
    InterruptDisable = 0b0000_0100,
    One = 0b0010_0000, // Always pushed as 1.
    B = 0b0001_0000,   // See https://www.nesdev.org/wiki/Status_flags#The_B_flag
}
pub struct Status {
    status: u8,
}

impl Status {
    fn new() -> Self {
        return Self { status: 0 };
    }
    fn reset(&mut self) {
        self.status = 0;
    }
    fn set_all(&mut self, val: u8) {
        self.status = val;
    }
    fn get_all(&self) -> u8 {
        return self.status;
    }
    fn set(&mut self, fl: StatusFlag, val: bool) {
        let fl_u8 = fl as u8;
        if val {
            self.status |= fl_u8;
            return;
        }
        self.status &= !fl_u8;
    }
    fn get(&self, fl: StatusFlag) -> bool {
        let fl_u8 = fl as u8;
        if self.status & fl_u8 != 0 {
            return true;
        }
        return false;
    }
}

pub struct CPU {
    pub register_a: u8,
    pub register_x: u8,
    pub register_y: u8,
    // stack_pointer is the offset of the address of the stack.
    // The stack starts at address `STACK` + `stack_pointer` and grows downward.
    pub stack_pointer: u8,
    pub program_counter: u16,
    pub status2: Status,
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
            status2: Status::new(),
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
        self.status2.reset();
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
                    self.status2.set(StatusFlag::Overflow, overflowed);
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

                    self.status2
                        .set(StatusFlag::Carry, val & 0b1000_0000 == 0b1000_0000);

                    let result = val << 1;

                    self.set_zero_and_negative_flags(result);

                    self.register_a = result;
                }
                "BCC" => {
                    let addr = self.get_operand_address(&opcode.mode);
                    if !self.status2.get(StatusFlag::Carry) {
                        self.program_counter = addr;
                    } else {
                        self.program_counter += opcode.bytes - 1;
                    }
                }
                "BCS" => {
                    let addr = self.get_operand_address(&opcode.mode);
                    if self.status2.get(StatusFlag::Carry) {
                        self.program_counter = addr;
                    } else {
                        self.program_counter += opcode.bytes - 1;
                    }
                }
                "BEQ" => {
                    let addr = self.get_operand_address(&opcode.mode);
                    if self.status2.get(StatusFlag::Zero) {
                        self.program_counter = addr;
                    } else {
                        self.program_counter += opcode.bytes - 1;
                    }
                }
                "BNE" => {
                    let addr = self.get_operand_address(&opcode.mode);
                    if !self.status2.get(StatusFlag::Zero) {
                        self.program_counter = addr;
                    } else {
                        self.program_counter += opcode.bytes - 1;
                    }
                }

                "BIT" => {
                    let addr = self.get_operand_address(&opcode.mode);
                    let val = self.mem_read(addr);

                    self.status2
                        .set(StatusFlag::Zero, val & self.register_a == 0);
                    self.status2
                        .set(StatusFlag::Overflow, (val & 0b0100_0000) != 0);
                    self.status2
                        .set(StatusFlag::Negative, val & 0b1000_0000 != 0);

                    self.program_counter += opcode.bytes - 1;
                }
                "BMI" => {
                    let addr = self.get_operand_address(&opcode.mode);
                    if self.status2.get(StatusFlag::Negative) {
                        self.program_counter = addr;
                    } else {
                        self.program_counter += opcode.bytes - 1;
                    }
                }
                "BPL" => {
                    let addr = self.get_operand_address(&opcode.mode);
                    if self.status2.get(StatusFlag::Negative) {
                        // negative. Do not jump.
                        self.program_counter += opcode.bytes - 1;
                    } else {
                        self.program_counter = addr;
                    }
                }
                "BVC" => {
                    let addr = self.get_operand_address(&opcode.mode);
                    if self.status2.get(StatusFlag::Overflow) {
                        // Overflow set. Do not jump.
                        self.program_counter += opcode.bytes - 1;
                    } else {
                        self.program_counter = addr;
                    }
                }
                "BVS" => {
                    let addr = self.get_operand_address(&opcode.mode);
                    if self.status2.get(StatusFlag::Overflow) {
                        self.program_counter = addr;
                    } else {
                        self.program_counter += opcode.bytes - 1;
                    }
                }
                "CLC" => {
                    self.status2.set(StatusFlag::Carry, false);
                    self.program_counter += opcode.bytes - 1;
                }
                "CLD" => {
                    self.status2.set(StatusFlag::Decimal, false);
                    self.program_counter += opcode.bytes - 1;
                }
                "CLI" => {
                    self.status2.set(StatusFlag::InterruptDisable, false);
                    self.program_counter += opcode.bytes - 1;
                }
                "CLV" => {
                    self.status2.set(StatusFlag::Overflow, false);
                    self.program_counter += opcode.bytes - 1;
                }
                "CMP" => {
                    let addr = self.get_operand_address(&opcode.mode);
                    let val = self.mem_read(addr);
                    if self.trace {
                        println!("  CMP A={} with M={}", self.register_a, val);
                    }
                    self.status2.set(StatusFlag::Carry, self.register_a >= val);
                    self.status2.set(StatusFlag::Zero, self.register_a == val);
                    self.status2
                        .set(StatusFlag::Negative, self.register_a < val);
                    self.program_counter += opcode.bytes - 1;
                }

                "CPX" => {
                    let addr = self.get_operand_address(&opcode.mode);
                    let val = self.mem_read(addr);
                    if self.trace {
                        println!("  CMP X={} with M={}", self.register_x, val);
                    }

                    self.status2.set(StatusFlag::Carry, self.register_x >= val);
                    self.status2.set(StatusFlag::Zero, self.register_x == val);
                    self.status2
                        .set(StatusFlag::Negative, self.register_x < val);

                    self.program_counter += opcode.bytes - 1;
                }

                "CPY" => {
                    let addr = self.get_operand_address(&opcode.mode);
                    let val = self.mem_read(addr);
                    if self.trace {
                        println!("  CMP Y={} with M={}", self.register_y, val);
                    }
                    self.status2.set(StatusFlag::Carry, self.register_y >= val);
                    self.status2.set(StatusFlag::Zero, self.register_y == val);
                    self.status2
                        .set(StatusFlag::Negative, self.register_y < val);
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
                    self.status2.set(StatusFlag::Carry, value & 0b1 == 0b1);

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
                    self.stack_push(self.status2.get_all() | CPU::B_FLAG | CPU::ONE_FLAG);
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
                    self.status2.set_all(val);
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
                    let has_old_carry = self.status2.get(StatusFlag::Carry);
                    val <<= 1;
                    if has_old_carry {
                        val |= 0b1;
                    }
                    self.status2.set(StatusFlag::Carry, bit7set);
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
                    let has_old_carry = self.status2.get(StatusFlag::Carry);
                    val >>= 1;
                    if has_old_carry {
                        val |= 0b1000_0000;
                    }
                    self.status2.set(StatusFlag::Carry, bit0set);
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
                    let popped = self.stack_pop();
                    // Remove B flag following https://github.com/bugzmanov/nes_ebook/blob/c4f905346b27e3ab17277e9651d191ff310f480b/code/ch3.3/src/cpu.rs#L710C21-L710C57
                    self.status2.set_all(popped);
                    self.status2.set(StatusFlag::B, false);
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

                    self.status2.set(StatusFlag::Overflow, underflowed);
                    self.set_zero_and_negative_flags(self.register_a);

                    self.program_counter += opcode.bytes - 1;
                }

                "SEC" => {
                    self.status2.set(StatusFlag::Carry, true);
                    self.program_counter += opcode.bytes - 1;
                }

                "SED" => {
                    self.status2.set(StatusFlag::Decimal, true);
                    self.program_counter += opcode.bytes - 1;
                }

                "SEI" => {
                    self.status2.set(StatusFlag::InterruptDisable, true);
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
        self.status2.set(StatusFlag::Zero, val == 0);
        self.status2
            .set(StatusFlag::Negative, val & 0b1000_0000 != 0);
    }

    fn get_carry(&self) -> u8 {
        if self.status2.get(StatusFlag::Carry) {
            return 0b0000_0001;
        }
        return 0;
    }

    fn set_carry(&mut self) {
        self.status2.set(StatusFlag::Carry, true);
    }

    fn clear_carry(&mut self) {
        self.status2.set(StatusFlag::Carry, false);
    }
}

#[cfg(test)]
mod test;
