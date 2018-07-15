#![allow(non_snake_case)]
use mem;

#[derive(Copy, Clone)]
struct Opcode {
    name: &'static str,
    len: u16,
    cycles: u32,
    execute: fn(&mut Cpu),
    jump: bool,
}

pub struct Registers {
    A: u8,
    B: u8,
    D: u8,
    H: u8,
    F: u8,
    C: u8,
    E: u8,
    L: u8,
    PC: u16,
    SP: u16,
}

// Sharp LR35902 CPU emulator
pub struct Cpu<'a> {
    mem: mem::Mem<'a>,
    PC: u16,
    total_cyles: u64,
    opcodes: Vec<Opcode>,
    A: u8,
}

pub fn addr16(cpu: &mut Cpu) -> u16 {
    let v = cpu.mem.read16(cpu.get_PC()+1);
    v
}
pub fn imm16(cpu: &mut Cpu) -> u16 {
    let v = cpu.mem.read16(cpu.get_PC()+1);
    v
}

pub fn UNK(cpu: &mut Cpu) {
    println!("*** Unknow instruction at {:04X}", cpu.get_PC())
}
pub fn NOP(_cpu: &mut Cpu) {
    println!("NOP")
}
pub fn XORa(cpu: &mut Cpu) {
    println!("XOR a");
    cpu.A = cpu.A^cpu.A;
}
pub fn LDhld16(cpu: &mut Cpu) {
    let imm = imm16(cpu);
    println!("LD HL, {:04X}", imm)
}
pub fn JPa16(cpu: &mut Cpu) {
    let addr = addr16(cpu);
    cpu.PC = addr;
    println!("JP {:04X}", addr)
}
impl<'a> Cpu<'a>{

    pub fn new(mem: mem::Mem) -> Cpu {
        let mut cpu: Cpu;
        cpu = Cpu{
            PC: 0x100,
            A: 0x00,
            mem: mem,
            total_cyles: 0,
            opcodes:
                vec![Opcode{
                    name: "UNK",
                    len: 1,
                    cycles: 4,
                    execute: UNK,
                    jump: false,
                }; 256]

        };
        cpu.opcodes[0] = Opcode {
            name: "NOP",
            len: 1,
            cycles: 4,
            execute: NOP,
                    jump: false,
        };
        cpu.opcodes[0x21] = Opcode {
            name: "LDhld16",
            len: 3,
            cycles: 13,
            execute: LDhld16,
            jump: false,
        };
        cpu.opcodes[0xAF] = Opcode {
            name: "XOR A",
            len: 1,
            cycles: 4,
            execute: XORa,
            jump: false,
        };
        cpu.opcodes[0xC3] = Opcode {
            name: "JP a16",
            len: 3,
            cycles: 16,
            execute: JPa16,
                    jump: true,
        };
        cpu
    }
    pub fn get_PC(&self) -> u16 {
        self.PC
    }

    pub fn print_status(&self) {
        println!("==== CPU ====");
        println!("PC: {:04X}", self.get_PC());
        println!("A : {:02X}", self.A);
        println!("==== END ====");
    }

    pub fn reset(&mut self) {
        self.PC = 0x100
    }

    pub fn step(&mut self) {
        let code = self.mem.read8(self.PC) as usize;
        let opcode = self.opcodes[code];
        let _name  = opcode.name;
        print!("{:04X}: {:02X} -> ", self.PC, code);
        (opcode.execute)(self);
        self.total_cyles = self.total_cyles + opcode.cycles as u64;
        if !opcode.jump {
            self.PC = self.PC.wrapping_add(opcode.len);
        }
    }


}
