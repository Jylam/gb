#![allow(non_snake_case)]
use std::process;
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
    fn set_BC(&mut self, v: u16) {
        self.B = (((v&0xFF00)as u16)>>8) as u8;
        self.C = (v&0xFF) as u8;
    }
    fn get_DE(self) -> u16 {
       ((self.D as u16)<<8) | ((self.E as u16)&0xFF)
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
    fn get_SP(self) -> u16 {
       self.SP
    }
    fn set_SP(&mut self, v: u16) {
        self.SP = v;
    }
    fn get_PC(self) -> u16 {
       self.PC
    }
    fn set_PC(&mut self, v: u16) {
        self.PC = v;
    }

    fn set_FZ(&mut self) {
        self.F |= 0b1000_0000
    }
    fn get_FZ(&mut self) -> bool{
        ((self.F&(0b1000_0000)>>7)==1) as bool
    }
    fn set_FN(&mut self) {
        self.F |= 0b0100_0000
    }
    fn get_FN(&mut self) -> bool{
        ((self.F&(0b0100_0000)>>6)==1) as bool
    }
    fn set_FH(&mut self) {
        self.F |= 0b0010_0000
    }
    fn get_FH(&mut self) -> bool{
        ((self.F&(0b0010_0000)>>5)==1) as bool
    }
    fn set_FC(&mut self) {
        self.F |= 0b0001_0000
    }
    fn get_FC(&mut self) -> bool{
        ((self.F&(0b0001_0000)>>4)==1) as bool
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
    let v = cpu.mem.read16(cpu.regs.get_PC()+1);
    v
}
pub fn imm16(cpu: &mut Cpu) -> u16 {
    let v = cpu.mem.read16(cpu.regs.get_PC()+1);
    v
}
pub fn imm8(cpu: &mut Cpu) -> u8 {
    let v = cpu.mem.read8(cpu.regs.get_PC()+1);
    v
}

pub fn UNK(cpu: &mut Cpu) {
    println!("*** Unknow instruction at {:04X}", cpu.regs.get_PC());
    cpu.print_status();
    process::exit(3);
}
pub fn NOP(_cpu: &mut Cpu) {
    println!("NOP")
}
pub fn XORa(cpu: &mut Cpu) {
    cpu.regs.A = cpu.regs.A^cpu.regs.A;
    println!("XOR A");
}
pub fn INCe(cpu: &mut Cpu) {
    cpu.regs.E = cpu.regs.E.wrapping_add(1);
    println!("INC E");
}
pub fn LDhld16(cpu: &mut Cpu) {
    let imm = imm16(cpu);
    cpu.regs.set_HL(imm);
    println!("LD HL, {:04X}", imm)
}
pub fn LDahl(cpu: &mut Cpu) {
    cpu.regs.A = cpu.mem.read8(cpu.regs.get_HL());
    println!("LDI A, HL")
}
pub fn LDdea(cpu: &mut Cpu) {
    cpu.mem.write8(cpu.regs.get_DE(), cpu.regs.A);
    println!("LDI (DE), A")
}
pub fn LDded16(cpu: &mut Cpu) {
    let imm = imm16(cpu);
    cpu.regs.set_DE(imm);
    println!("LD DE, {:04X}", imm)
}
pub fn LDspd16(cpu: &mut Cpu) {
    let imm = imm16(cpu);
    cpu.regs.set_SP(imm);
    println!("LD SP, {:04X}", imm)
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
pub fn LDad8(cpu: &mut Cpu) {
    let imm = imm8(cpu);
    cpu.regs.A = imm;
    println!("LD A, {:02X}", imm)
}
pub fn LDbd8(cpu: &mut Cpu) {
    let imm = imm8(cpu);
    cpu.regs.B = imm;
    println!("LD B, {:02X}", imm)
}
pub fn LDha8a(cpu: &mut Cpu) {
    let imm = imm8(cpu);
    cpu.mem.write8(0xFF00+imm as u16, cpu.regs.A);
    println!("LDH ({:02X}), A", imm)
}
pub fn LDa16a(cpu: &mut Cpu) {
    let imm = addr16(cpu);
    cpu.mem.write8(imm, cpu.regs.A);
    println!("LD ({:04X}), A", imm)
}
pub fn LDal(cpu: &mut Cpu) {
    cpu.regs.A = cpu.regs.L;
    println!("LDH A, L")
}
pub fn LDba(cpu: &mut Cpu) {
    cpu.regs.B = cpu.regs.A;
    println!("LD B, A")
}
pub fn LDah(cpu: &mut Cpu) {
    cpu.regs.A = cpu.regs.H;
    println!("LDH A, H")
}
pub fn JPa16(cpu: &mut Cpu) {
    let addr = addr16(cpu);
    cpu.regs.PC = addr;
    println!("JP {:04X}", addr)
}
pub fn JRr8(cpu: &mut Cpu) {
    let offset = imm8(cpu) as u16;
    cpu.regs.PC += offset;
    println!("JR {:04X}", cpu.regs.PC)
}
pub fn JPnzr8(cpu: &mut Cpu) {
    let offset = cpu.regs.PC;
    let v      = imm8(cpu) as u16;
    if cpu.regs.get_FZ() == false {
        cpu.regs.PC = offset+v;
    }
    println!("JP NZ {:02X}", v)
}
pub fn CALLa16(cpu: &mut Cpu) {
    let addr = addr16(cpu);
    let next = cpu.regs.PC + 3;
    PushStack(cpu, next);
    cpu.regs.PC = addr;
    println!("CALL {:04X}", addr)
}
pub fn RET(cpu: &mut Cpu) {
    let addr = PopStack(cpu);
    cpu.regs.PC = addr;
    println!("RET (-> {:04X})", addr)
}
pub fn DI(_cpu: &mut Cpu) {
    println!("DI")
}

pub fn PushStack(cpu: &mut Cpu, v: u16) {
    println!("Pushing {:04X} into stack at {:04X}", v, cpu.regs.SP);
    cpu.mem.write16(cpu.regs.SP, v);
    cpu.regs.SP -= 2
}
pub fn PopStack(cpu: &mut Cpu) -> u16 {
    cpu.regs.SP += 2;
    let addr = cpu.mem.read16(cpu.regs.SP);
    println!("Poping {:04X} from stack at {:04X}", addr, cpu.regs.SP);
    addr
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
            name: "LD C, d8",
            len: 2,
            cycles: 8,
            execute: LDcd8,
            jump: false,
        };
        cpu.opcodes[0x11] = Opcode {
            name: "LD DE, d16",
            len: 3,
            cycles: 12,
            execute: LDded16,
            jump: false,
        };
        cpu.opcodes[0x12] = Opcode {
            name: "LD (DE), A",
            len: 1,
            cycles: 8,
            execute: LDdea,
            jump: false,
        };

        cpu.opcodes[0x18] = Opcode {
            name: "JR r8",
            len: 2,
            cycles: 12,
            execute: JRr8,
            jump: true,
        };
        cpu.opcodes[0x1C] = Opcode {
            name: "INC E",
            len: 1,
            cycles: 4,
            execute: INCe,
            jump: false,
        };
        cpu.opcodes[0x20] = Opcode {
            name: "JR NZ, r8",
            len: 2,
            cycles: 12,
            execute: JPnzr8,
            jump: true,
        };
        cpu.opcodes[0x21] = Opcode {
            name: "LD HL, d16",
            len: 3,
            cycles: 13,
            execute: LDhld16,
            jump: false,
        };
        cpu.opcodes[0x2A] = Opcode {
            name: "LDI A, (HL)",
            len: 1,
            cycles: 8,
            execute: LDahl,
            jump: false,
        };
        cpu.opcodes[0x31] = Opcode {
            name: "LD SP, d16",
            len: 3,
            cycles: 12,
            execute: LDspd16,
            jump: false,
        };
        cpu.opcodes[0x32] = Opcode {
            name: "LDD (HL), a",
            len: 1,
            cycles: 8,
            execute: LDDhla,
            jump: false,
        };
        cpu.opcodes[0x3E] = Opcode {
            name: "LD A, d8",
            len: 2,
            cycles: 8,
            execute: LDad8,
            jump: false,
        };
        cpu.opcodes[0x47] = Opcode {
            name: "LD B, A",
            len: 1,
            cycles: 4,
            execute: LDba,
            jump: false,
        };
        cpu.opcodes[0x7C] = Opcode {
            name: "LD A, H",
            len: 1,
            cycles: 4,
            execute: LDah,
            jump: false,
        };
        cpu.opcodes[0x7D] = Opcode {
            name: "LD A, L",
            len: 1,
            cycles: 4,
            execute: LDal,
            jump: false,
        };
        cpu.opcodes[0xAF] = Opcode {
            name: "XOR A",
            len: 1,
            cycles: 4,
            execute: XORa,
            jump: false,
        };
        cpu.opcodes[0xE0] = Opcode {
            name: "LDH (a8),A",
            len: 2,
            cycles: 12,
            execute: LDha8a,
            jump: false,
        };
        cpu.opcodes[0xEA] = Opcode {
            name: "LD (a16),A",
            len: 3,
            cycles: 16,
            execute: LDa16a,
            jump: false,
        };
        cpu.opcodes[0xF3] = Opcode {
            name: "DI",
            len: 1,
            cycles: 4,
            execute: DI,
            jump: false,
        };
        cpu.opcodes[0xC3] = Opcode {
            name: "JP a16",
            len: 3,
            cycles: 16,
            execute: JPa16,
            jump: true,
        };
        cpu.opcodes[0xC9] = Opcode {
            name: "RET",
            len: 1,
            cycles: 16,
            execute: RET,
            jump: true,
        };
        cpu.opcodes[0xCD] = Opcode {
            name: "CALL a16",
            len: 3,
            cycles: 24,
            execute: CALLa16,
            jump: true,
        };
        cpu
    }

    pub fn print_status(&self) {
        println!("==== CPU ====");
        println!("PC: {:04X}", self.regs.get_PC());
        println!("SP: {:04X}", self.regs.get_SP());
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
        print!("{:04X}: {:02X} -> ", self.regs.PC, code);
        (opcode.execute)(self);
        self.total_cyles = self.total_cyles + opcode.cycles as u64;
        if !opcode.jump {
            self.regs.PC = self.regs.PC.wrapping_add(opcode.len);
        }
    }


}
