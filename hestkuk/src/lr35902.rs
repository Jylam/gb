#![allow(non_snake_case)]
use mem;

struct Opcode {
    name: &'static str,
    len: u16,
    cycles: u32,
    execute: fn(&mut Cpu),
}


// Sharp LR35902 CPU emulator
pub struct Cpu<'a> {
    mem: mem::Mem<'a>,
    PC: u16,
    opcodes: [Opcode; 256],
}


impl<'a> Cpu<'a>{

    pub fn UNK(&self) {
        println!("*** Unknow instruction")
    }

    pub fn new(mem: mem::Mem) -> Cpu {
        let cpu: Cpu;
        cpu = Cpu{
            PC: 0x100,
            mem: mem,
            opcodes: [
                    Opcode {
                        name: "UNK",
                        len: 1,
                        cycles: 4,
                        execute: self.UNK(cpu),
                    }; 256]
        };



        cpu
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
        let opcode = self.mem.read8(self.PC) as usize;
        let name  = self.opcodes[opcode].name;
        println!("{:04X}: {:02X} {}", self.PC, opcode, name);
        self.PC = self.PC.wrapping_add(self.opcodes[opcode].len);
    }


}
