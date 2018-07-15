#![allow(non_snake_case)]
use mem;
// Sharp LR35902 CPU emulator
pub struct Cpu<'a> {
    mem: &'a mem::Mem<'a>,
    PC: u16,
}

impl<'a> Cpu<'a>{
    pub fn new(mem: &'a mem::Mem) -> Cpu<'a> {
        Cpu{
            PC: 0x100,
            mem: mem,
        }
    }
    pub fn get_PC(&self) -> u16 {
        self.PC
    }

    pub fn print_status(&self) {
        println!("==== CPU ====");
        println!("PC: {:04X}", self.get_PC());
    }

    pub fn reset(&mut self) {
        self.PC = 0x100
    }

    pub fn step(&mut self) {
        println!("{:02X}", self.mem.read8())
    }

}
