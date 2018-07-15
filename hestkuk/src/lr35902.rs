#![allow(non_snake_case)]
use mem;
// Sharp LR35902 CPU emulator
pub struct Cpu<'a> {
    mem: mem::Mem<'a>,
    PC: u16,
}

impl<'a> Cpu<'a>{
    pub fn new(mem: mem::Mem) -> Cpu {
        Cpu{
            PC: 0x100,
            mem: mem.clone(),
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
        self.PC = 0x101
    }

    pub fn step(&mut self) {
        println!("{:04X}: {:02X}", self.PC, self.mem.read8(self.PC))
    }

}
