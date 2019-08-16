// Sharp LR35902 CPU emulator
#![allow(non_snake_case)]
#![allow(dead_code)]

use std::process;
use mem;

#[derive(Copy, Clone)]
struct Opcode {
    name: &'static str,
    len: u16,
    cycles: u32,
    jump: bool,
}

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
    I: bool,
}

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
    fn HLd(&mut self) -> u16 {
        let h = self.get_HL();
        self.set_HL(h-1);
        h
    }
    fn HLi(&mut self) -> u16 {
        let h = self.get_HL();
        self.set_HL(h+1);
        h
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
    fn set_FZ(&mut self, b: bool) {
        if b  {self.F |= 0b1000_0000;}
        else  {self.F &= 0b0111_1111;}
    }
    fn get_FZ(&mut self) -> bool{
        (((self.F&(0b1000_0000))>>7)==1) as bool
    }

    fn set_FN(&mut self, b: bool) {
        if b {self.F |= 0b0100_0000;}
        else {self.F &= 0b1011_1111;}
    }
    fn get_FN(&mut self) -> bool{
        (((self.F&(0b0100_0000))>>6)==1) as bool
    }

    fn set_FH(&mut self, b: bool) {
        if b {self.F |= 0b0010_0000;}
        else {self.F &= 0b1101_1111;}
    }
    fn get_FH(&mut self) -> bool{
        (((self.F&(0b0010_0000))>>5)==1) as bool
    }

    fn set_FC(&mut self, b: bool) {
        if b {self.F |= 0b0001_0000;}
        else {self.F &= 0b1110_1111;}
    }
    fn get_FC(&mut self) -> bool{
        (((self.F&(0b0001_0000))>>4)==1) as bool
    }

}

#[derive(Clone)]
pub struct Cpu<'a> {
    pub mem: mem::Mem<'a>,
    regs: Registers,
    total_cyles: u64,
    halted: bool,
    setdi: u32,
    setei: u32,
}






impl<'a> Cpu<'a>{
    pub fn PushStack(&mut self, v: u16) {
        debug!("Pushing {:04X} into stack at {:04X}", v, self.regs.SP);
        self.mem.write16(self.regs.SP, v);
        self.regs.SP.wrapping_sub(2);
    }
    pub fn PopStack(&mut self) -> u16 {
        self.regs.SP.wrapping_add(2);
        let addr = self.mem.read16(self.regs.SP);
        debug!("Poping {:04X} from stack at {:04X}", addr, self.regs.SP);
        addr
    }

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
                I: false,
                PC: 0,
                SP: 0,
            },
            mem: mem,
            total_cyles: 0,
            halted: false,
            setdi: 0,
            setei: 0,
        };
        cpu
    }


    pub fn readMem8(&mut self, addr: u16) -> u8 {
        self.mem.read8(addr)
    }
    pub fn readMem16(&mut self, addr: u16) -> u16 {
        self.mem.read16(addr)
    }
    pub fn writeMem8(&mut self, addr: u16, v: u8)  {
        self.mem.write8(addr, v)
    }
    pub fn imm16(&mut self) -> u16 {
        let addr = self.regs.get_PC()+1;
        let v = self.readMem16(addr);
        v
    }
    pub fn imm8(&mut self) -> u8 {
        let addr = self.regs.get_PC()+1;
        let v = self.readMem8(addr);
        v
    }
    pub fn fetch8(&mut self) -> u8 {
        let pc = self.regs.get_PC();
        let v = self.readMem8(pc);
        self.regs.set_PC(pc+1);
        v
    }
    pub fn fetch16(&mut self) -> u16 {
        let pc = self.regs.get_PC();
        let v = self.readMem16(pc);
        self.regs.set_PC(pc+2);
        v
    }

    pub fn print_status(&mut self) {
        debug!("==== CPU ====");
        debug!("PC: {:04X}", self.regs.get_PC());
        debug!("SP: {:04X}", self.regs.get_SP());
        debug!("A : {:02X}\tF : {:02X}", self.regs.A, self.regs.F);
        debug!("B : {:02X}\tC : {:02X}", self.regs.B, self.regs.C);
        debug!("D : {:02X}\tE : {:02X}", self.regs.D, self.regs.E);
        debug!("H : {:02X}\tL : {:02X}", self.regs.H, self.regs.L);
        debug!("RST Vectors : ");
        /*        for i in vec![0x00,0x08,0x10,0x18,0x20,0x28,0x30,0x38].iter() {
                  debug!("0x00{:02X}:  {:02X} {:02X}", i, self.mem.read8(*i as u16), self.mem.read8((i+1) as u16));
                  }*/
        debug!("==== END ====");
        //        self.mem.print_infos();
    }

    pub fn interrupts_enabled(&mut self) -> bool {
        self.regs.I
    }

    pub fn irq_vblank(&mut self) {
        //DI(self);
        let addr = self.regs.PC;
        self.PushStack(addr);
        self.regs.PC = 0x0040;
    }

    pub fn reset(&mut self) {
        self.regs.PC = 0x0000
    }

    pub fn step(&mut self) -> u8 {
        let code = self.fetch8() as usize;

        debug!("----------------------------------------");
        debug!("{:04X}: {:02X} -> ", self.regs.PC-1, code);

        let cur_cycle_count = match code {

            0x01 => { let v = self.fetch16(); self.regs.set_BC(v); 3 },
            0x02 => { self.mem.write8(self.regs.get_BC(), self.regs.A); 2 },
            0x03 => { self.regs.set_BC(self.regs.get_BC().wrapping_add(1)); 2 },
            0x04 => { self.regs.B = self.alu_inc(self.regs.B); 1 },
            0x05 => { self.regs.B = self.alu_dec(self.regs.B); 1 },
            0x06 => { self.regs.B = self.fetch8(); 2 },
            0x07 => { self.regs.A = self.alu_rlc(self.regs.A); self.regs.set_FZ(false); 1 },
            0x08 => { let a = self.fetch16(); self.mem.write16(a, self.regs.get_SP()); 5 },
            0x09 => { self.alu_add16(self.regs.get_BC()); 2 },
            0x0A => { self.regs.A = self.mem.read8(self.regs.get_BC()); 2 },
            0x0B => { self.regs.set_BC(self.regs.get_BC().wrapping_sub(1)); 2 },
            0x0C => { let c = self.regs.C; self.regs.C = self.alu_inc(c); 1 },
            0x0D => { self.regs.C = self.alu_dec(self.regs.C); 1 },
            0x0E => { let v = self.fetch8(); self.regs.C = v; 1 },
            0x0F => { self.regs.A = self.alu_rrc(self.regs.A); self.regs.set_FZ(false); 1 },
            0x10 => { self.stop(); 1 }, // STOP
            0x11 => { let v = self.fetch16(); self.regs.set_DE(v); 3 },
            0x12 => { self.mem.write8(self.regs.get_DE(), self.regs.A); 2 },
            0x13 => { self.regs.set_DE(self.regs.get_DE().wrapping_add(1)); 2 },
            0x14 => { self.regs.D = self.alu_inc(self.regs.D); 1 },
            0x15 => { self.regs.D = self.alu_dec(self.regs.D); 1 },
            0x16 => { self.regs.D = self.fetch8(); 2 },
            0x17 => { self.regs.A = self.alu_rl(self.regs.A); self.regs.set_FZ(false); 1 },
            0x18 => { self.cpu_jr(); 3 },
            0x19 => { self.alu_add16(self.regs.get_DE()); 2 },
            0x1A => { self.regs.A = self.mem.read8(self.regs.get_DE()); 2 },
            0x1B => { self.regs.set_DE(self.regs.get_DE().wrapping_sub(1)); 2 },
            0x1C => { self.regs.E = self.alu_inc(self.regs.E); 1 },
            0x1D => { self.regs.E = self.alu_dec(self.regs.E); 1 },
            0x1E => { self.regs.E = self.fetch8(); 2 },
            0x1F => { self.regs.A = self.alu_rr(self.regs.A); self.regs.set_FZ(false); 1 },
            0x20 => { if !self.regs.get_FZ() { self.cpu_jr(); 1 } else { self.regs.PC += 1; 1}},
            0x21 => { let v = self.fetch16(); self.regs.set_HL(v); 1},
            0x22 => { self.mem.write8(self.regs.HLi(), self.regs.A); 2 },
            0x23 => { let v = self.regs.get_HL().wrapping_add(1); self.regs.set_HL(v); 2 },
            0x24 => { self.regs.H = self.alu_inc(self.regs.H); 1 },
            0x25 => { self.regs.H = self.alu_dec(self.regs.H); 1 },
            0x26 => { self.regs.H = self.fetch8(); 2 },
            0x27 => { self.alu_daa(); 1 },
            0x28 => { if self.regs.get_FZ() { self.cpu_jr(); 3 } else { self.regs.PC += 1; 2  } },
            0x29 => { let v = self.regs.get_HL(); self.alu_add16(v); 2 },
            0x2A => { self.regs.A = self.mem.read8(self.regs.HLi()); 2 },
            0x2B => { let v = self.regs.get_HL().wrapping_sub(1); self.regs.set_HL(v); 2 },
            0x2C => { self.regs.L = self.alu_inc(self.regs.L); 1 },
            0x2D => { self.regs.L = self.alu_dec(self.regs.L); 1 },
            0x2E => { self.regs.L = self.fetch8(); 2 },
            0x2F => { self.regs.A = !self.regs.A; self.regs.set_FH(true); self.regs.set_FN( true); 1 },
            0x30 => { if !self.regs.get_FC() { self.cpu_jr(); 3 } else { self.regs.PC += 1; 2 } },
            0x31 => { let sp = self.fetch16(); self.regs.SP = sp; 1},
            0x32 => { self.mem.write8(self.regs.HLd(), self.regs.A); 1}
            0x33 => { self.regs.set_SP(self.regs.get_SP().wrapping_add(1)); 2 },
            0x34 => { let a = self.regs.get_HL(); let v = self.mem.read8(a); let v2 = self.alu_inc(v); self.mem.write8(a, v2); 3 },
            0x35 => { let a = self.regs.get_HL(); let v = self.mem.read8(a); let v2 = self.alu_dec(v); self.mem.write8(a, v2); 3 },
            0x36 => { let v = self.fetch8(); self.mem.write8(self.regs.get_HL(), v); 3 },
            0x37 => { self.regs.set_FC(true); self.regs.set_FH(false); self.regs.set_FN( false); 1 },
            0x38 => { if self.regs.get_FC() { self.cpu_jr(); 3 } else { self.regs.PC += 1; 2  } },
            0x39 => { self.alu_add16(self.regs.get_SP()); 2 },
            0x3A => { self.regs.A = self.mem.read8(self.regs.HLd()); 2 },
            0x3B => { self.regs.set_SP( self.regs.get_SP().wrapping_sub(1)); 2 },
            0x3C => { self.regs.A = self.alu_inc(self.regs.A); 1 },
            0x3D => { self.regs.A = self.alu_dec(self.regs.A); 1 },
            0x3E => { self.regs.A = self.fetch8(); 1 }
            0x3F => { let v = !self.regs.get_FC(); self.regs.set_FC(v); self.regs.set_FH(false); self.regs.set_FN(false); 1 },
            0x40 => { 1 },
            0x41 => { self.regs.B = self.regs.C; 1 },
            0x42 => { self.regs.B = self.regs.D; 1 },
            0x43 => { self.regs.B = self.regs.E; 1 },
            0x44 => { self.regs.B = self.regs.H; 1 },
            0x45 => { self.regs.B = self.regs.L; 1 },
            0x46 => { self.regs.B = self.mem.read8(self.regs.get_HL()); 2 },
            0x47 => { self.regs.B = self.regs.A; 1 },
            0x48 => { self.regs.C = self.regs.B; 1 },
            0x49 => { 1 },
            0x4A => { self.regs.C = self.regs.D; 1 },
            0x4B => { self.regs.C = self.regs.E; 1 },
            0x4C => { self.regs.C = self.regs.H; 1 },
            0x4D => { self.regs.C = self.regs.L; 1 },
            0x4E => { self.regs.C = self.mem.read8(self.regs.get_HL()); 2 },
            0x4F => { self.regs.C = self.regs.A; 1 },
            0x50 => { self.regs.D = self.regs.B; 1 },
            0x51 => { self.regs.D = self.regs.C; 1 },
            0x52 => { 1 },
            0x53 => { self.regs.D = self.regs.E; 1 },
            0x54 => { self.regs.D = self.regs.H; 1 },
            0x55 => { self.regs.D = self.regs.L; 1 },
            0x56 => { self.regs.D = self.mem.read8(self.regs.get_HL()); 2 },
            0x57 => { self.regs.D = self.regs.A; 1 },
            0x58 => { self.regs.E = self.regs.B; 1 },
            0x59 => { self.regs.E = self.regs.C; 1 },
            0x5A => { self.regs.E = self.regs.D; 1 },
            0x5B => { 1 },
            0x5C => { self.regs.E = self.regs.H; 1 },
            0x5D => { self.regs.E = self.regs.L; 1 },
            0x5E => { self.regs.E = self.mem.read8(self.regs.get_HL()); 2 },
            0x5F => { self.regs.E = self.regs.A; 1 },
            0x60 => { self.regs.H = self.regs.B; 1 },
            0x61 => { self.regs.H = self.regs.C; 1 },
            0x62 => { self.regs.H = self.regs.D; 1 },
            0x63 => { self.regs.H = self.regs.E; 1 },
            0x64 => { 1 },
            0x65 => { self.regs.H = self.regs.L; 1 },
            0x66 => { self.regs.H = self.mem.read8(self.regs.get_HL()); 2 },
            0x67 => { self.regs.H = self.regs.A; 1 },
            0x68 => { self.regs.L = self.regs.B; 1 },
            0x69 => { self.regs.L = self.regs.C; 1 },
            0x6A => { self.regs.L = self.regs.D; 1 },
            0x6B => { self.regs.L = self.regs.E; 1 },
            0x6C => { self.regs.L = self.regs.H; 1 },
            0x6D => { 1 },
            0x6E => { self.regs.L = self.mem.read8(self.regs.get_HL()); 2 },
            0x6F => { self.regs.L = self.regs.A; 1 },
            0x70 => { self.mem.write8(self.regs.get_HL(), self.regs.B); 2 },
            0x71 => { self.mem.write8(self.regs.get_HL(), self.regs.C); 2 },
            0x72 => { self.mem.write8(self.regs.get_HL(), self.regs.D); 2 },
            0x73 => { self.mem.write8(self.regs.get_HL(), self.regs.E); 2 },
            0x74 => { self.mem.write8(self.regs.get_HL(), self.regs.H); 2 },
            0x75 => { self.mem.write8(self.regs.get_HL(), self.regs.L); 2 },
            0x76 => { self.halted = true; 1 },
            0x77 => { self.mem.write8(self.regs.get_HL(), self.regs.A); 2 },
            0x78 => { self.regs.A = self.regs.B; 1 },
            0x79 => { self.regs.A = self.regs.C; 1 },
            0x7A => { self.regs.A = self.regs.D; 1 },
            0x7B => { self.regs.A = self.regs.E; 1 },
            0x7C => { self.regs.A = self.regs.H; 1 },
            0x7D => { self.regs.A = self.regs.L; 1 },
            0x7E => { self.regs.A = self.mem.read8(self.regs.get_HL()); 2 },
            0x7F => { 1 },
            0x80 => { self.alu_add(self.regs.B, false); 1 },
            0x81 => { self.alu_add(self.regs.C, false); 1 },
            0x82 => { self.alu_add(self.regs.D, false); 1 },
            0x83 => { self.alu_add(self.regs.E, false); 1 },
            0x84 => { self.alu_add(self.regs.H, false); 1 },
            0x85 => { self.alu_add(self.regs.L, false); 1 },
            0x86 => { let v = self.mem.read8(self.regs.get_HL()); self.alu_add(v, false); 2 },
            0x87 => { self.alu_add(self.regs.A, false); 1 },
            0x88 => { self.alu_add(self.regs.B, true); 1 },
            0x89 => { self.alu_add(self.regs.C, true); 1 },
            0x8A => { self.alu_add(self.regs.D, true); 1 },
            0x8B => { self.alu_add(self.regs.E, true); 1 },
            0x8C => { self.alu_add(self.regs.H, true); 1 },
            0x8D => { self.alu_add(self.regs.L, true); 1 },
            0x8E => { let v = self.mem.read8(self.regs.get_HL()); self.alu_add(v, true); 2 },
            0x8F => { self.alu_add(self.regs.A, true); 1 },
            0x90 => { self.alu_sub(self.regs.B, false); 1 },
            0x91 => { self.alu_sub(self.regs.C, false); 1 },
            0x92 => { self.alu_sub(self.regs.D, false); 1 },
            0x93 => { self.alu_sub(self.regs.E, false); 1 },
            0x94 => { self.alu_sub(self.regs.H, false); 1 },
            0x95 => { self.alu_sub(self.regs.L, false); 1 },
            0x96 => { let v = self.mem.read8(self.regs.get_HL()); self.alu_sub(v, false); 2 },
            0x97 => { self.alu_sub(self.regs.A, false); 1 },
            0x98 => { self.alu_sub(self.regs.B, true); 1 },
            0x99 => { self.alu_sub(self.regs.C, true); 1 },
            0x9A => { self.alu_sub(self.regs.D, true); 1 },
            0x9B => { self.alu_sub(self.regs.E, true); 1 },
            0x9C => { self.alu_sub(self.regs.H, true); 1 },
            0x9D => { self.alu_sub(self.regs.L, true); 1 },
            0x9E => { let v = self.mem.read8(self.regs.get_HL()); self.alu_sub(v, true); 2 },
            0x9F => { self.alu_sub(self.regs.A, true); 1 },
            0xA0 => { self.alu_and(self.regs.B); 1 },
            0xA1 => { self.alu_and(self.regs.C); 1 },
            0xA2 => { self.alu_and(self.regs.D); 1 },
            0xA3 => { self.alu_and(self.regs.E); 1 },
            0xA4 => { self.alu_and(self.regs.H); 1 },
            0xA5 => { self.alu_and(self.regs.L); 1 },
            0xA6 => { let v = self.mem.read8(self.regs.get_HL()); self.alu_and(v); 2 },
            0xA7 => { self.alu_and(self.regs.A); 1 },
            0xA8 => { self.alu_xor(self.regs.B); 1 },
            0xA9 => { self.alu_xor(self.regs.C); 1 },
            0xAA => { self.alu_xor(self.regs.D); 1 },
            0xAB => { self.alu_xor(self.regs.E); 1 },
            0xAC => { self.alu_xor(self.regs.H); 1 },
            0xAD => { self.alu_xor(self.regs.L); 1 },
            0xAE => { let v = self.mem.read8(self.regs.get_HL()); self.alu_xor(v); 2 },
            0xAF => { let A  = self.regs.A; self.alu_xor(A); 1},
            0xB0 => { self.alu_or(self.regs.B); 1 },
            0xB1 => { self.alu_or(self.regs.C); 1 },
            0xB2 => { self.alu_or(self.regs.D); 1 },
            0xB3 => { self.alu_or(self.regs.E); 1 },
            0xB4 => { self.alu_or(self.regs.H); 1 },
            0xB5 => { self.alu_or(self.regs.L); 1 },
            0xB6 => { let v = self.mem.read8(self.regs.get_HL()); self.alu_or(v); 2 },
            0xB7 => { self.alu_or(self.regs.A); 1 },
            0xB8 => { self.alu_cp(self.regs.B); 1 },
            0xB9 => { self.alu_cp(self.regs.C); 1 },
            0xBA => { self.alu_cp(self.regs.D); 1 },
            0xBB => { self.alu_cp(self.regs.E); 1 },
            0xBC => { self.alu_cp(self.regs.H); 1 },
            0xBD => { self.alu_cp(self.regs.L); 1 },
            0xBE => { let v = self.mem.read8(self.regs.get_HL()); self.alu_cp(v); 2 },
            0xBF => { self.alu_cp(self.regs.A); 1 },
            0xC0 => { if !self.regs.get_FZ() { self.regs.PC = self.PopStack(); 5 } else { 2 } },
            0xC1 => { let v = self.PopStack(); self.regs.set_BC(v); 3 },
            0xC2 => { if !self.regs.get_FZ() { self.regs.PC = self.fetch16(); 4 } else { self.regs.PC += 2; 3 } },
            0xC3 => { self.regs.PC = self.fetch16(); 4 },
            0xC4 => { if !self.regs.get_FZ() { self.PushStack(self.regs.PC + 2); self.regs.PC = self.fetch16(); 6 } else { self.regs.PC += 2; 3 } },
            0xC5 => { self.PushStack(self.regs.get_BC()); 4 },
            0xC6 => { let v = self.fetch8(); self.alu_add(v, false); 2 },
            0xC7 => { self.PushStack(self.regs.PC); self.regs.PC = 0x00; 4 },
            0xC8 => { if self.regs.get_FZ() { self.regs.PC = self.PopStack(); 5 } else { 2 } },
            0xC9 => { self.regs.PC = self.PopStack(); 4 },
            0xCA => { if self.regs.get_FZ() { self.regs.PC = self.fetch16(); 4 } else { self.regs.PC += 2; 3 } },
            0xCB => { self.step_CB() },
            0xCC => { if self.regs.get_FZ() { self.PushStack(self.regs.PC + 2); self.regs.PC = self.fetch16(); 6 } else { self.regs.PC += 2; 3 } },
            0xCD => { self.PushStack(self.regs.PC + 2); self.regs.PC = self.fetch16(); 6 },
            0xCE => { let v = self.fetch8(); self.alu_add(v, true); 2 },
            0xCF => { self.PushStack(self.regs.PC); self.regs.PC = 0x08; 4 },
            0xD0 => { if !self.regs.get_FC() { self.regs.PC = self.PopStack(); 5 } else { 2 } },
            0xD1 => { let v = self.PopStack(); self.regs.set_DE(v); 3 },
            0xD2 => { if !self.regs.get_FC() { self.regs.PC = self.fetch16(); 4 } else { self.regs.PC += 2; 3 } },
            0xD4 => { if !self.regs.get_FC() { self.PushStack(self.regs.PC + 2); self.regs.PC = self.fetch16(); 6 } else { self.regs.PC += 2; 3 } },
            0xD5 => { self.PushStack(self.regs.get_DE()); 4 },
            0xD6 => { let v = self.fetch8(); self.alu_sub(v, false); 2 },
            0xD7 => { self.PushStack(self.regs.PC); self.regs.PC = 0x10; 4 },
            0xD8 => { if self.regs.get_FC() { self.regs.PC = self.PopStack(); 5 } else { 2 } },
            0xD9 => { self.regs.PC = self.PopStack(); self.setei = 1; 4 },
            0xDA => { if self.regs.get_FC() { self.regs.PC = self.fetch16(); 4 } else { self.regs.PC += 2; 3 } },
            0xDC => { if self.regs.get_FC() { self.PushStack(self.regs.PC + 2); self.regs.PC = self.fetch16(); 6 } else { self.regs.PC += 2; 3 } },
            0xDE => { let v = self.fetch8(); self.alu_sub(v, true); 2 },
            0xDF => { self.PushStack(self.regs.PC); self.regs.PC = 0x18; 4 },
            0xE0 => { let a = 0xFF00 | self.fetch8() as u16; self.mem.write8(a, self.regs.A); 3 },
            0xE1 => { let v = self.PopStack(); self.regs.set_HL(v); 3 },
            0xE2 => { self.mem.write8(0xFF00 | self.regs.C as u16, self.regs.A); 1 }
            0xE5 => { self.PushStack(self.regs.get_HL()); 4 },
            0xE6 => { let v = self.fetch8(); self.alu_and(v); 2 },
            0xE7 => { self.PushStack(self.regs.PC); self.regs.PC = 0x20; 4 },
            0xE8 => { let s = self.alu_add16imm(self.regs.get_SP()); self.regs.set_SP(s); 4 },
            0xE9 => { self.regs.PC = self.regs.get_HL(); 1 },
            0xEA => { let a = self.fetch16(); self.mem.write8(a, self.regs.A); 4 },
            0xEE => { let v = self.fetch8(); self.alu_xor(v); 2 },
            0xEF => { self.PushStack(self.regs.PC); self.regs.PC = 0x28; 4 },
            0xF0 => { let a = 0xFF00 | self.fetch8() as u16; self.regs.A = self.mem.read8(a); 3 },
            0xF1 => { let v = self.PopStack() & 0xFFF0; self.regs.set_AF(v); 3 },
            0xF2 => { self.regs.A = self.mem.read8(0xFF00 | self.regs.C as u16); 2 },
            0xF3 => { self.setdi = 2; 1 },
            0xF5 => { self.PushStack(self.regs.get_AF()); 4 },
            0xF6 => { let v = self.fetch8(); self.alu_or(v); 2 },
            0xF7 => { self.PushStack(self.regs.PC); self.regs.PC = 0x30; 4 },
            0xF8 => { let r = self.alu_add16imm(self.regs.get_SP()); self.regs.set_HL(r); 3 },
            0xF9 => { self.regs.set_SP(self.regs.get_HL()); 2 },
            0xFA => { let a = self.fetch16(); self.regs.A = self.mem.read8(a); 4 },
            0xFB => { self.setei = 2; 1 },
            0xFE => { let v = self.fetch8(); self.alu_cp(v); 2 },
            0xFF => { self.PushStack(self.regs.PC); self.regs.PC = 0x38; 4 },
            _    => { println!("Unknown opcode {:02X}", code);process::exit(0x0100);}
        };
        self.print_status();
        debug!("----------------------------------------");
        self.total_cyles = self.total_cyles + cur_cycle_count as u64;
        1
    }

    pub fn step_CB(&mut self) -> u8 {
        let code = self.fetch8() as usize;
        match code {
            0x00 => { self.regs.B = self.alu_rlc(self.regs.B); 2 },
            0x01 => { self.regs.C = self.alu_rlc(self.regs.C); 2 },
            0x02 => { self.regs.D = self.alu_rlc(self.regs.D); 2 },
            0x03 => { self.regs.E = self.alu_rlc(self.regs.E); 2 },
            0x04 => { self.regs.H = self.alu_rlc(self.regs.H); 2 },
            0x05 => { self.regs.L = self.alu_rlc(self.regs.L); 2 },
            0x06 => { let a = self.regs.get_HL(); let v = self.mem.read8(a); let v2 = self.alu_rlc(v); self.mem.write8(a, v2); 4 },
            0x07 => { self.regs.A = self.alu_rlc(self.regs.A); 2 },
            0x08 => { self.regs.B = self.alu_rrc(self.regs.B); 2 },
            0x09 => { self.regs.C = self.alu_rrc(self.regs.C); 2 },
            0x0A => { self.regs.D = self.alu_rrc(self.regs.D); 2 },
            0x0B => { self.regs.E = self.alu_rrc(self.regs.E); 2 },
            0x0C => { self.regs.H = self.alu_rrc(self.regs.H); 2 },
            0x0D => { self.regs.L = self.alu_rrc(self.regs.L); 2 },
            0x0E => { let a = self.regs.get_HL(); let v = self.mem.read8(a); let v2 = self.alu_rrc(v); self.mem.write8(a, v2); 4 },
            0x0F => { self.regs.A = self.alu_rrc(self.regs.A); 2 },
            0x10 => { self.regs.B = self.alu_rl(self.regs.B); 2 },
            0x11 => { self.regs.C = self.alu_rl(self.regs.C); 2 },
            0x12 => { self.regs.D = self.alu_rl(self.regs.D); 2 },
            0x13 => { self.regs.E = self.alu_rl(self.regs.E); 2 },
            0x14 => { self.regs.H = self.alu_rl(self.regs.H); 2 },
            0x15 => { self.regs.L = self.alu_rl(self.regs.L); 2 },
            0x16 => { let a = self.regs.get_HL(); let v = self.mem.read8(a); let v2 = self.alu_rl(v); self.mem.write8(a, v2); 4 },
            0x17 => { self.regs.A = self.alu_rl(self.regs.A); 2 },
            0x18 => { self.regs.B = self.alu_rr(self.regs.B); 2 },
            0x19 => { self.regs.C = self.alu_rr(self.regs.C); 2 },
            0x1A => { self.regs.D = self.alu_rr(self.regs.D); 2 },
            0x1B => { self.regs.E = self.alu_rr(self.regs.E); 2 },
            0x1C => { self.regs.H = self.alu_rr(self.regs.H); 2 },
            0x1D => { self.regs.L = self.alu_rr(self.regs.L); 2 },
            0x1E => { let a = self.regs.get_HL(); let v = self.mem.read8(a); let v2 = self.alu_rr(v); self.mem.write8(a, v2); 4 },
            0x1F => { self.regs.A = self.alu_rr(self.regs.A); 2 },
            0x20 => { self.regs.B = self.alu_sla(self.regs.B); 2 },
            0x21 => { self.regs.C = self.alu_sla(self.regs.C); 2 },
            0x22 => { self.regs.D = self.alu_sla(self.regs.D); 2 },
            0x23 => { self.regs.E = self.alu_sla(self.regs.E); 2 },
            0x24 => { self.regs.H = self.alu_sla(self.regs.H); 2 },
            0x25 => { self.regs.L = self.alu_sla(self.regs.L); 2 },
            0x26 => { let a = self.regs.get_HL(); let v = self.mem.read8(a); let v2 = self.alu_sla(v); self.mem.write8(a, v2); 4 },
            0x27 => { self.regs.A = self.alu_sla(self.regs.A); 2 },
            0x28 => { self.regs.B = self.alu_sra(self.regs.B); 2 },
            0x29 => { self.regs.C = self.alu_sra(self.regs.C); 2 },
            0x2A => { self.regs.D = self.alu_sra(self.regs.D); 2 },
            0x2B => { self.regs.E = self.alu_sra(self.regs.E); 2 },
            0x2C => { self.regs.H = self.alu_sra(self.regs.H); 2 },
            0x2D => { self.regs.L = self.alu_sra(self.regs.L); 2 },
            0x2E => { let a = self.regs.get_HL(); let v = self.mem.read8(a); let v2 = self.alu_sra(v); self.mem.write8(a, v2); 4 },
            0x2F => { self.regs.A = self.alu_sra(self.regs.A); 2 },
            0x30 => { self.regs.B = self.alu_swap(self.regs.B); 2 },
            0x31 => { self.regs.C = self.alu_swap(self.regs.C); 2 },
            0x32 => { self.regs.D = self.alu_swap(self.regs.D); 2 },
            0x33 => { self.regs.E = self.alu_swap(self.regs.E); 2 },
            0x34 => { self.regs.H = self.alu_swap(self.regs.H); 2 },
            0x35 => { self.regs.L = self.alu_swap(self.regs.L); 2 },
            0x36 => { let a = self.regs.get_HL(); let v = self.mem.read8(a); let v2 = self.alu_swap(v); self.mem.write8(a, v2); 4 },
            0x37 => { self.regs.A = self.alu_swap(self.regs.A); 2 },
            0x38 => { self.regs.B = self.alu_srl(self.regs.B); 2 },
            0x39 => { self.regs.C = self.alu_srl(self.regs.C); 2 },
            0x3A => { self.regs.D = self.alu_srl(self.regs.D); 2 },
            0x3B => { self.regs.E = self.alu_srl(self.regs.E); 2 },
            0x3C => { self.regs.H = self.alu_srl(self.regs.H); 2 },
            0x3D => { self.regs.L = self.alu_srl(self.regs.L); 2 },
            0x3E => { let a = self.regs.get_HL(); let v = self.mem.read8(a); let v2 = self.alu_srl(v); self.mem.write8(a, v2); 4 },
            0x3F => { self.regs.A = self.alu_srl(self.regs.A); 2 },
            0x40 => { self.alu_bit(self.regs.B, 0); 2 },
            0x41 => { self.alu_bit(self.regs.C, 0); 2 },
            0x42 => { self.alu_bit(self.regs.D, 0); 2 },
            0x43 => { self.alu_bit(self.regs.E, 0); 2 },
            0x44 => { self.alu_bit(self.regs.H, 0); 2 },
            0x45 => { self.alu_bit(self.regs.L, 0); 2 },
            0x46 => { let v = self.mem.read8(self.regs.get_HL()); self.alu_bit(v, 0); 3 },
            0x47 => { self.alu_bit(self.regs.A, 0); 2 },
            0x48 => { self.alu_bit(self.regs.B, 1); 2 },
            0x49 => { self.alu_bit(self.regs.C, 1); 2 },
            0x4A => { self.alu_bit(self.regs.D, 1); 2 },
            0x4B => { self.alu_bit(self.regs.E, 1); 2 },
            0x4C => { self.alu_bit(self.regs.H, 1); 2 },
            0x4D => { self.alu_bit(self.regs.L, 1); 2 },
            0x4E => { let v = self.mem.read8(self.regs.get_HL()); self.alu_bit(v, 1); 3 },
            0x4F => { self.alu_bit(self.regs.A, 1); 2 },
            0x50 => { self.alu_bit(self.regs.B, 2); 2 },
            0x51 => { self.alu_bit(self.regs.C, 2); 2 },
            0x52 => { self.alu_bit(self.regs.D, 2); 2 },
            0x53 => { self.alu_bit(self.regs.E, 2); 2 },
            0x54 => { self.alu_bit(self.regs.H, 2); 2 },
            0x55 => { self.alu_bit(self.regs.L, 2); 2 },
            0x56 => { let v = self.mem.read8(self.regs.get_HL()); self.alu_bit(v, 2); 3 },
            0x57 => { self.alu_bit(self.regs.A, 2); 2 },
            0x58 => { self.alu_bit(self.regs.B, 3); 2 },
            0x59 => { self.alu_bit(self.regs.C, 3); 2 },
            0x5A => { self.alu_bit(self.regs.D, 3); 2 },
            0x5B => { self.alu_bit(self.regs.E, 3); 2 },
            0x5C => { self.alu_bit(self.regs.H, 3); 2 },
            0x5D => { self.alu_bit(self.regs.L, 3); 2 },
            0x5E => { let v = self.mem.read8(self.regs.get_HL()); self.alu_bit(v, 3); 3 },
            0x5F => { self.alu_bit(self.regs.A, 3); 2 },
            0x60 => { self.alu_bit(self.regs.B, 4); 2 },
            0x61 => { self.alu_bit(self.regs.C, 4); 2 },
            0x62 => { self.alu_bit(self.regs.D, 4); 2 },
            0x63 => { self.alu_bit(self.regs.E, 4); 2 },
            0x64 => { self.alu_bit(self.regs.H, 4); 2 },
            0x65 => { self.alu_bit(self.regs.L, 4); 2 },
            0x66 => { let v = self.mem.read8(self.regs.get_HL()); self.alu_bit(v, 4); 3 },
            0x67 => { self.alu_bit(self.regs.A, 4); 2 },
            0x68 => { self.alu_bit(self.regs.B, 5); 2 },
            0x69 => { self.alu_bit(self.regs.C, 5); 2 },
            0x6A => { self.alu_bit(self.regs.D, 5); 2 },
            0x6B => { self.alu_bit(self.regs.E, 5); 2 },
            0x6C => { self.alu_bit(self.regs.H, 5); 2 },
            0x6D => { self.alu_bit(self.regs.L, 5); 2 },
            0x6E => { let v = self.mem.read8(self.regs.get_HL()); self.alu_bit(v, 5); 3 },
            0x6F => { self.alu_bit(self.regs.A, 5); 2 },
            0x70 => { self.alu_bit(self.regs.B, 6); 2 },
            0x71 => { self.alu_bit(self.regs.C, 6); 2 },
            0x72 => { self.alu_bit(self.regs.D, 6); 2 },
            0x73 => { self.alu_bit(self.regs.E, 6); 2 },
            0x74 => { self.alu_bit(self.regs.H, 6); 2 },
            0x75 => { self.alu_bit(self.regs.L, 6); 2 },
            0x76 => { let v = self.mem.read8(self.regs.get_HL()); self.alu_bit(v, 6); 3 },
            0x77 => { self.alu_bit(self.regs.A, 6); 2 },
            0x78 => { self.alu_bit(self.regs.B, 7); 2 },
            0x79 => { self.alu_bit(self.regs.C, 7); 2 },
            0x7A => { self.alu_bit(self.regs.D, 7); 2 },
            0x7B => { self.alu_bit(self.regs.E, 7); 2 },
            0x7C => { self.alu_bit(self.regs.H, 7); 2 },
            0x7D => { self.alu_bit(self.regs.L, 7); 2 },
            0x7E => { let v = self.mem.read8(self.regs.get_HL()); self.alu_bit(v, 7); 3 },
            0x7F => { self.alu_bit(self.regs.A, 7); 2 },
            0x80 => { self.regs.B = self.regs.B & !(1 << 0); 2 },
            0x81 => { self.regs.C = self.regs.C & !(1 << 0); 2 },
            0x82 => { self.regs.D = self.regs.D & !(1 << 0); 2 },
            0x83 => { self.regs.E = self.regs.E & !(1 << 0); 2 },
            0x84 => { self.regs.H = self.regs.H & !(1 << 0); 2 },
            0x85 => { self.regs.L = self.regs.L & !(1 << 0); 2 },
            0x86 => { let a = self.regs.get_HL(); let v = self.mem.read8(a) & !(1 << 0); self.mem.write8(a, v); 4 },
            0x87 => { self.regs.A = self.regs.A & !(1 << 0); 2 },
            0x88 => { self.regs.B = self.regs.B & !(1 << 1); 2 },
            0x89 => { self.regs.C = self.regs.C & !(1 << 1); 2 },
            0x8A => { self.regs.D = self.regs.D & !(1 << 1); 2 },
            0x8B => { self.regs.E = self.regs.E & !(1 << 1); 2 },
            0x8C => { self.regs.H = self.regs.H & !(1 << 1); 2 },
            0x8D => { self.regs.L = self.regs.L & !(1 << 1); 2 },
            0x8E => { let a = self.regs.get_HL(); let v = self.mem.read8(a) & !(1 << 1); self.mem.write8(a, v); 4 },
            0x8F => { self.regs.A = self.regs.A & !(1 << 1); 2 },
            0x90 => { self.regs.B = self.regs.B & !(1 << 2); 2 },
            0x91 => { self.regs.C = self.regs.C & !(1 << 2); 2 },
            0x92 => { self.regs.D = self.regs.D & !(1 << 2); 2 },
            0x93 => { self.regs.E = self.regs.E & !(1 << 2); 2 },
            0x94 => { self.regs.H = self.regs.H & !(1 << 2); 2 },
            0x95 => { self.regs.L = self.regs.L & !(1 << 2); 2 },
            0x96 => { let a = self.regs.get_HL(); let v = self.mem.read8(a) & !(1 << 2); self.mem.write8(a, v); 4 },
            0x97 => { self.regs.A = self.regs.A & !(1 << 2); 2 },
            0x98 => { self.regs.B = self.regs.B & !(1 << 3); 2 },
            0x99 => { self.regs.C = self.regs.C & !(1 << 3); 2 },
            0x9A => { self.regs.D = self.regs.D & !(1 << 3); 2 },
            0x9B => { self.regs.E = self.regs.E & !(1 << 3); 2 },
            0x9C => { self.regs.H = self.regs.H & !(1 << 3); 2 },
            0x9D => { self.regs.L = self.regs.L & !(1 << 3); 2 },
            0x9E => { let a = self.regs.get_HL(); let v = self.mem.read8(a) & !(1 << 3); self.mem.write8(a, v); 4 },
            0x9F => { self.regs.A = self.regs.A & !(1 << 3); 2 },
            0xA0 => { self.regs.B = self.regs.B & !(1 << 4); 2 },
            0xA1 => { self.regs.C = self.regs.C & !(1 << 4); 2 },
            0xA2 => { self.regs.D = self.regs.D & !(1 << 4); 2 },
            0xA3 => { self.regs.E = self.regs.E & !(1 << 4); 2 },
            0xA4 => { self.regs.H = self.regs.H & !(1 << 4); 2 },
            0xA5 => { self.regs.L = self.regs.L & !(1 << 4); 2 },
            0xA6 => { let a = self.regs.get_HL(); let v = self.mem.read8(a) & !(1 << 4); self.mem.write8(a, v); 4 },
            0xA7 => { self.regs.A = self.regs.A & !(1 << 4); 2 },
            0xA8 => { self.regs.B = self.regs.B & !(1 << 5); 2 },
            0xA9 => { self.regs.C = self.regs.C & !(1 << 5); 2 },
            0xAA => { self.regs.D = self.regs.D & !(1 << 5); 2 },
            0xAB => { self.regs.E = self.regs.E & !(1 << 5); 2 },
            0xAC => { self.regs.H = self.regs.H & !(1 << 5); 2 },
            0xAD => { self.regs.L = self.regs.L & !(1 << 5); 2 },
            0xAE => { let a = self.regs.get_HL(); let v = self.mem.read8(a) & !(1 << 5); self.mem.write8(a, v); 4 },
            0xAF => { self.regs.A = self.regs.A & !(1 << 5); 2 },
            0xB0 => { self.regs.B = self.regs.B & !(1 << 6); 2 },
            0xB1 => { self.regs.C = self.regs.C & !(1 << 6); 2 },
            0xB2 => { self.regs.D = self.regs.D & !(1 << 6); 2 },
            0xB3 => { self.regs.E = self.regs.E & !(1 << 6); 2 },
            0xB4 => { self.regs.H = self.regs.H & !(1 << 6); 2 },
            0xB5 => { self.regs.L = self.regs.L & !(1 << 6); 2 },
            0xB6 => { let a = self.regs.get_HL(); let v = self.mem.read8(a) & !(1 << 6); self.mem.write8(a, v); 4 },
            0xB7 => { self.regs.A = self.regs.A & !(1 << 6); 2 },
            0xB8 => { self.regs.B = self.regs.B & !(1 << 7); 2 },
            0xB9 => { self.regs.C = self.regs.C & !(1 << 7); 2 },
            0xBA => { self.regs.D = self.regs.D & !(1 << 7); 2 },
            0xBB => { self.regs.E = self.regs.E & !(1 << 7); 2 },
            0xBC => { self.regs.H = self.regs.H & !(1 << 7); 2 },
            0xBD => { self.regs.L = self.regs.L & !(1 << 7); 2 },
            0xBE => { let a = self.regs.get_HL(); let v = self.mem.read8(a) & !(1 << 7); self.mem.write8(a, v); 4 },
            0xBF => { self.regs.A = self.regs.A & !(1 << 7); 2 },
            0xC0 => { self.regs.B = self.regs.B | (1 << 0); 2 },
            0xC1 => { self.regs.C = self.regs.C | (1 << 0); 2 },
            0xC2 => { self.regs.D = self.regs.D | (1 << 0); 2 },
            0xC3 => { self.regs.E = self.regs.E | (1 << 0); 2 },
            0xC4 => { self.regs.H = self.regs.H | (1 << 0); 2 },
            0xC5 => { self.regs.L = self.regs.L | (1 << 0); 2 },
            0xC6 => { let a = self.regs.get_HL(); let v = self.mem.read8(a) | (1 << 0); self.mem.write8(a, v); 4 },
            0xC7 => { self.regs.A = self.regs.A | (1 << 0); 2 },
            0xC8 => { self.regs.B = self.regs.B | (1 << 1); 2 },
            0xC9 => { self.regs.C = self.regs.C | (1 << 1); 2 },
            0xCA => { self.regs.D = self.regs.D | (1 << 1); 2 },
            0xCB => { self.regs.E = self.regs.E | (1 << 1); 2 },
            0xCC => { self.regs.H = self.regs.H | (1 << 1); 2 },
            0xCD => { self.regs.L = self.regs.L | (1 << 1); 2 },
            0xCE => { let a = self.regs.get_HL(); let v = self.mem.read8(a) | (1 << 1); self.mem.write8(a, v); 4 },
            0xCF => { self.regs.A = self.regs.A | (1 << 1); 2 },
            0xD0 => { self.regs.B = self.regs.B | (1 << 2); 2 },
            0xD1 => { self.regs.C = self.regs.C | (1 << 2); 2 },
            0xD2 => { self.regs.D = self.regs.D | (1 << 2); 2 },
            0xD3 => { self.regs.E = self.regs.E | (1 << 2); 2 },
            0xD4 => { self.regs.H = self.regs.H | (1 << 2); 2 },
            0xD5 => { self.regs.L = self.regs.L | (1 << 2); 2 },
            0xD6 => { let a = self.regs.get_HL(); let v = self.mem.read8(a) | (1 << 2); self.mem.write8(a, v); 4 },
            0xD7 => { self.regs.A = self.regs.A | (1 << 2); 2 },
            0xD8 => { self.regs.B = self.regs.B | (1 << 3); 2 },
            0xD9 => { self.regs.C = self.regs.C | (1 << 3); 2 },
            0xDA => { self.regs.D = self.regs.D | (1 << 3); 2 },
            0xDB => { self.regs.E = self.regs.E | (1 << 3); 2 },
            0xDC => { self.regs.H = self.regs.H | (1 << 3); 2 },
            0xDD => { self.regs.L = self.regs.L | (1 << 3); 2 },
            0xDE => { let a = self.regs.get_HL(); let v = self.mem.read8(a) | (1 << 3); self.mem.write8(a, v); 4 },
            0xDF => { self.regs.A = self.regs.A | (1 << 3); 2 },
            0xE0 => { self.regs.B = self.regs.B | (1 << 4); 2 },
            0xE1 => { self.regs.C = self.regs.C | (1 << 4); 2 },
            0xE2 => { self.regs.D = self.regs.D | (1 << 4); 2 },
            0xE3 => { self.regs.E = self.regs.E | (1 << 4); 2 },
            0xE4 => { self.regs.H = self.regs.H | (1 << 4); 2 },
            0xE5 => { self.regs.L = self.regs.L | (1 << 4); 2 },
            0xE6 => { let a = self.regs.get_HL(); let v = self.mem.read8(a) | (1 << 4); self.mem.write8(a, v); 4 },
            0xE7 => { self.regs.A = self.regs.A | (1 << 4); 2 },
            0xE8 => { self.regs.B = self.regs.B | (1 << 5); 2 },
            0xE9 => { self.regs.C = self.regs.C | (1 << 5); 2 },
            0xEA => { self.regs.D = self.regs.D | (1 << 5); 2 },
            0xEB => { self.regs.E = self.regs.E | (1 << 5); 2 },
            0xEC => { self.regs.H = self.regs.H | (1 << 5); 2 },
            0xED => { self.regs.L = self.regs.L | (1 << 5); 2 },
            0xEE => { let a = self.regs.get_HL(); let v = self.mem.read8(a) | (1 << 5); self.mem.write8(a, v); 4 },
            0xEF => { self.regs.A = self.regs.A | (1 << 5); 2 },
            0xF0 => { self.regs.B = self.regs.B | (1 << 6); 2 },
            0xF1 => { self.regs.C = self.regs.C | (1 << 6); 2 },
            0xF2 => { self.regs.D = self.regs.D | (1 << 6); 2 },
            0xF3 => { self.regs.E = self.regs.E | (1 << 6); 2 },
            0xF4 => { self.regs.H = self.regs.H | (1 << 6); 2 },
            0xF5 => { self.regs.L = self.regs.L | (1 << 6); 2 },
            0xF6 => { let a = self.regs.get_HL(); let v = self.mem.read8(a) | (1 << 6); self.mem.write8(a, v); 4 },
            0xF7 => { self.regs.A = self.regs.A | (1 << 6); 2 },
            0xF8 => { self.regs.B = self.regs.B | (1 << 7); 2 },
            0xF9 => { self.regs.C = self.regs.C | (1 << 7); 2 },
            0xFA => { self.regs.D = self.regs.D | (1 << 7); 2 },
            0xFB => { self.regs.E = self.regs.E | (1 << 7); 2 },
            0xFC => { self.regs.H = self.regs.H | (1 << 7); 2 },
            0xFD => { self.regs.L = self.regs.L | (1 << 7); 2 },
            0xFE => { let a = self.regs.get_HL(); let v = self.mem.read8(a) | (1 << 7); self.mem.write8(a, v); 4 },
            0xFF => { self.regs.A = self.regs.A | (1 << 7); 2 },

            _    => { println!("Unknown extented opcode {:02X}", code);process::exit(0x0100);}
        }
    }


    fn stop(&mut self) {
        println!("STOP");
    }

    fn alu_add(&mut self, b: u8, usec: bool) {
        let c = if usec && self.regs.get_FC() { 1 } else { 0 };
        let a = self.regs.A;
        let r = a.wrapping_add(b).wrapping_add(c);
        self.regs.set_FZ(r == 0);
        self.regs.set_FH((a & 0xF) + (b & 0xF) + c > 0xF);
        self.regs.set_FN(false);
        self.regs.set_FC((a as u16) + (b as u16) + (c as u16) > 0xFF);
        self.regs.A = r;
    }

    fn alu_sub(&mut self, b: u8, usec: bool) {
        let c = if usec && self.regs.get_FC() { 1 } else { 0 };
        let a = self.regs.A;
        let r = a.wrapping_sub(b).wrapping_sub(c);
        self.regs.set_FZ(r == 0);
        self.regs.set_FH((a & 0x0F) < (b & 0x0F) + c);
        self.regs.set_FN(true);
        self.regs.set_FC((a as u16) < (b as u16) + (c as u16));
        self.regs.A = r;
    }

    fn alu_and(&mut self, b: u8) {
        let r = self.regs.A & b;
        self.regs.set_FZ(r == 0);
        self.regs.set_FH(true);
        self.regs.set_FC(false);
        self.regs.set_FN(false);
        self.regs.A = r;
    }

    fn alu_or(&mut self, b: u8) {
        let r = self.regs.A | b;
        self.regs.set_FZ(r == 0);
        self.regs.set_FC(false);
        self.regs.set_FH(false);
        self.regs.set_FN(false);
        self.regs.A = r;
    }

    fn alu_xor(&mut self, b: u8) {
        let r = self.regs.A ^ b;
        self.regs.set_FZ(r == 0);
        self.regs.set_FC(false);
        self.regs.set_FH(false);
        self.regs.set_FN(false);
        self.regs.A = r;
    }

    fn alu_cp(&mut self, b: u8) {
        let r = self.regs.A;
        self.alu_sub(b, false);
        self.regs.A = r;
    }

    fn alu_inc(&mut self, a: u8) -> u8 {
        let r = a.wrapping_add(1);
        self.regs.set_FZ(r == 0);
        self.regs.set_FH((a & 0x0F) + 1 > 0x0F);
        self.regs.set_FN(false);
        return r
    }

    fn alu_dec(&mut self, a: u8) -> u8 {
        let r = a.wrapping_sub(1);
        self.regs.set_FZ(r == 0);
        self.regs.set_FH((a & 0x0F) == 0);
        self.regs.set_FN(true);
        return r
    }

    fn alu_add16(&mut self, b: u16) {
        let a = self.regs.get_HL();
        let r = a.wrapping_add(b);
        self.regs.set_FH((a & 0x07FF) + (b & 0x07FF) > 0x07FF);
        self.regs.set_FN(false);
        self.regs.set_FC(a > 0xFFFF - b);
        self.regs.set_HL(r);
    }

    fn alu_add16imm(&mut self, a: u16) -> u16 {
        let b = self.fetch8() as i8 as i16 as u16;
        self.regs.set_FN(false);
        self.regs.set_FZ(false);
        self.regs.set_FH((a & 0x000F) + (b & 0x000F) > 0x000F);
        self.regs.set_FC((a & 0x00FF) + (b & 0x00FF) > 0x00FF);
        return a.wrapping_add(b)
    }

    fn alu_swap(&mut self, a: u8) -> u8 {
        self.regs.set_FZ(a == 0);
        self.regs.set_FC(false);
        self.regs.set_FH(false);
        self.regs.set_FN(false);
        (a >> 4) | (a << 4)
    }

    fn alu_srflagupdate(&mut self, r: u8, c: bool) {
        self.regs.set_FH(false);
        self.regs.set_FN(false);
        self.regs.set_FZ(r == 0);
        self.regs.set_FC(c);
    }

    fn alu_rlc(&mut self, a: u8) -> u8 {
        let c = a & 0x80 == 0x80;
        let r = (a << 1) | (if c { 1 } else { 0 });
        self.alu_srflagupdate(r, c);
        return r
    }

    fn alu_rl(&mut self, a: u8) -> u8 {
        let c = a & 0x80 == 0x80;
        let r = (a << 1) | (if self.regs.get_FC() { 1 } else { 0 });
        self.alu_srflagupdate(r, c);
        return r
    }

    fn alu_rrc(&mut self, a: u8) -> u8 {
        let c = a & 0x01 == 0x01;
        let r = (a >> 1) | (if c { 0x80 } else { 0 });
        self.alu_srflagupdate(r, c);
        return r
    }

    fn alu_rr(&mut self, a: u8) -> u8 {
        let c = a & 0x01 == 0x01;
        let r = (a >> 1) | (if self.regs.get_FC() { 0x80 } else { 0 });
        self.alu_srflagupdate(r, c);
        return r
    }

    fn alu_sla(&mut self, a: u8) -> u8 {
        let c = a & 0x80 == 0x80;
        let r = a << 1;
        self.alu_srflagupdate(r, c);
        return r
    }

    fn alu_sra(&mut self, a: u8) -> u8 {
        let c = a & 0x01 == 0x01;
        let r = (a >> 1) | (a & 0x80);
        self.alu_srflagupdate(r, c);
        return r
    }

    fn alu_srl(&mut self, a: u8) -> u8 {
        let c = a & 0x01 == 0x01;
        let r = a >> 1;
        self.alu_srflagupdate(r, c);
        return r
    }

    fn alu_bit(&mut self, a: u8, b: u8) {
        let r = a & (1 << (b as u32)) == 0;
        self.regs.set_FN(false);
        self.regs.set_FH(true);
        self.regs.set_FZ(r);
    }

    fn alu_daa(&mut self) {
        let mut a = self.regs.A;
        let mut adjust = if self.regs.get_FC() { 0x60 } else { 0x00 };
        if self.regs.get_FH() { adjust |= 0x06; };
        if !self.regs.get_FN() {
            if a & 0x0F > 0x09 { adjust |= 0x06; };
            if a > 0x99 { adjust |= 0x60; };
            a = a.wrapping_add(adjust);
        } else {
            a = a.wrapping_sub(adjust);
        }

        self.regs.set_FC(adjust >= 0x60);
        self.regs.set_FH(false);
        self.regs.set_FZ(a == 0);
        self.regs.A = a;
    }

    fn cpu_jr(&mut self) {
        let n = self.fetch8() as i8;
        self.regs.set_PC(((self.regs.get_PC() as u32 as i32) + (n as i32)) as u16);
    }
}
