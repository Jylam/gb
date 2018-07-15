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

#[allow(dead_code)]
#[derive(Copy, Clone)]
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
#[allow(dead_code)]
impl Registers {
    fn get_AF(self) -> u16 {
       ((self.A as u16)<<8) | ((self.F as u16)&0xFF)
    }
    fn set_AF(&mut self, v: u16) {
        self.A = ((v&0xFF00)>>8) as u8;
        self.F = (v&0xFF) as u8;
    }
    fn get_BC(self) -> u16 {
       ((self.B as u16)<<8) | ((self.C as u16)&0xFF)
    }
    fn set_DE(&mut self, v: u16) {
        self.D = ((v&0xFF00)>>8) as u8;
        self.E = (v&0xFF) as u8;
    }
    fn get_HL(self) -> u16 {
       ((self.H as u16)<<8) | ((self.L as u16)&0xFF)
    }
    fn set_HL(&mut self, v: u16) {
        self.H = (((v&0xFF00)as u16)>>8) as u8;
        self.L = (v&0xFF) as u8;
    }
}

// Sharp LR35902 CPU emulator
pub struct Cpu<'a> {
    mem: mem::Mem<'a>,
    regs: Registers,
    total_cyles: u64,
    opcodes: Vec<Opcode>,
}

pub fn addr16(cpu: &mut Cpu) -> u16 {
    let v = cpu.mem.read16(cpu.get_PC()+1);
    v
}
pub fn imm16(cpu: &mut Cpu) -> u16 {
    let v = cpu.mem.read16(cpu.get_PC()+1);
    v
}
pub fn imm8(cpu: &mut Cpu) -> u8 {
    let v = cpu.mem.read8(cpu.get_PC()+1);
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
    cpu.regs.A = cpu.regs.A^cpu.regs.A;
}
pub fn LDhld16(cpu: &mut Cpu) {
    let imm = imm16(cpu);
    cpu.regs.set_HL(imm);
    println!("LD HL, {:04X}", imm)
}
pub fn LDDhla(cpu: &mut Cpu) {
    cpu.mem.write8(cpu.regs.get_HL(), cpu.regs.A);
    println!("LD [HL], a")
}
pub fn LDcd8(cpu: &mut Cpu) {
    let imm = imm8(cpu);
    cpu.regs.C = imm;
    println!("LD C, {:02X}", imm)
}
pub fn LDbd8(cpu: &mut Cpu) {
    let imm = imm8(cpu);
    cpu.regs.B = imm;
    println!("LD B, {:02X}", imm)
}
pub fn JPa16(cpu: &mut Cpu) {
    let addr = addr16(cpu);
    cpu.regs.PC = addr;
    println!("JP {:04X}", addr)
}
impl<'a> Cpu<'a>{

    pub fn new(mem: mem::Mem) -> Cpu {
        let mut cpu: Cpu;
        cpu = Cpu{
            regs: Registers {
                A: 0,
                B: 0,
                D: 0,
                H: 0,
                F: 0,
                C: 0,
                E: 0,
                L: 0,
                PC: 0,
                SP: 0,
            },
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
        cpu.opcodes[0x06] = Opcode {
            name: "LD B,d8",
            len: 2,
            cycles: 8,
            execute: LDbd8,
            jump: false,
        };
        cpu.opcodes[0x0E] = Opcode {
            name: "LD C,d8",
            len: 2,
            cycles: 8,
            execute: LDcd8,
            jump: false,
        };
        cpu.opcodes[0x21] = Opcode {
            name: "LDhld16",
            len: 3,
            cycles: 13,
            execute: LDhld16,
            jump: false,
        };
        cpu.opcodes[0x32] = Opcode {
            name: "LDD (HL), a",
            len: 1,
            cycles: 8,
            execute: LDDhla,
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
        self.regs.PC
    }

    pub fn print_status(&self) {
        println!("==== CPU ====");
        println!("PC: {:04X}", self.get_PC());
        println!("A : {:02X}\tF : {:02X}", self.regs.A, self.regs.F);
        println!("B : {:02X}\tC : {:02X}", self.regs.B, self.regs.C);
        println!("D : {:02X}\tE : {:02X}", self.regs.D, self.regs.E);
        println!("H : {:02X}\tL : {:02X}", self.regs.H, self.regs.L);
        println!("==== END ====");
    }

    pub fn reset(&mut self) {
        self.regs.PC = 0x100
    }

    pub fn step(&mut self) {
        let code = self.mem.read8(self.regs.PC) as usize;
        let opcode = self.opcodes[code];
        let _name  = opcode.name;
        print!("{:04X}: {:02X} -> ", self.regs.PC, code);
        (opcode.execute)(self);
        self.total_cyles = self.total_cyles + opcode.cycles as u64;
        if !opcode.jump {
            self.regs.PC = self.regs.PC.wrapping_add(opcode.len);
        }
    }


}
