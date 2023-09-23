pub mod cpu;
pub mod opcodes;

fn main() {
    let mut cpu = cpu::CPU::new();
    cpu.load_and_run(vec![0]);
}
