#![allow(non_snake_case)]
use mem;

#[derive(Copy, Clone)]
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
    total_cyles: u64,
    opcodes: [Opcode; 256],
}


pub fn UNK(cpu: &mut Cpu) {
    println!("*** Unknow instruction at {:04X}", cpu.get_PC())
}
impl<'a> Cpu<'a>{


    pub fn new(mem: mem::Mem) -> Cpu {
        let cpu: Cpu;
        cpu = Cpu{
            PC: 0x100,
            mem: mem,
            total_cyles: 0,
            opcodes: [
                    Opcode {
                        name: "UNK",
                        len: 1,
                        cycles: 4,
                        execute: UNK,
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
        let code = self.mem.read8(self.PC) as usize;
        let opcode = self.opcodes[code];
        let name  = opcode.name;
        println!("{:04X}: {:02X} {}", self.PC, code, name);
        (opcode.execute)(self);
        self.total_cyles = self.total_cyles + opcode.cycles as u64;
        self.PC = self.PC.wrapping_add(opcode.len);
    }


}
