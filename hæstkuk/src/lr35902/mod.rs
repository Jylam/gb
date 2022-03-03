// Sharp LR35902 CPU emulator
#![allow(non_snake_case)]
#![allow(dead_code)]

use std::thread::sleep;
use std::time::Duration;
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
    C: u8,
    D: u8,
    E: u8,
    F: u8,
    H: u8,
    L: u8,
    PC: u16,
    SP: u16,
    I: bool,
}
#[allow(dead_code)]
impl Registers {
    fn get_AF(self) -> u16 {
        ((self.A as u16)<<8) | ((self.F as u16)&0xF0)
    }
    fn set_AF(&mut self, v: u16) {
        self.A = ((v&0xFF00)>>8) as u8;
        self.F = (v&0x00F0) as u8;
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
        self.H = (v>>8) as u8;
        self.L = (v&0x00FF) as u8;
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
    fn set_FZ(&mut self, v: bool) {
        if v {
            self.F |= 0b1000_0000;
        } else {
            self.F &= 0b0111_1111;
        }
    }
    fn get_FZ(&mut self) -> bool{
        (((self.F&(0b1000_0000))>>7)==1) as bool
    }
    fn set_FN(&mut self, v: bool) {
        if v {
            self.F |= 0b0100_0000
        }  else {
            self.F &= 0b1011_1111
        }
    }
    fn get_FN(&mut self) -> bool{
        (((self.F&(0b0100_0000))>>6)==1) as bool
    }
    fn set_FH(&mut self, v: bool) {
        if v {
            self.F |= 0b0010_0000
        } else {
            self.F &= 0b1101_1111
        }
    }
    fn get_FH(&mut self) -> bool{
        (((self.F&(0b0010_0000))>>5)==1) as bool
    }
    fn set_FC(&mut self, v: bool) {
        if v {
            self.F |= 0b0001_0000
        } else {
            self.F &= 0b1110_1111
        }
    }
    fn get_FC(&mut self) -> bool{
        (((self.F&(0b0001_0000))>>4)==1) as bool
    }



}

pub struct Cpu<'a> {
    pub mem: mem::Mem<'a>,
    regs: Registers,
    total_cyles: u64,
    opcodes: Vec<Opcode>,
    alt_opcodes: Vec<Opcode>,
}

pub fn addr16(cpu: &mut Cpu) -> u16 {
    cpu.mem.read16(cpu.regs.get_PC()+1)
}
pub fn imm16(cpu: &mut Cpu) -> u16 {
    cpu.mem.read16(cpu.regs.get_PC()+1)
}
pub fn imm8(cpu: &mut Cpu) -> u8 {
    cpu.mem.read8(cpu.regs.get_PC()+1)
}

pub fn UNK(cpu: &mut Cpu) {
    println!("*** Unknow instruction at {:04X}", cpu.regs.get_PC());
    cpu.print_status();
    sleep(Duration::from_secs(5));
    process::exit(3);
}
pub fn ALTUNK(cpu: &mut Cpu) {
    println!("*** Unknow alternative instruction [{:02X}] at {:04X}", cpu.mem.read8(cpu.regs.get_PC()+1), cpu.regs.get_PC());
    cpu.print_status();
    process::exit(3);
}
pub fn alu_sub(cpu: &mut Cpu, b: u8, carry: bool) {
    let c = if carry && cpu.regs.get_FC() { 1 } else { 0 };
    let a = cpu.regs.A;
    let r = a.wrapping_sub(b).wrapping_sub(c);
    cpu.regs.set_FZ(r == 0);
    cpu.regs.set_FH((a & 0xF) < (b & 0xF) + c);
    cpu.regs.set_FN(true);
    cpu.regs.set_FC((a as u16) < (b as u16) + (c as u16));
    cpu.regs.A = r;
}
pub fn alu_dec(cpu: &mut Cpu, a: u8) -> u8 {
    let r = a.wrapping_sub(1);
    cpu.regs.set_FZ( r == 0);
    cpu.regs.set_FH( (a & 0x0F) == 0);
    cpu.regs.set_FN( true);
    return r
}
pub fn alu_add(cpu: &mut Cpu, b: u8, carry: bool) {
    let c = if carry && cpu.regs.get_FC() { 1 } else { 0 };
    let a = cpu.regs.A;
    let r = a.wrapping_add(b).wrapping_add(c);
    cpu.regs.set_FZ(r == 0);
    cpu.regs.set_FH((a & 0xF) + (b & 0xF) + c > 0xF);
    cpu.regs.set_FN(false);
    cpu.regs.set_FC((a as u16) + (b as u16) + (c as u16) > 0xFF);
    cpu.regs.A = r;
}

pub fn alu_inc(cpu: &mut Cpu, a: u8) -> u8 {
    let r = a.wrapping_add(1);
    cpu.regs.set_FZ( r == 0);
    cpu.regs.set_FH( (a & 0x0F) + 1 > 0x0F);
    cpu.regs.set_FN( false);
    return r
}
pub fn alu_add16(cpu: &mut Cpu, b: u16) {
    let a = cpu.regs.get_HL();
    let r = a.wrapping_add(b);
//    cpu.regs.set_FH((a & 0x07FF) + (b & 0x07FF) > 0x07FF);
    cpu.regs.set_FH(((cpu.regs.get_HL()&0xFFF) + (b&0xFFF)) > 0xFFF);
    cpu.regs.set_FN(false);
    cpu.regs.set_FC(a > 0xFFFF - b);
    cpu.regs.set_HL(r);
}

fn alu_add16imm(cpu: &mut Cpu, a: u16) -> u16 {
    let b = imm8(cpu) as i8 as i16 as u16;
    cpu.regs.set_FN(false);
    cpu.regs.set_FZ(false);
    cpu.regs.set_FH((a & 0x000F) + (b & 0x000F) > 0x000F);
    cpu.regs.set_FC((a & 0x00FF) + (b & 0x00FF) > 0x00FF);
    return a.wrapping_add(b)
}

fn alu_daa(cpu: &mut Cpu) {
    let mut a = cpu.regs.A;
    let mut adjust = if cpu.regs.get_FC() { 0x60 } else { 0x00 };
    if cpu.regs.get_FH() { adjust |= 0x06; };
    if !cpu.regs.get_FN() {
        if a & 0x0F > 0x09 { adjust |= 0x06; };
        if a > 0x99 { adjust |= 0x60; };
        a = a.wrapping_add(adjust);
    } else {
        a = a.wrapping_sub(adjust);
    }

    cpu.regs.set_FC(adjust >= 0x60);
    cpu.regs.set_FH(false);
    cpu.regs.set_FZ(a == 0);
    cpu.regs.A = a;
}
pub fn alu_cp(cpu: &mut Cpu, b: u8) {
    let r = cpu.regs.A;
    alu_sub(cpu, b, false);
    cpu.regs.A = r;
}

fn alu_and(cpu: &mut Cpu, b: u8) {
    let r = cpu.regs.A & b;
    cpu.regs.set_FZ(r == 0);
    cpu.regs.set_FH(true);
    cpu.regs.set_FC(false);
    cpu.regs.set_FN(false);
    cpu.regs.A = r;
}

fn alu_or(cpu: &mut Cpu, b: u8) {
    let r = cpu.regs.A | b;
    cpu.regs.set_FZ(r == 0);
    cpu.regs.set_FC(false);
    cpu.regs.set_FH(false);
    cpu.regs.set_FN(false);
    cpu.regs.A = r;
}

fn alu_xor(cpu: &mut Cpu, b: u8) {
    let r = cpu.regs.A ^ b;
    cpu.regs.set_FZ(r == 0);
    cpu.regs.set_FC(false);
    cpu.regs.set_FH(false);
    cpu.regs.set_FN(false);
    cpu.regs.A = r;
}

fn alu_rr(cpu: &mut Cpu, a: u8) -> u8 {
    let c = a & 0x01 == 0x01;
    let r = (a >> 1) | (if cpu.regs.get_FC() { 0x80 } else { 0 });
    alu_srflagupdate(cpu, r, c);
    r
}

fn alu_rrc(cpu: &mut Cpu, a: u8) -> u8 {
    let c = a & 0x01 == 0x01;
    let r = (a >> 1) | (if c { 0x80 } else { 0 });
    alu_srflagupdate(cpu, r, c);
    return r
}

fn alu_srflagupdate(cpu: &mut Cpu, r: u8, c: bool) {
    cpu.regs.set_FH(false);
    cpu.regs.set_FN(false);
    cpu.regs.set_FZ(r == 0);
    cpu.regs.set_FC(c);
}


fn alu_sra(cpu: &mut Cpu, a: u8) -> u8 {
    let c = a & 0x01 == 0x01;
    let r = a >> 1  | (a & 0x80) ;
    alu_srflagupdate(cpu, r, c);
    r
}
fn alu_srl(cpu: &mut Cpu, a: u8) -> u8 {
    let c = a & 0x01 == 0x01;
    let r = a >> 1;
    alu_srflagupdate(cpu, r, c);
    r
}
fn alu_rl(cpu: &mut Cpu, a: u8) -> u8 {
    let c = a & 0x80 == 0x80;
    let r = (a << 1) | (if cpu.regs.get_FC() { 1 } else { 0 });
    alu_srflagupdate(cpu, r, c);
    return r
}


fn alu_bit(cpu: &mut Cpu, a: u8, b: u8) {
    let r = a & (1 << (b as u32)) == 0;
    cpu.regs.set_FN(false);
    cpu.regs.set_FH(true);
    cpu.regs.set_FZ(r);
}

fn cpu_jr(cpu: &mut Cpu) {
    let n = imm8(cpu) as i8;
    cpu.regs.set_PC((((cpu.regs.get_PC()+2) as u32 as i32) + (n as i32)) as u16);
}

pub fn XORd8(cpu: &mut Cpu) {
    let imm = imm8(cpu);
    alu_xor(cpu, imm);
    debug!("XOR {:02X}", imm);
}
pub fn XOR_hl(cpu: &mut Cpu) {
    let hl = cpu.mem.read8(cpu.regs.get_HL());
    alu_xor(cpu, hl);
    debug!("XOR A, [HL]");
}
pub fn ORhl(cpu: &mut Cpu) {
    let v = cpu.mem.read8(cpu.regs.get_HL());
    alu_or(cpu, v);
    debug!("OR (hl)");
}
pub fn ORd8(cpu: &mut Cpu) {
    let v = imm8(cpu);
    alu_or(cpu, v);
    debug!("OR imm8");
}
pub fn ANDhl(cpu: &mut Cpu) {
    let hl = cpu.mem.read8(cpu.regs.get_HL());
    alu_and(cpu, hl);
    debug!("AND A");
}
pub fn SUBad8(cpu: &mut Cpu) {
    let imm = imm8(cpu);
    alu_sub(cpu, imm, false);
    debug!("SUB A, {:02X}", imm);
}

pub fn ADCad8(cpu: &mut Cpu) {
    let imm = imm8(cpu);
    alu_add(cpu, imm, true);
    debug!("ADC A, {:02X}", imm);
}
pub fn ADDad8(cpu: &mut Cpu) {
    let imm = imm8(cpu);
    alu_add(cpu, imm, false);
    debug!("ADD A, {:02X}", imm);
}
pub fn ADDaa(cpu: &mut Cpu) {
    alu_add(cpu, cpu.regs.A, false);
    debug!("ADD A,A");
}
pub fn ADDad(cpu: &mut Cpu) {
    alu_add(cpu, cpu.regs.D, false);
    debug!("ADD A,D")
}
pub fn ADDahl(cpu: &mut Cpu) {
    let hl = cpu.mem.read8(cpu.regs.get_HL());
    alu_add(cpu, hl, false);
    debug!("ADD A, (HL)");
}
pub fn ADDab(cpu: &mut Cpu) {
    alu_add(cpu, cpu.regs.B, false);
    debug!("ADD A,B");
}
pub fn ADDac(cpu: &mut Cpu) {
    alu_add(cpu, cpu.regs.C, false);
    debug!("ADD A,C");
}
pub fn ADDhlde(cpu: &mut Cpu) {
    let de = cpu.regs.get_DE();
    alu_add16(cpu, de);
    debug!("ADD HL,DE");
}

pub fn ADDhlsp(cpu: &mut Cpu) {
    let sp = cpu.regs.get_SP();
    alu_add16(cpu, sp);
    debug!("ADD HL,SP");
}
pub fn INChl(cpu: &mut Cpu) {
    let hl = cpu.regs.get_HL();
    cpu.regs.set_HL(hl.wrapping_add(1));
    debug!("INC HL");
}
pub fn INCsp(cpu: &mut Cpu) {
    let sp = cpu.regs.get_SP();
    cpu.regs.set_SP(sp.wrapping_add(1));
    debug!("INC SP");
}
pub fn INC_hl(cpu: &mut Cpu) {
    let hl = cpu.mem.read8(cpu.regs.get_HL());
    cpu.mem.write8(cpu.regs.get_HL(), hl.wrapping_add(1));
    debug!("INC (HL)");
}
pub fn INCde(cpu: &mut Cpu) {
    let de = cpu.regs.get_DE();
    cpu.regs.set_DE(de.wrapping_add(1));
    debug!("INC DE");
}
pub fn INCa(cpu: &mut Cpu) {
    cpu.regs.A = alu_inc(cpu, cpu.regs.A);
    debug!("INC A");
}
pub fn INCh(cpu: &mut Cpu) {
    cpu.regs.H = alu_inc(cpu, cpu.regs.H);
    debug!("INC H");
}
pub fn INCl(cpu: &mut Cpu) {
    cpu.regs.L = alu_inc(cpu, cpu.regs.L);
    debug!("INC L");
}
pub fn INCc(cpu: &mut Cpu) {
    cpu.regs.C = alu_inc(cpu, cpu.regs.C);
    debug!("INC C");
}
pub fn INCd(cpu: &mut Cpu) {
    cpu.regs.D = alu_inc(cpu, cpu.regs.D);
    debug!("INC D");
}
pub fn INCe(cpu: &mut Cpu) {
    cpu.regs.E = alu_inc(cpu, cpu.regs.E);
    debug!("INC E");
}
pub fn CPc(cpu: &mut Cpu) {
    alu_cp(cpu, cpu.regs.C);
    debug!("CP C")
}
pub fn CPhl(cpu: &mut Cpu) {
    let hl = cpu.mem.read8(cpu.regs.get_HL());
    alu_cp(cpu, hl);
    debug!("CP HL");
}
pub fn CPd8(cpu: &mut Cpu) {
    let imm = imm8(cpu);
    alu_cp(cpu, imm);
    debug!("CP {:02X}", imm)
}

pub fn LDade(cpu: &mut Cpu) {
    let addr = cpu.regs.get_DE();
    cpu.regs.A = cpu.mem.read8(addr);
    debug!("LD A, (DE) ({:04X})", addr);
}
pub fn LDlhl(cpu: &mut Cpu) {
    let addr = cpu.regs.get_HL();
    cpu.regs.L = cpu.mem.read8(addr);
    debug!("LD L, (HL) ({:04X})", addr);
}
pub fn LDbhl(cpu: &mut Cpu) {
    let addr = cpu.regs.get_HL();
    cpu.regs.B = cpu.mem.read8(addr);
    debug!("LD B, (HL) ({:04X})", addr);
}
pub fn LDchl(cpu: &mut Cpu) {
    let addr = cpu.regs.get_HL();
    cpu.regs.C = cpu.mem.read8(addr);
    debug!("LD C, (HL) ({:04X})", addr);
}
pub fn LDaa16(cpu: &mut Cpu) {
    let addr = addr16(cpu);
    cpu.regs.A = cpu.mem.read8(addr);
    debug!("LD A, (a16) ({:04X})", addr);

}
pub fn LDhld16(cpu: &mut Cpu) {
    let imm = imm16(cpu);
    cpu.regs.set_HL(imm);
    debug!("LD HL, {:04X}", imm)
}
pub fn LDhla(cpu: &mut Cpu) {
    let hl = cpu.regs.get_HL();
    cpu.mem.write8(hl, cpu.regs.A);
    debug!("LD {:04X}, A", hl);
}
pub fn LDhlpa(cpu: &mut Cpu) {
    let hl = cpu.regs.get_HL();
    cpu.mem.write8(hl, cpu.regs.A);
    cpu.regs.set_HL(hl.wrapping_add(1));
    debug!("LD {:04X}+, A", hl);
}
pub fn LDIahlp(cpu: &mut Cpu) {
    let hl = cpu.regs.get_HL();
    cpu.regs.A = cpu.mem.read8(hl);
    cpu.regs.set_HL(hl.wrapping_add(1));
    debug!("LD A, (HL+)  {:02X}<-({:04X})", cpu.regs.A, hl);
}
pub fn LDhhl(cpu: &mut Cpu) {
    let hl = cpu.regs.get_HL();
    cpu.regs.H = cpu.mem.read8(hl);
    debug!("LD H, (HL)")
}
pub fn LDhlb(cpu: &mut Cpu) {
    let B = cpu.regs.B;
    cpu.regs.set_HL(B as u16);
    debug!("LD (HL), B")
}
pub fn LDhlc(cpu: &mut Cpu) {
    let C = cpu.regs.C;
    cpu.regs.set_HL(C as u16);
    debug!("LD (HL), C")
}
pub fn LDhld(cpu: &mut Cpu) {
    let D = cpu.regs.D;
    cpu.regs.set_HL(D as u16);
    debug!("LD (HL), D")
}
pub fn LDpca(cpu: &mut Cpu) {
    let C = cpu.regs.C as u16;
    cpu.mem.write8(0xFF00 + C, cpu.regs.A);
    debug!("LD (C), A")
}
pub fn LDspd16(cpu: &mut Cpu) {
    let imm = imm16(cpu);
    cpu.regs.set_SP(imm);
    debug!("LD SP, {:04X}", imm)
}
pub fn LDDhmla(cpu: &mut Cpu) {
    let hl = cpu.regs.get_HL();
    cpu.mem.write8(hl, cpu.regs.A);
    cpu.regs.set_HL(hl.wrapping_sub(1));
    debug!("LD- [{:04X}], a", hl);
}
pub fn LDed8(cpu: &mut Cpu) {
    let imm = imm8(cpu);
    cpu.regs.E = imm;
    debug!("LD E, {:02X}", imm)
}
pub fn LDld8(cpu: &mut Cpu) {
    let imm = imm8(cpu);
    cpu.regs.L = imm;
    debug!("LD L, {:02X}", imm)
}
pub fn LDad8(cpu: &mut Cpu) {
    let imm = imm8(cpu);
    cpu.regs.A = imm;
    debug!("LD A, {:02X}", imm)
}
pub fn LDdd8(cpu: &mut Cpu) {
    let imm = imm8(cpu);
    cpu.regs.D = imm;
    debug!("LD D, {:02X}", imm)
}
pub fn LDha8a(cpu: &mut Cpu) {
    let imm = imm8(cpu);
    cpu.mem.write8(0xFF00+imm as u16, cpu.regs.A);
    debug!("LDH (FF{:02X}), A", imm)
}
pub fn LDa16a(cpu: &mut Cpu) {
    let imm = addr16(cpu);
    cpu.mem.write8(imm, cpu.regs.A);
    debug!("LD ({:04X}), A", imm)
}
pub fn LDal(cpu: &mut Cpu) {
    cpu.regs.A = cpu.regs.L;
    debug!("LDH A, L")
}
pub fn LDba(cpu: &mut Cpu) {
    cpu.regs.B = cpu.regs.A;
    debug!("LD B, A")
}
pub fn LDbb(cpu: &mut Cpu) {
    cpu.regs.B = cpu.regs.B;
    debug!("LD B, B")
}
pub fn LDea(cpu: &mut Cpu) {
    cpu.regs.E = cpu.regs.A;
    debug!("LD E, A")
}
pub fn LDah(cpu: &mut Cpu) {
    cpu.regs.A = cpu.regs.H;
    debug!("LDH A, H")
}
pub fn LDehl(cpu: &mut Cpu) {
    let m = cpu.mem.read8(cpu.regs.get_HL());
    cpu.regs.E = m;
    debug!("LD E, {:04X}", m);
}
pub fn JRr8(cpu: &mut Cpu) {
    cpu_jr(cpu);
}
pub fn POPhl(cpu: &mut Cpu) {
    let sp = PopStack(cpu);
    cpu.regs.set_HL(sp);
    debug!("POP HL");
}
pub fn POPde(cpu: &mut Cpu) {
    let de = PopStack(cpu);
    cpu.regs.set_DE(de);
    debug!("POP DE");
}
pub fn POPbc(cpu: &mut Cpu) {
    let sp = PopStack(cpu);
    cpu.regs.set_BC(sp);
    debug!("POP BC");
}
pub fn POPaf(cpu: &mut Cpu) {
    let sp = PopStack(cpu);
    cpu.regs.set_AF(sp&0xFFF0);
    debug!("POP AF");
}
pub fn RST28h(cpu: &mut Cpu) {
    let PC = cpu.regs.PC;
    PushStack(cpu, PC);
    cpu.regs.PC = 0x28;
    debug!("RST 28h")
}
pub fn RST38h(cpu: &mut Cpu) {
    let PC = cpu.regs.PC;
    PushStack(cpu, PC);
    cpu.regs.PC = 0x38;
    debug!("RST 38h")
}
pub fn JRncr8(cpu: &mut Cpu) {
    if !cpu.regs.get_FC() { cpu_jr(cpu); } else { let pc = cpu.regs.get_PC(); cpu.regs.set_PC(pc+2); }
}
pub fn JRnzr8(cpu: &mut Cpu) {
    if !cpu.regs.get_FZ() { cpu_jr(cpu); } else { let pc = cpu.regs.get_PC(); cpu.regs.set_PC(pc+2); }
}
pub fn JRcr8(cpu: &mut Cpu) {
    if cpu.regs.get_FC() { cpu_jr(cpu); } else { let pc = cpu.regs.get_PC(); cpu.regs.set_PC(pc+2); }
}
pub fn JRzr8(cpu: &mut Cpu) {
    if cpu.regs.get_FZ() { cpu_jr(cpu); } else { let pc = cpu.regs.get_PC(); cpu.regs.set_PC(pc+2); }
}
pub fn RET(cpu: &mut Cpu) {
    let addr = PopStack(cpu);
    cpu.regs.PC = addr;
    debug!("RET (-> {:04X})", addr)
}
pub fn RETI(cpu: &mut Cpu) {
    let addr = PopStack(cpu);
    cpu.regs.PC = addr;
    EI(cpu);
    debug!("RETI (-> {:04X})", addr)
}
pub fn RETC(cpu: &mut Cpu) {
    if cpu.regs.get_FC() == true {
        let addr = PopStack(cpu);
        cpu.regs.PC = addr;
        debug!("RET NC (-> {:04X})", addr)
    } else {
        cpu.regs.PC = cpu.regs.PC.wrapping_add(1);
        debug!("RET NC (-> continue)")
    }
}
pub fn RETNC(cpu: &mut Cpu) {
    if cpu.regs.get_FC() == false {
        let addr = PopStack(cpu);
        cpu.regs.PC = addr;
        debug!("RET NC (-> {:04X})", addr)
    } else {
        cpu.regs.PC = cpu.regs.PC.wrapping_add(1);
        debug!("RET NC (-> continue)")
    }
}
pub fn DI(cpu: &mut Cpu) {
    cpu.regs.I = false;
    debug!("DI")
}
pub fn EI(cpu: &mut Cpu) {
    cpu.regs.I = true;
    debug!("EI")
}

pub fn SWAPa(cpu: &mut Cpu) {
    cpu.regs.A = ((cpu.regs.A&0xF0)>>4)|(cpu.regs.A<<4);
    cpu.regs.set_FZ(cpu.regs.A == 0);
    cpu.regs.set_FN(false);
    cpu.regs.set_FH(false);
    cpu.regs.set_FC(false);

    debug!("SWAP A");
}
pub fn RLc(cpu: &mut Cpu) {
    cpu.regs.C = alu_rl(cpu, cpu.regs.C);
}
pub fn RLa(cpu: &mut Cpu) {
    cpu.regs.A = alu_rl(cpu, cpu.regs.A);
}
pub fn RLA(cpu: &mut Cpu) {
    cpu.regs.A = alu_rl(cpu, cpu.regs.A);
    cpu.regs.set_FZ(false);
}
pub fn BIT5a(cpu: &mut Cpu) {
    alu_bit(cpu, cpu.regs.A, 5);
    debug!("BIT5, A")
}
pub fn BIT6a(cpu: &mut Cpu) {
    alu_bit(cpu, cpu.regs.A, 6);
    debug!("BIT6, A")
}
pub fn BIT7h(cpu: &mut Cpu) {
    alu_bit(cpu, cpu.regs.H, 7);
    debug!("BIT7, H")
}


pub fn SRLa(cpu: &mut Cpu) {
    cpu.regs.A = alu_srl(cpu, cpu.regs.A);
}
pub fn SRLb(cpu: &mut Cpu) {
    cpu.regs.B = alu_srl(cpu, cpu.regs.B);
}
pub fn SRLc(cpu: &mut Cpu) {
    cpu.regs.C = alu_srl(cpu, cpu.regs.C);
}
pub fn SRLd(cpu: &mut Cpu) {
    cpu.regs.D = alu_srl(cpu, cpu.regs.D);
}
pub fn SRLe(cpu: &mut Cpu) {
    cpu.regs.E = alu_srl(cpu, cpu.regs.E);
}
pub fn SRLh(cpu: &mut Cpu) {
    cpu.regs.H = alu_srl(cpu, cpu.regs.H);
}
pub fn SRLl(cpu: &mut Cpu) {
    cpu.regs.L = alu_srl(cpu, cpu.regs.L);
}


pub fn PushStack(cpu: &mut Cpu, v: u16) {
    cpu.mem.write16(cpu.regs.SP, v);
    cpu.regs.SP = cpu.regs.SP.wrapping_sub(2);
}
pub fn PopStack(cpu: &mut Cpu) -> u16 {
    cpu.regs.SP = cpu.regs.SP.wrapping_add(2);
    let addr = cpu.mem.read16(cpu.regs.SP);
    addr
}


impl<'a> Cpu<'a>{

    pub fn new(mem: mem::Mem) -> Cpu {
        let mut cpu: Cpu;
        cpu = Cpu{
            regs: Registers {
                A: 1,
                B: 0,
                D: 0,
                H: 1,
                F: 0xB0,
                C: 0,
                E: 0xD8,
                L: 0x4D,
                I: false,
                PC: 0,
                SP: 0xFFFE,
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
                }; 256],
                alt_opcodes:
                    vec![Opcode{
                        name: "ALT UNK",
                        len: 1,
                        cycles: 4,
                        execute: ALTUNK,
                        jump: false,
                    }; 256]

        };
        cpu.opcodes[0] = Opcode {
            name: "NOP",
            len: 1,
            cycles: 4,
            execute: |_cpu|{},
            jump: false,
        };
        cpu.opcodes[0x01] = Opcode {
            name: "LD BC, d16",
            len: 3,
            cycles: 12,
            execute: |cpu|{
                let imm = imm16(cpu);
                cpu.regs.set_BC(imm);
            },
            jump: false,
        };
        cpu.opcodes[0x03] = Opcode {
            name: "INC BC",
            len: 1,
            cycles: 8,
            execute: |cpu|{
                let bc = cpu.regs.get_BC();
                cpu.regs.set_BC(bc.wrapping_add(1));
            },
            jump: false,
        };
        cpu.opcodes[0x04] = Opcode {
            name: "INC B",
            len: 1,
            cycles: 4,
            execute: |cpu| {
                cpu.regs.B = alu_inc(cpu, cpu.regs.B);
            },
            jump: false,
        };
        cpu.opcodes[0x05] = Opcode {
            name: "DEC B",
            len: 1,
            cycles: 4,
            execute: |cpu|{
                cpu.regs.B = alu_dec(cpu, cpu.regs.B);
            },
            jump: false,
        };
        cpu.opcodes[0x06] = Opcode {
            name: "LD B, d8",
            len: 2,
            cycles: 8,
            execute: |cpu|{
                let imm = imm8(cpu);
                cpu.regs.B = imm;
            },
            jump: false,
        };
        cpu.opcodes[0x07] = Opcode {
            name: "RLCA",
            len: 1,
            cycles: 4,
            execute: |cpu|{
                let c = cpu.regs.A >> 7;
                cpu.regs.A = (cpu.regs.A << 1) | c;
                cpu.regs.set_FZ(cpu.regs.A == 0);
                cpu.regs.set_FN(false);
                cpu.regs.set_FH(false);
                cpu.regs.set_FC(c==1);
            },
            jump: false,
        };
        cpu.opcodes[0x08] = Opcode {
            name: "LD (a16),SP",
            len: 3,
            cycles: 20,
            execute: |cpu|{
                let a = imm16(cpu); cpu.mem.write16(a, cpu.regs.get_SP());
            },
            jump: false,
        };
        cpu.opcodes[0x09] = Opcode {
            name: "ADD HL, BC",
            len: 1,
            cycles: 8,
            execute: |cpu|{
                let bc = cpu.regs.get_BC();
                alu_add16(cpu, bc);
            },
            jump: false,
        };
        cpu.opcodes[0x0B] = Opcode {
            name: "DEC BC",
            len: 1,
            cycles: 4,
            execute: |cpu|{
                let bc = cpu.regs.get_BC();
                cpu.regs.set_BC(bc.wrapping_sub(1));
            },
            jump: false,
        };
        cpu.opcodes[0x0C] = Opcode {
            name: "INC C",
            len: 1,
            cycles: 4,
            execute: |cpu| {
                cpu.regs.C = alu_inc(cpu, cpu.regs.C);
            },
            jump: false,
        };
        cpu.opcodes[0x0D] = Opcode {
            name: "DEC C",
            len: 1,
            cycles: 4,
            execute: |cpu| {
                cpu.regs.C = alu_dec(cpu, cpu.regs.C);
            },
            jump: false,
        };
        cpu.opcodes[0x0E] = Opcode {
            name: "LD C, d8",
            len: 2,
            cycles: 8,
            execute: |cpu|{
                let imm = imm8(cpu);
                cpu.regs.C = imm;

            },
            jump: false,
        };
        cpu.opcodes[0x0F] = Opcode {
            name: "RRCA",
            len: 1,
            cycles: 4,
            execute: |cpu|{
                cpu.regs.A = alu_rrc(cpu, cpu.regs.A);
                cpu.regs.set_FZ(false);
            },
            jump: false,
        };
        cpu.opcodes[0x11] = Opcode {
            name: "LD DE, d16",
            len: 3,
            cycles: 12,
            execute: |cpu| {
                let imm = imm16(cpu);
                cpu.regs.set_DE(imm);
            },
            jump: false,
        };
        cpu.opcodes[0x12] = Opcode {
            name: "LD (DE), A",
            len: 1,
            cycles: 8,
            execute: |cpu|{
                cpu.mem.write8(cpu.regs.get_DE(), cpu.regs.A);
            },
            jump: false,
        };
        cpu.opcodes[0x13] = Opcode {
            name: "INC DE",
            len: 1,
            cycles: 8,
            execute: INCde,
            jump: false,
        };
        cpu.opcodes[0x14] = Opcode {
            name: "INC D",
            len: 1,
            cycles: 4,
            execute: |cpu| {
                cpu.regs.D = alu_inc(cpu, cpu.regs.D);
            },
            jump: false,
        };
        cpu.opcodes[0x15] = Opcode {
            name: "DEC D",
            len: 1,
            cycles: 4,
            execute: |cpu| {
                cpu.regs.D = alu_dec(cpu, cpu.regs.D);
            },
            jump: false,
        };
        cpu.opcodes[0x16] = Opcode {
            name: "LD D, d8",
            len: 2,
            cycles: 8,
            execute: LDdd8,
            jump: false,
        };
        cpu.opcodes[0x17] = Opcode {
            name: "RLA",
            len: 1,
            cycles: 4,
            execute: RLA,
            jump: false,
        };
        cpu.opcodes[0x18] = Opcode {
            name: "JR r8",
            len: 2,
            cycles: 12,
            execute: JRr8,
            jump: true,
        };
        cpu.opcodes[0x19] = Opcode {
            name: "ADD HL, DE",
            len: 1,
            cycles: 8,
            execute: ADDhlde,
            jump: false,
        };
        cpu.opcodes[0x1A] = Opcode {
            name: "LD A, (DE)",
            len: 1,
            cycles: 8,
            execute: LDade,
            jump: false,
        };
        cpu.opcodes[0x1B] = Opcode {
            name: "DEC DE",
            len: 1,
            cycles: 4,
            execute: |cpu| {
                let de = cpu.regs.get_DE();
                cpu.regs.set_BC(de.wrapping_sub(1)) },
                jump: false,
        };
        cpu.opcodes[0x1C] = Opcode {
            name: "INC E",
            len: 1,
            cycles: 4,
            execute: |cpu| {
                cpu.regs.E = alu_inc(cpu, cpu.regs.E);
            },
            jump: false,
        };
        cpu.opcodes[0x1D] = Opcode {
            name: "DEC E",
            len: 1,
            cycles: 4,
            execute: |cpu| {
                cpu.regs.E = alu_dec(cpu, cpu.regs.E);
            },
            jump: false,
        };
        cpu.opcodes[0x1E] = Opcode {
            name: "LD E, d8",
            len: 2,
            cycles: 8,
            execute: LDed8,
            jump: false,
        };
        cpu.opcodes[0x1F] = Opcode {
            name: "RRA",
            len: 1,
            cycles: 4,
            execute: |cpu|{cpu.regs.A = alu_rr(cpu, cpu.regs.A);},
            jump: false,
        };
        cpu.opcodes[0x20] = Opcode {
            name: "JR NZ, r8",
            len: 2,
            cycles: 12,
            execute: JRnzr8,
            jump: true,
        };
        cpu.opcodes[0x21] = Opcode {
            name: "LD HL, d16",
            len: 3,
            cycles: 13,
            execute: LDhld16,
            jump: false,
        };
        cpu.opcodes[0x22] = Opcode {
            name: "LD (HL+),A",
            len: 1,
            cycles: 8,
            execute: LDhlpa,
            jump: false,
        };
        cpu.opcodes[0x23] = Opcode {
            name: "INC HL",
            len: 1,
            cycles: 8,
            execute: INChl,
            jump: false,
        };
        cpu.opcodes[0x24] = Opcode {
            name: "INC H",
            len: 1,
            cycles: 4,
            execute: |cpu| {
                cpu.regs.H = alu_inc(cpu, cpu.regs.H);
            },
            jump: false,
        };
        cpu.opcodes[0x25] = Opcode {
            name: "DEC H",
            len: 1,
            cycles: 4,
            execute: |cpu| {
                cpu.regs.H = alu_dec(cpu, cpu.regs.H);
            },
            jump: false,
        };
        cpu.opcodes[0x26] = Opcode {
            name: "LD H, d8",
            len: 2,
            cycles: 8,
            execute: |cpu|{
                let imm = imm8(cpu);
                cpu.regs.H = imm;
            },
            jump: false,
        };
        cpu.opcodes[0x27] = Opcode {
            name: "DAA",
            len: 1,
            cycles: 4,
            execute: |cpu| {alu_daa(cpu);},
            jump: false,
        };
        cpu.opcodes[0x28] = Opcode {
            name: "JR Z, r8",
            len: 2,
            cycles: 12,
            execute: JRzr8,
            jump: true,
        };
        cpu.opcodes[0x29] = Opcode {
            name: "ADD HL, HL",
            len: 1,
            cycles: 8,
            execute: |cpu|{let hl = cpu.regs.get_HL(); alu_add16(cpu, hl);},
            jump: false,
        };
        cpu.opcodes[0x2A] = Opcode {
            name: "LDI A, (HL+)",
            len: 1,
            cycles: 8,
            execute: LDIahlp,
            jump: false,
        };
        cpu.opcodes[0x2B] = Opcode {
            name: "DEC HL",
            len: 1,
            cycles: 8,
            execute: |cpu|{let v = cpu.regs.get_HL().wrapping_sub(1); cpu.regs.set_HL(v);},
            jump: false,
        };
        cpu.opcodes[0x2C] = Opcode {
            name: "INC L",
            len: 1,
            cycles: 4,
            execute: |cpu| {cpu.regs.L = alu_inc(cpu, cpu.regs.L); },
            jump: false,
        };
        cpu.opcodes[0x2D] = Opcode {
            name: "DEC L",
            len: 1,
            cycles: 4,
            execute: |cpu| {cpu.regs.L = alu_dec(cpu, cpu.regs.L);},
            jump: false,
        };
        cpu.opcodes[0x2E] = Opcode {
            name: "LD L, d8",
            len: 2,
            cycles: 8,
            execute: LDld8,
            jump: false,
        };
        cpu.opcodes[0x2F] = Opcode {
            name: "CPL",
            len: 1,
            cycles: 4,
            execute: |cpu| {let A = cpu.regs.A; cpu.regs.A = !A; cpu.regs.set_FN(true); cpu.regs.set_FH(true); },
            jump: false,
        };
        cpu.opcodes[0x30] = Opcode {
            name: "JR NC, r8",
            len: 2,
            cycles: 12,
            execute: JRncr8,
            jump: true,
        };
        cpu.opcodes[0x31] = Opcode {
            name: "LD SP, d16",
            len: 3,
            cycles: 12,
            execute: LDspd16,
            jump: false,
        };
        cpu.opcodes[0x32] = Opcode {
            name: "LDD (HL-), A",
            len: 1,
            cycles: 8,
            execute: LDDhmla,
            jump: false,
        };
        cpu.opcodes[0x33] = Opcode {
            name: "INC SP",
            len: 1,
            cycles: 8,
            execute: INCsp,
            jump: false,
        };
        cpu.opcodes[0x34] = Opcode {
            name: "INC (HL)",
            len: 1,
            cycles: 12,
            execute: INC_hl,
            jump: false,
        };
        cpu.opcodes[0x35] = Opcode {
            name: "DEC (hl)",
            len: 1,
            cycles: 12,
            execute: |cpu| {let hl = cpu.mem.read8(cpu.regs.get_HL()); cpu.mem.write8(cpu.regs.get_HL(), hl.wrapping_sub(1)); },
            jump: false,
        };
        cpu.opcodes[0x36] = Opcode {
            name: "LD (HL), d8",
            len: 2,
            cycles: 12,
            execute: |cpu|{let imm = imm8(cpu); cpu.mem.write8(cpu.regs.get_HL(), imm);},
            jump: false,
        };
        cpu.opcodes[0x38] = Opcode {
            name: "JR C r8",
            len: 2,
            cycles: 12,
            execute: JRcr8,
            jump: true,
        };
        cpu.opcodes[0x39] = Opcode {
            name: "ADD HL, SP",
            len: 1,
            cycles: 8,
            execute: ADDhlsp,
            jump: false,
        };
        cpu.opcodes[0x3B] = Opcode {
            name: "DEC SP",
            len: 1,
            cycles: 4,
            execute: |cpu| {
                cpu.regs.SP = cpu.regs.SP.wrapping_sub(1);
            },
            jump: false,
        };
        cpu.opcodes[0x3C] = Opcode {
            name: "INC A",
            len: 1,
            cycles: 4,
            execute: |cpu| {
                cpu.regs.A = alu_inc(cpu, cpu.regs.A);
            },
            jump: false,
        };
        cpu.opcodes[0x3D] = Opcode {
            name: "DEC A",
            len: 1,
            cycles: 4,
            execute: |cpu| {
                cpu.regs.A = alu_dec(cpu, cpu.regs.A);
            },
            jump: false,
        };
        cpu.opcodes[0x3E] = Opcode {
            name: "LD A, d8",
            len: 2,
            cycles: 8,
            execute: LDad8,
            jump: false,
        };
        cpu.opcodes[0x40] = Opcode {
            name: "LD B, B",
            len: 1,
            cycles: 4,
            execute: |cpu|{cpu.regs.B = cpu.regs.B;},
            jump: false,
        };
        cpu.opcodes[0x41] = Opcode {
            name: "LD B, C",
            len: 1,
            cycles: 4,
            execute: |cpu|{cpu.regs.B = cpu.regs.C;},
            jump: false,
        };
        cpu.opcodes[0x42] = Opcode {
            name: "LD B, D",
            len: 1,
            cycles: 4,
            execute: |cpu|{cpu.regs.B = cpu.regs.D;},
            jump: false,
        };
        cpu.opcodes[0x43] = Opcode {
            name: "LD B, E",
            len: 1,
            cycles: 4,
            execute: |cpu|{cpu.regs.B = cpu.regs.E;},
            jump: false,
        };
        cpu.opcodes[0x44] = Opcode {
            name: "LD B, H",
            len: 1,
            cycles: 4,
            execute: |cpu|{cpu.regs.B = cpu.regs.H;},
            jump: false,
        };
        cpu.opcodes[0x45] = Opcode {
            name: "LD B, L",
            len: 1,
            cycles: 4,
            execute: |cpu|{cpu.regs.B = cpu.regs.L;},
            jump: false,
        };
        cpu.opcodes[0x46] = Opcode {
            name: "LD B, (HL)",
            len: 1,
            cycles: 8,
            execute: LDbhl,
            jump: false,
        };
        cpu.opcodes[0x47] = Opcode {
            name: "LD B, A",
            len: 1,
            cycles: 4,
            execute: |cpu|{cpu.regs.B = cpu.regs.A;},
            jump: false,
        };
        cpu.opcodes[0x48] = Opcode {
            name: "LD C, B",
            len: 1,
            cycles: 4,
            execute: |cpu|{cpu.regs.C = cpu.regs.B;},
            jump: false,
        };
        cpu.opcodes[0x49] = Opcode {
            name: "LD C, C",
            len: 1,
            cycles: 4,
            execute: |cpu|{cpu.regs.C = cpu.regs.C;},
            jump: false,
        };
        cpu.opcodes[0x4A] = Opcode {
            name: "LD C, D",
            len: 1,
            cycles: 4,
            execute: |cpu|{cpu.regs.C = cpu.regs.D;},
            jump: false,
        };
        cpu.opcodes[0x4B] = Opcode {
            name: "LD C, E",
            len: 1,
            cycles: 4,
            execute: |cpu|{cpu.regs.C = cpu.regs.E;},
            jump: false,
        };
        cpu.opcodes[0x4C] = Opcode {
            name: "LD C, H",
            len: 1,
            cycles: 4,
            execute: |cpu|{cpu.regs.C = cpu.regs.H;},
            jump: false,
        };
        cpu.opcodes[0x4D] = Opcode {
            name: "LD C, L",
            len: 1,
            cycles: 4,
            execute: |cpu|{cpu.regs.C = cpu.regs.L;},
            jump: false,
        };
        cpu.opcodes[0x4E] = Opcode {
            name: "LD C, (HL)",
            len: 1,
            cycles: 8,
            execute: LDchl,
            jump: false,
        };
        cpu.opcodes[0x4F] = Opcode {
            name: "LD C, A",
            len: 1,
            cycles: 4,
            execute: |cpu|{cpu.regs.C = cpu.regs.A;},
            jump: false,
        };
        cpu.opcodes[0x50] = Opcode {
            name: "LD D, B",
            len: 1,
            cycles: 4,
            execute: |cpu|{cpu.regs.D = cpu.regs.B;},
            jump: false,
        };
        cpu.opcodes[0x51] = Opcode {
            name: "LD D, C",
            len: 1,
            cycles: 4,
            execute: |cpu|{cpu.regs.D = cpu.regs.C;},
            jump: false,
        };
        cpu.opcodes[0x52] = Opcode {
            name: "LD D, D",
            len: 1,
            cycles: 4,
            execute: |cpu|{cpu.regs.D = cpu.regs.D;},
            jump: false,
        };
        cpu.opcodes[0x53] = Opcode {
            name: "LD D, E",
            len: 1,
            cycles: 4,
            execute: |cpu|{cpu.regs.D = cpu.regs.E;},
            jump: false,
        };
        cpu.opcodes[0x54] = Opcode {
            name: "LD D, H",
            len: 1,
            cycles: 4,
            execute: |cpu|{cpu.regs.D = cpu.regs.H;},
            jump: false,
        };
        cpu.opcodes[0x55] = Opcode {
            name: "LD D, L",
            len: 1,
            cycles: 4,
            execute: |cpu|{cpu.regs.D = cpu.regs.L;},
            jump: false,
        };
        cpu.opcodes[0x56] = Opcode {
            name: "LD D, (HL)",
            len: 1,
            cycles: 8,
            execute: |cpu|{ let m = cpu.mem.read8(cpu.regs.get_HL()); cpu.regs.D = m; },
            jump: false,
        };
        cpu.opcodes[0x57] = Opcode {
            name: "LD D, A",
            len: 1,
            cycles: 4,
            execute: |cpu|{cpu.regs.D = cpu.regs.A;},
            jump: false,
        };
        cpu.opcodes[0x58] = Opcode {
            name: "LD E, B",
            len: 1,
            cycles: 4,
            execute: |cpu|{cpu.regs.E = cpu.regs.B;},
            jump: false,
        };
        cpu.opcodes[0x59] = Opcode {
            name: "LD E, C",
            len: 1,
            cycles: 4,
            execute: |cpu|{cpu.regs.E = cpu.regs.C;},
            jump: false,
        };
        cpu.opcodes[0x5A] = Opcode {
            name: "LD E, D",
            len: 1,
            cycles: 4,
            execute: |cpu|{cpu.regs.E = cpu.regs.D;},
            jump: false,
        };
        cpu.opcodes[0x5B] = Opcode {
            name: "LD E, E",
            len: 1,
            cycles: 4,
            execute: |cpu|{cpu.regs.E = cpu.regs.E;},
            jump: false,
        };
        cpu.opcodes[0x5C] = Opcode {
            name: "LD E, H",
            len: 1,
            cycles: 4,
            execute: |cpu|{cpu.regs.E = cpu.regs.H;},
            jump: false,
        };
        cpu.opcodes[0x5D] = Opcode {
            name: "LD E, L",
            len: 1,
            cycles: 4,
            execute: |cpu|{cpu.regs.E = cpu.regs.L;},
            jump: false,
        };
        cpu.opcodes[0x5E] = Opcode {
            name: "LD E, (HL)",
            len: 1,
            cycles: 8,
            execute: LDehl,
            jump: false,
        };
        cpu.opcodes[0x5F] = Opcode {
            name: "LD E, A",
            len: 1,
            cycles: 4,
            execute: LDea,
            jump: false,
        };
        cpu.opcodes[0x60] = Opcode {
            name: "LD H, B",
            len: 1,
            cycles: 4,
            execute: |cpu|{cpu.regs.H = cpu.regs.B;},
            jump: false,
        };
        cpu.opcodes[0x61] = Opcode {
            name: "LD H, C",
            len: 1,
            cycles: 4,
            execute: |cpu|{cpu.regs.H = cpu.regs.C;},
            jump: false,
        };
        cpu.opcodes[0x62] = Opcode {
            name: "LD H, D",
            len: 1,
            cycles: 4,
            execute: |cpu|{cpu.regs.H = cpu.regs.D;},
            jump: false,
        };
        cpu.opcodes[0x63] = Opcode {
            name: "LD H, E",
            len: 1,
            cycles: 4,
            execute: |cpu|{cpu.regs.H = cpu.regs.E;},
            jump: false,
        };
        cpu.opcodes[0x64] = Opcode {
            name: "LD H, H",
            len: 1,
            cycles: 4,
            execute: |cpu|{cpu.regs.H = cpu.regs.H;},
            jump: false,
        };
        cpu.opcodes[0x65] = Opcode {
            name: "LD H, L",
            len: 1,
            cycles: 4,
            execute: |cpu|{cpu.regs.H = cpu.regs.L;},
            jump: false,
        };
        cpu.opcodes[0x66] = Opcode {
            name: "LD H, (HL)",
            len: 1,
            cycles: 8,
            execute: LDhhl,
            jump: false,
        };
        cpu.opcodes[0x67] = Opcode {
            name: "LD H, A",
            len: 1,
            cycles: 4,
            execute: |cpu|{cpu.regs.H = cpu.regs.A;},
            jump: false,
        };
        cpu.opcodes[0x68] = Opcode {
            name: "LD L, B",
            len: 1,
            cycles: 4,
            execute: |cpu|{cpu.regs.L = cpu.regs.B;},
            jump: false,
        };
        cpu.opcodes[0x69] = Opcode {
            name: "LD L, C",
            len: 1,
            cycles: 4,
            execute: |cpu|{cpu.regs.L = cpu.regs.C;},
            jump: false,
        };
        cpu.opcodes[0x6A] = Opcode {
            name: "LD L, D",
            len: 1,
            cycles: 4,
            execute: |cpu|{cpu.regs.L = cpu.regs.D;},
            jump: false,
        };
        cpu.opcodes[0x6B] = Opcode {
            name: "LD L, E",
            len: 1,
            cycles: 4,
            execute: |cpu|{cpu.regs.L = cpu.regs.E;},
            jump: false,
        };
        cpu.opcodes[0x6C] = Opcode {
            name: "LD L, H",
            len: 1,
            cycles: 4,
            execute: |cpu|{cpu.regs.L = cpu.regs.H;},
            jump: false,
        };
        cpu.opcodes[0x6D] = Opcode {
            name: "LD L, L",
            len: 1,
            cycles: 4,
            execute: |cpu|{cpu.regs.L = cpu.regs.L;},
            jump: false,
        };
        cpu.opcodes[0x6E] = Opcode {
            name: "LD L, (HL)",
            len: 1,
            cycles: 8,
            execute: LDlhl,
            jump: false,
        };
        cpu.opcodes[0x6F] = Opcode {
            name: "LD L, A",
            len: 1,
            cycles: 4,
            execute: |cpu|{cpu.regs.L = cpu.regs.A;},
            jump: false,
        };
        cpu.opcodes[0x70] = Opcode {
            name: "LD (HL),B",
            len: 1,
            cycles: 8,
            execute: |cpu|{let hl = cpu.regs.get_HL(); cpu.mem.write8(hl, cpu.regs.B); },
            jump: false,
        };
        cpu.opcodes[0x71] = Opcode {
            name: "LD (HL),C",
            len: 1,
            cycles: 8,
            execute: |cpu|{let hl = cpu.regs.get_HL(); cpu.mem.write8(hl, cpu.regs.C); },
            jump: false,
        };
        cpu.opcodes[0x72] = Opcode {
            name: "LD (HL),D",
            len: 1,
            cycles: 8,
            execute: |cpu|{let hl = cpu.regs.get_HL(); cpu.mem.write8(hl, cpu.regs.D); },
            jump: false,
        };
        cpu.opcodes[0x73] = Opcode {
            name: "LD (HL),E",
            len: 1,
            cycles: 8,
            execute: |cpu|{let hl = cpu.regs.get_HL(); cpu.mem.write8(hl, cpu.regs.E); },
            jump: false,
        };
        cpu.opcodes[0x74] = Opcode {
            name: "LD (HL),H",
            len: 1,
            cycles: 8,
            execute: |cpu|{let hl = cpu.regs.get_HL(); cpu.mem.write8(hl, cpu.regs.H); },
            jump: false,
        };
        cpu.opcodes[0x75] = Opcode {
            name: "LD (HL),L",
            len: 1,
            cycles: 8,
            execute: |cpu|{let hl = cpu.regs.get_HL(); cpu.mem.write8(hl, cpu.regs.L); },
            jump: false,
        };
        cpu.opcodes[0x76] = Opcode {
            name: "HALT",
            len: 1,
            cycles: 8,
            execute: |_|{println!("HALT"); loop{}},
            jump: false,
        };
        cpu.opcodes[0x77] = Opcode {
            name: "LD (HL),A",
            len: 1,
            cycles: 8,
            execute: |cpu|{let hl = cpu.regs.get_HL(); cpu.mem.write8(hl, cpu.regs.A); },
            jump: false,
        };
        cpu.opcodes[0x78] = Opcode {
            name: "LD A, B",
            len: 1,
            cycles: 4,
            execute: |cpu|{cpu.regs.A = cpu.regs.B;},
            jump: false,
        };
        cpu.opcodes[0x7A] = Opcode {
            name: "LD A, D",
            len: 1,
            cycles: 4,
            execute: |cpu|{cpu.regs.A = cpu.regs.D;},
            jump: false,
        };
        cpu.opcodes[0x7B] = Opcode {
            name: "LD A, E",
            len: 1,
            cycles: 4,
            execute: |cpu|{cpu.regs.A = cpu.regs.E;},
            jump: false,
        };
        cpu.opcodes[0x79] = Opcode {
            name: "LD A, C",
            len: 1,
            cycles: 4,
            execute: |cpu|{cpu.regs.A = cpu.regs.C;},
            jump: false,
        };
        cpu.opcodes[0x7C] = Opcode {
            name: "LD A, H",
            len: 1,
            cycles: 4,
            execute: |cpu|{cpu.regs.A = cpu.regs.H;},
            jump: false,
        };
        cpu.opcodes[0x7D] = Opcode {
            name: "LD A, L",
            len: 1,
            cycles: 4,
            execute: |cpu|{cpu.regs.A = cpu.regs.L;},
            jump: false,
        };
        cpu.opcodes[0x7E] = Opcode {
            name: "LD A, C",
            len: 1,
            cycles: 4,
            execute: |cpu|{cpu.regs.A = cpu.regs.C;},
            jump: false,
        };
        cpu.opcodes[0x7F] = Opcode {
            name: "LD A, A",
            len: 1,
            cycles: 4,
            execute: |cpu|{cpu.regs.A = cpu.regs.A;},
            jump: false,
        };
        cpu.opcodes[0x80] = Opcode {
            name: "ADD A,B",
            len: 1,
            cycles: 4,
            execute: ADDab,
            jump: false,
        };
        cpu.opcodes[0x81] = Opcode {
            name: "ADD A,C",
            len: 1,
            cycles: 4,
            execute: ADDac,
            jump: false,
        };
        cpu.opcodes[0x82] = Opcode {
            name: "ADD A,D",
            len: 1,
            cycles: 4,
            execute: ADDad,
            jump: false,
        };
        cpu.opcodes[0x86] = Opcode {
            name: "ADD A, (HL)",
            len: 1,
            cycles: 8,
            execute: ADDahl,
            jump: false,
        };
        cpu.opcodes[0x87] = Opcode {
            name: "ADD A,A",
            len: 1,
            cycles: 4,
            execute: ADDaa,
            jump: false,
        };
        cpu.opcodes[0x89] = Opcode {
            name: "ADC A,C",
            len: 1,
            cycles: 4,
            execute: |cpu| {alu_add(cpu, cpu.regs.C, true);},
            jump: false,
        };
        cpu.opcodes[0x8C] = Opcode {
            name: "ADC A,H",
            len: 1,
            cycles: 4,
            execute: |cpu| {alu_add(cpu, cpu.regs.H, true);},
            jump: false,
        };
        cpu.opcodes[0x90] = Opcode {
            name: "SUB B",
            len: 1,
            cycles: 4,
            execute: |cpu|{alu_sub(cpu, cpu.regs.B, false);},
            jump: false,
        };
        cpu.opcodes[0x91] = Opcode {
            name: "SUB C",
            len: 1,
            cycles: 4,
            execute: |cpu|{alu_sub(cpu, cpu.regs.C, false);},
            jump: false,
        };
        cpu.opcodes[0x92] = Opcode {
            name: "SUB D",
            len: 1,
            cycles: 4,
            execute: |cpu|{alu_sub(cpu, cpu.regs.D, false);},
            jump: false,
        };
        cpu.opcodes[0x97] = Opcode {
            name: "SUB A",
            len: 1,
            cycles: 4,
            execute: |cpu|{alu_sub(cpu, cpu.regs.A, false);},
            jump: false,
        };
        cpu.opcodes[0x98] = Opcode {
            name: "SBC A, B",
            len: 1,
            cycles: 4,
            execute: |cpu|{alu_sub(cpu, cpu.regs.B, true);},
            jump: false,
        };
        cpu.opcodes[0x9B] = Opcode {
            name: "SBC A, E",
            len: 1,
            cycles: 4,
            execute: |cpu|{let e = cpu.regs.E; alu_sub(cpu, e, true);},
            jump: false,
        };
        cpu.opcodes[0x9C] = Opcode {
            name: "SBC A, H",
            len: 1,
            cycles: 4,
            execute: |cpu|{let h = cpu.regs.H; alu_sub(cpu, h, true);},
            jump: false,
        };
        cpu.opcodes[0xA1] = Opcode {
            name: "AND C",
            len: 1,
            cycles: 4,
            execute: |cpu|{alu_and(cpu, cpu.regs.C);},
            jump: false,
        };
        cpu.opcodes[0xA4] = Opcode {
            name: "AND H",
            len: 1,
            cycles: 4,
            execute: |cpu|{alu_and(cpu, cpu.regs.H);},
            jump: false,
        };
        cpu.opcodes[0xA6] = Opcode {
            name: "AND (HL)",
            len: 1,
            cycles: 4,
            execute: ANDhl,
            jump: false,
        };
        cpu.opcodes[0xA7] = Opcode {
            name: "AND A",
            len: 1,
            cycles: 4,
            execute: |cpu|{alu_and(cpu, cpu.regs.A);},
            jump: false,
        };
        cpu.opcodes[0xA9] = Opcode {
            name: "XOR C",
            len: 1,
            cycles: 4,
            execute: |cpu|{ alu_xor(cpu, cpu.regs.C); },
            jump: false,
        };
        cpu.opcodes[0xAD] = Opcode {
            name: "XOR L",
            len: 1,
            cycles: 4,
            execute: |cpu|{ alu_xor(cpu, cpu.regs.L); },
            jump: false,
        };
        cpu.opcodes[0xAE] = Opcode {
            name: "XOR A, (HL)",
            len: 1,
            cycles: 8,
            execute: XOR_hl,
            jump: false,
        };
        cpu.opcodes[0xAF] = Opcode {
            name: "XOR A",
            len: 1,
            cycles: 4,
            execute: |cpu|{ alu_xor(cpu, cpu.regs.A); },
            jump: false,
        };
        cpu.opcodes[0xB0] = Opcode {
            name: "OR B",
            len: 1,
            cycles: 4,
            execute: |cpu|{ alu_or(cpu, cpu.regs.B); },
            jump: false,
        };
        cpu.opcodes[0xB1] = Opcode {
            name: "OR C",
            len: 1,
            cycles: 4,
            execute: |cpu|{ alu_or(cpu, cpu.regs.C); },
            jump: false,
        };
        cpu.opcodes[0xB2] = Opcode {
            name: "OR D",
            len: 1,
            cycles: 4,
            execute: |cpu|{ alu_or(cpu, cpu.regs.D); },
            jump: false,
        };
        cpu.opcodes[0xB3] = Opcode {
            name: "OR E",
            len: 1,
            cycles: 4,
            execute: |cpu|{ alu_or(cpu, cpu.regs.E); },
            jump: false,
        };
        cpu.opcodes[0xB6] = Opcode {
            name: "OR [HL]",
            len: 1,
            cycles: 8,
            execute: ORhl,
            jump: false,
        };
        cpu.opcodes[0xB7] = Opcode {
            name: "OR A",
            len: 1,
            cycles: 4,
            execute: |cpu|{ alu_or(cpu, cpu.regs.A); },
            jump: false,
        };
        cpu.opcodes[0xB8] = Opcode {
            name: "CP B",
            len: 1,
            cycles: 4,
            execute: |cpu| {alu_cp(cpu, cpu.regs.B);},
            jump: false,
        };
        cpu.opcodes[0xB9] = Opcode {
            name: "CPC",
            len: 1,
            cycles: 4,
            execute: CPc,
            jump: false,
        };
        cpu.opcodes[0xBB] = Opcode {
            name: "CP E",
            len: 1,
            cycles: 4,
            execute: |cpu| {alu_cp(cpu, cpu.regs.E);},
            jump: false,
        };
        cpu.opcodes[0xBE] = Opcode {
            name: "CP (HL)",
            len: 1,
            cycles: 8,
            execute: CPhl,
            jump: false,
        };
        cpu.opcodes[0xBF] = Opcode {
            name: "CP A",
            len: 1,
            cycles: 4,
            execute: |cpu| {alu_cp(cpu, cpu.regs.A);},
            jump: false,
        };
        cpu.opcodes[0xCA] = Opcode {
            name: "JP Z a16",
            len: 3,
            cycles: 16,
            execute: |cpu|{
                let addr = addr16(cpu);
                if cpu.regs.get_FZ() == true {
                    cpu.regs.PC = addr;
                } else {
                    let pc = cpu.regs.get_PC();
                    cpu.regs.set_PC(pc+2);
                }
            },
            jump: true,
        };
        cpu.opcodes[0xC0] = Opcode {
            name: "RET NZ",
            len: 1,
            cycles: 20,
            execute: |cpu| { if !cpu.regs.get_FZ() {cpu.regs.PC = PopStack(cpu);} else {cpu.regs.PC = cpu.regs.PC.wrapping_add(1);}},
            jump: true,
        };
        cpu.opcodes[0xC1] = Opcode {
            name: "POP BC",
            len: 1,
            cycles: 12,
            execute: POPbc,
            jump: false,
        };

        cpu.opcodes[0xC2] = Opcode {
            name: "JPNZ a16",
            len: 3,
            cycles: 16,
            execute: |cpu|{
                let addr = addr16(cpu);
                let pc = cpu.regs.get_PC();
                if !cpu.regs.get_FZ() { cpu.regs.set_PC(addr); } else {cpu.regs.set_PC(pc+2);}
            },
            jump: true,
        };
        cpu.opcodes[0xC3] = Opcode {
            name: "JP a16",
            len: 3,
            cycles: 16,
            execute: |cpu| {let addr = addr16(cpu); cpu.regs.PC = addr; },
            jump: true,
        };
        cpu.opcodes[0xC4] = Opcode {
            name: "CALL NZ a16",
            len: 3,
            cycles: 24,
            execute: |cpu|{
                let addr = addr16(cpu);
                if !cpu.regs.get_FZ() == true {
                    let next = cpu.regs.PC + 3;
                    PushStack(cpu, next);
                    cpu.regs.PC = addr;
                } else {
                    cpu.regs.PC = cpu.regs.PC.wrapping_add(3);
                }
            },
            jump: true,
        };
        cpu.opcodes[0xC5] = Opcode {
            name: "PUSH BC",
            len: 1,
            cycles: 16,
            execute: |cpu|{ PushStack(cpu, cpu.regs.get_BC()); },
            jump: false,
        };
        cpu.opcodes[0xC6] = Opcode {
            name: "ADD A,d8",
            len: 2,
            cycles: 8,
            execute: ADDad8,
            jump: false,
        };
        cpu.opcodes[0xC8] = Opcode {
            name: "RET Z",
            len: 1,
            cycles: 20,
            execute: |cpu| { if cpu.regs.get_FZ() {cpu.regs.PC = PopStack(cpu);} else {cpu.regs.PC = cpu.regs.PC.wrapping_add(1);}},
            jump: true,
        };
        cpu.opcodes[0xC9] = Opcode {
            name: "RET",
            len: 1,
            cycles: 16,
            execute: RET,
            jump: true,
        };
        cpu.opcodes[0xCC] = Opcode {
            name: "CALL Z a16",
            len: 3,
            cycles: 24,
            execute: |cpu|{
                if cpu.regs.get_FZ() {
                let addr = addr16(cpu);
                let next = cpu.regs.PC + 3;
                PushStack(cpu, next);
                cpu.regs.PC = addr;
                } else {
                cpu.regs.PC = cpu.regs.PC+3;
                };
            },
            jump: true,
        };
        cpu.opcodes[0xCD] = Opcode {
            name: "CALL a16",
            len: 3,
            cycles: 24,
            execute: |cpu|{
                let addr = addr16(cpu);
                let next = cpu.regs.PC + 3;
                PushStack(cpu, next);
                cpu.regs.PC = addr;
            },
            jump: true,
        };
        cpu.opcodes[0xCE] = Opcode {
            name: "ADC d8",
            len: 2,
            cycles: 8,
            execute: ADCad8,
            jump: false,
        };
        cpu.opcodes[0xD0] = Opcode {
            name: "RET NC",
            len: 1,
            cycles: 20,
            execute: RETNC,
            jump: true,
        };
        cpu.opcodes[0xD1] = Opcode {
            name: "POP DE",
            len: 1,
            cycles: 12,
            execute: POPde,
            jump: false,
        };
        cpu.opcodes[0xD5] = Opcode {
            name: "PUSH DE",
            len: 1,
            cycles: 16,
            execute: |cpu|{ PushStack(cpu, cpu.regs.get_DE()); },
            jump: false,
        };
        cpu.opcodes[0xD9] = Opcode {
            name: "RETI",
            len: 1,
            cycles: 20,
            execute: RETI,
            jump: true,
        };
        cpu.opcodes[0xD6] = Opcode {
            name: "SUB A,d8",
            len: 2,
            cycles: 8,
            execute: SUBad8,
            jump: false,
        };
        cpu.opcodes[0xD8] = Opcode {
            name: "RET NC",
            len: 1,
            cycles: 20,
            execute: RETNC,
            jump: true,
        };
        cpu.opcodes[0xDE] = Opcode {
            name: "SBC A, d8",
            len: 2,
            cycles: 8,
            execute: |cpu|{ let v = imm8(cpu); alu_sub(cpu, v, true); },
            jump: false,
        };

        cpu.opcodes[0xE0] = Opcode {
            name: "LDH (a8),A",
            len: 2,
            cycles: 12,
            execute: LDha8a,
            jump: false,
        };
        cpu.opcodes[0xE1] = Opcode {
            name: "POP HL",
            len: 1,
            cycles: 12,
            execute: POPhl,
            jump: false,
        };
        cpu.opcodes[0xE2] = Opcode {
            name: "LD (C), A",
            len: 1,
            cycles: 8,
            execute: LDpca,
            jump: false,
        };
        cpu.opcodes[0xE5] = Opcode {
            name: "PUSH HL",
            len: 1,
            cycles: 16,
            execute: |cpu|{ PushStack(cpu, cpu.regs.get_HL()); },
            jump: false,
        };
        cpu.opcodes[0xE6] = Opcode {
            name: "AND d8",
            len: 2,
            cycles: 8,
            execute: |cpu|{ let imm = imm8(cpu); alu_and(cpu, imm); },
            jump: false,
        };
        cpu.opcodes[0xE8] = Opcode {
            name: "DEC SP",
            len: 2,
            cycles: 16,
            execute: |cpu| { cpu.regs.SP = alu_add16imm(cpu, cpu.regs.SP);},
            jump: false,
        };
        cpu.opcodes[0xE9] = Opcode {
            name: "JP (HL)",
            len: 1,
            cycles: 4,
            execute: |cpu| { let addr = cpu.regs.get_HL(); cpu.regs.PC = addr;},
            jump: true,
        };
        cpu.opcodes[0xEA] = Opcode {
            name: "LD (a16),A",
            len: 3,
            cycles: 16,
            execute: LDa16a,
            jump: false,
        };
        cpu.opcodes[0xEE] = Opcode {
            name: "XOR d8",
            len: 2,
            cycles: 8,
            execute: XORd8,
            jump: false,
        };
        cpu.opcodes[0xEF] = Opcode {
            name: "RST 28h",
            len: 1,
            cycles: 16,
            execute: RST28h,
            jump: true,
        };
        cpu.opcodes[0xF0] = Opcode {
            name: "LDH A,(a8)",
            len: 2,
            cycles: 12,
            execute: |cpu| {let imm = imm8(cpu); cpu.regs.A = cpu.mem.read8(0xFF00+imm as u16); },
            jump: false,
        };
        cpu.opcodes[0xF1] = Opcode {
            name: "POP AF",
            len: 1,
            cycles: 12,
            execute: POPaf,
            jump: false,
        };
        cpu.opcodes[0xF2] = Opcode {
            name: "LD A,(C)",
            len: 1,
            cycles: 8,
            execute: |cpu| {let c = cpu.mem.read8(0xFF00+cpu.regs.C as u16); cpu.regs.A = c; },
            jump: false,
        };
        cpu.opcodes[0xF3] = Opcode {
            name: "DI",
            len: 1,
            cycles: 4,
            execute: DI,
            jump: false,
        };
        cpu.opcodes[0xF5] = Opcode {
            name: "PUSH AF",
            len: 1,
            cycles: 16,
            execute: |cpu|{ PushStack(cpu, cpu.regs.get_AF()); },
            jump: false,
        };
        cpu.opcodes[0xF6] = Opcode {
            name: "OR d8",
            len: 2,
            cycles: 8,
            execute: ORd8,
            jump: false,
        };
        cpu.opcodes[0xF8] = Opcode {
            name: "LD HL,SP+r8",
            len: 2,
            cycles: 12,
            execute: |cpu|{let r = alu_add16imm(cpu, cpu.regs.get_SP()); cpu.regs.set_HL(r);},
            jump: false,
        };
        cpu.opcodes[0xF9] = Opcode {
            name: "LD SP, HL",
            len: 1,
            cycles: 8,
            execute: |cpu|{let r = cpu.regs.get_HL(); cpu.regs.set_SP(r);},
            jump: false,
        };

        cpu.opcodes[0xFA] = Opcode {
            name: "LD A, (a16)",
            len: 3,
            cycles: 16,
            execute: LDaa16,
            jump: false,
        };
        cpu.opcodes[0xFB] = Opcode {
            name: "EI",
            len: 1,
            cycles: 4,
            execute: EI,
            jump: false,
        };
        cpu.opcodes[0xFE] = Opcode {
            name: "CP d8",
            len: 2,
            cycles: 8,
            execute: CPd8,
            jump: false,
        };
        cpu.opcodes[0xFF] = Opcode {
            name: "RST 38h",
            len: 1,
            cycles: 16,
            execute: RST38h,
            jump: true,
        };



        /************ Alternative (PREFIX) opcodes **************/
        cpu.alt_opcodes[0x11] = Opcode {
            name: "RL C",
            len: 2,
            cycles: 8,
            execute: RLc,
            jump: false,
        };
        cpu.alt_opcodes[0x17] = Opcode {
            name: "RL A",
            len: 2,
            cycles: 8,
            execute: RLa,
            jump: false,
        };
        cpu.alt_opcodes[0x18] = Opcode {
            name: "RR B",
            len: 2,
            cycles: 8,
            execute: |cpu|{cpu.regs.B = alu_rr(cpu, cpu.regs.B);},
            jump: false,
        };
        cpu.alt_opcodes[0x19] = Opcode {
            name: "RR C",
            len: 2,
            cycles: 8,
            execute: |cpu|{cpu.regs.C = alu_rr(cpu, cpu.regs.C);},
            jump: false,
        };
        cpu.alt_opcodes[0x1A] = Opcode {
            name: "RR D",
            len: 2,
            cycles: 8,
            execute: |cpu|{cpu.regs.D = alu_rr(cpu, cpu.regs.D);},
            jump: false,
        };
        cpu.alt_opcodes[0x1B] = Opcode {
            name: "RR E",
            len: 2,
            cycles: 8,
            execute: |cpu|{cpu.regs.E = alu_rr(cpu, cpu.regs.E);},
            jump: false,
        };
        cpu.alt_opcodes[0x1C] = Opcode {
            name: "RR H",
            len: 2,
            cycles: 8,
            execute: |cpu|{cpu.regs.H = alu_rr(cpu, cpu.regs.H);},
            jump: false,
        };
        cpu.alt_opcodes[0x1D] = Opcode {
            name: "RR L",
            len: 2,
            cycles: 8,
            execute: |cpu|{cpu.regs.L = alu_rr(cpu, cpu.regs.L);},
            jump: false,
        };
        cpu.alt_opcodes[0x41] = Opcode {
            name: "BIT 0,C",
            len: 2,
            cycles: 8,
            execute: |cpu| {alu_bit(cpu, cpu.regs.C, 0);},
            jump: false,
        };
        cpu.alt_opcodes[0x42] = Opcode {
            name: "BIT 0,D",
            len: 2,
            cycles: 8,
            execute: |cpu| {alu_bit(cpu, cpu.regs.D, 0);},
            jump: false,
        };
        cpu.alt_opcodes[0x47] = Opcode {
            name: "BIT 0,A",
            len: 2,
            cycles: 8,
            execute: |cpu| {alu_bit(cpu, cpu.regs.A, 0);},
            jump: false,
        };
        cpu.alt_opcodes[0x57] = Opcode {
            name: "BIT 2,A",
            len: 2,
            cycles: 8,
            execute: |cpu| {alu_bit(cpu, cpu.regs.A, 2);},
            jump: false,
        };
        cpu.alt_opcodes[0x6F] = Opcode {
            name: "BIT 5,A",
            len: 2,
            cycles: 8,
            execute: |cpu| {alu_bit(cpu, cpu.regs.A, 5);},
            jump: false,
        };
        cpu.alt_opcodes[0x77] = Opcode {
            name: "BIT 6,A",
            len: 2,
            cycles: 8,
            execute: BIT6a,
            jump: false,
        };
        cpu.alt_opcodes[0x79] = Opcode {
            name: "BIT 7,C",
            len: 2,
            cycles: 8,
            execute: |cpu| {alu_bit(cpu, cpu.regs.C, 7);},
            jump: false,
        };
        cpu.alt_opcodes[0x7C] = Opcode {
            name: "BIT 7,H",
            len: 2,
            cycles: 8,
            execute: BIT7h,
            jump: false,
        };
        cpu.alt_opcodes[0x7F] = Opcode {
            name: "BIT 7,A",
            len: 2,
            cycles: 8,
            execute: |cpu| {alu_bit(cpu, cpu.regs.A, 7);},
            jump: false,
        };
        cpu.alt_opcodes[0x37] = Opcode {
            name: "SWAP A",
            len: 2,
            cycles: 8,
            execute: SWAPa,
            jump: false,
        };
        cpu.alt_opcodes[0x38] = Opcode {
            name: "SRL B",
            len: 2,
            cycles: 8,
            execute: SRLb,
            jump: false,
        };
        cpu.alt_opcodes[0x39] = Opcode {
            name: "SRL C",
            len: 2,
            cycles: 8,
            execute: SRLc,
            jump: false,
        };
        cpu.alt_opcodes[0x3A] = Opcode {
            name: "SRL D",
            len: 2,
            cycles: 8,
            execute: SRLd,
            jump: false,
        };
        cpu.alt_opcodes[0x3B] = Opcode {
            name: "SRL E",
            len: 2,
            cycles: 8,
            execute: SRLe,
            jump: false,
        };
        cpu.alt_opcodes[0x3C] = Opcode {
            name: "SRL H",
            len: 2,
            cycles: 8,
            execute: SRLh,
            jump: false,
        };
        cpu.alt_opcodes[0x3D] = Opcode {
            name: "SRL L",
            len: 2,
            cycles: 8,
            execute: SRLl,
            jump: false,
        };
        cpu.alt_opcodes[0x3F] = Opcode {
            name: "SRL A",
            len: 2,
            cycles: 8,
            execute: SRLa,
            jump: false,
        };
        cpu.alt_opcodes[0x46] = Opcode {
            name: "BIT 0, (HL)",
            len: 2,
            cycles: 16,
            execute: |cpu| {let hl = cpu.mem.read8(cpu.regs.get_HL()); alu_bit(cpu, hl, 0);},
            jump: false,
        };
        cpu.alt_opcodes[0x87] = Opcode {
            name: "RES 0, A",
            len: 2,
            cycles: 8,
            execute: |cpu|{cpu.regs.A = cpu.regs.A & !(1 << 0);},
            jump: false,
        };
        cpu.alt_opcodes[0x8B] = Opcode {
            name: "RES 1, E",
            len: 2,
            cycles: 8,
            execute: |cpu|{cpu.regs.E = cpu.regs.E & !(1 << 1);},
            jump: false,
        };
        cpu.alt_opcodes[0x8E] = Opcode {
            name: "RES 1, (HL)",
            len: 2,
            cycles: 16,
            execute: |cpu|{let a = cpu.regs.get_HL(); let v = cpu.mem.read8(a) & !(1 << 1); cpu.mem.write8(a, v);},
            jump: false,
        };
        cpu.alt_opcodes[0x8F] = Opcode {
            name: "RES 1, A",
            len: 2,
            cycles: 8,
            execute: |cpu|{cpu.regs.A = cpu.regs.A & !(1 << 1);},
            jump: false,
        };
        cpu.alt_opcodes[0x96] = Opcode {
            name: "RES 2, (HL)",
            len: 2,
            cycles: 16,
            execute: |cpu|{let a = cpu.regs.get_HL(); let v = cpu.mem.read8(a) & !(1 << 2); cpu.mem.write8(a, v);},
            jump: false,
        };
        cpu.alt_opcodes[0x97] = Opcode {
            name: "RES 2, A",
            len: 2,
            cycles: 8,
            execute: |cpu|{cpu.regs.A = cpu.regs.A & !(1 << 2);},
            jump: false,
        };
        cpu.alt_opcodes[0x9E] = Opcode {
            name: "RES 3, (HL)",
            len: 2,
            cycles: 8,
            execute: |cpu|{let a = cpu.regs.get_HL(); let v = cpu.mem.read8(a) & !(1 << 3); cpu.mem.write8(a, v);},
            jump: false,
        };
        cpu.alt_opcodes[0x9F] = Opcode {
            name: "RES 3, A",
            len: 2,
            cycles: 8,
            execute: |cpu|{cpu.regs.A = cpu.regs.A & !(1 << 3);},
            jump: false,
        };
        cpu.alt_opcodes[0xA6] = Opcode {
            name: "RES 4, (HL)",
            len: 2,
            cycles: 16,
            execute: |cpu|{let a = cpu.regs.get_HL(); let v = cpu.mem.read8(a) & !(1 << 4); cpu.mem.write8(a, v);},
            jump: false,
        };
        cpu.alt_opcodes[0xA7] = Opcode {
            name: "RES 4, A",
            len: 2,
            cycles: 8,
            execute: |cpu|{cpu.regs.A = cpu.regs.A & !(1 << 4);},
            jump: false,
        };
        cpu.alt_opcodes[0xAE] = Opcode {
            name: "RES 5, (HL)",
            len: 2,
            cycles: 16,
            execute: |cpu|{let a = cpu.regs.get_HL(); let v = cpu.mem.read8(a) & !(1 << 5); cpu.mem.write8(a, v);},
            jump: false,
        };
        cpu.alt_opcodes[0xAF] = Opcode {
            name: "RES 5, A",
            len: 2,
            cycles: 8,
            execute: |cpu|{cpu.regs.A = cpu.regs.A & !(1 << 5);},
            jump: false,
        };
        cpu.alt_opcodes[0xBE] = Opcode {
            name: "RES 7, (HL)",
            len: 2,
            cycles: 16,
            execute: |cpu|{let a = cpu.regs.get_HL(); let v = cpu.mem.read8(a) & !(1 << 7); cpu.mem.write8(a, v);},
            jump: false,
        };
        cpu.alt_opcodes[0xC7] = Opcode {
            name: "SET 0, A",
            len: 2,
            cycles: 8,
            execute: |cpu|{cpu.regs.A = cpu.regs.A | (1 << 0);},
            jump: false,
        };
        cpu.alt_opcodes[0xE6] = Opcode {
            name: "SET 4, (HL)",
            len: 2,
            cycles: 8,
            execute: |cpu|{
                let hl = cpu.regs.get_HL();
                let mut v =  cpu.mem.read8(hl);
                v|=1<<4;
                cpu.mem.write8(hl, v);
            },
            jump: false,
        };
        cpu.alt_opcodes[0xFE] = Opcode {
            name: "SET 7, (HL)",
            len: 2,
            cycles: 8,
            execute: |cpu|{
                let hl = cpu.regs.get_HL();
                let mut v =  cpu.mem.read8(hl);
                v|=1<<7;
                cpu.mem.write8(hl, v);
            },
            jump: false,
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

    pub fn get_opcode_args(&mut self, codestr: &str) -> String {
        let mut code_str = String::from(codestr);
        if code_str.contains("r8") {
            code_str = code_str.replace("r8", &String::from(format!("0x{:02X}",imm8(self) as i8)));
        }
        if code_str.contains("d8") {
            code_str = code_str.replace("d8", &String::from(format!("0x{:02X}",imm8(self) as i8)));
        }
        if code_str.contains("a16") {
            code_str = code_str.replace("a16", &String::from(format!("0x{:04X}",imm16(self) as i16)));
        }
        if code_str.contains("d16") {
            code_str = code_str.replace("d16", &String::from(format!("0x{:04X}",imm16(self) as i16)));
        }
        if code_str.contains("(HL)") {
            let hl = self.mem.read8(self.regs.get_HL());
            code_str = code_str.replace("(HL)", &String::from(format!("0x{:02X}", hl as i8)));
        }
        if code_str.contains("(HL-)") {
            let hl = self.regs.get_HL();
            code_str = code_str.replace("(HL-)", &String::from(format!("(0x{:04X})", hl as u16)));
        }

        code_str
    }

    pub fn print_status(&mut self) {
        let code = self.mem.read8(self.regs.PC) as usize;
        let alt_code = self.mem.read8(self.regs.PC+1) as usize;

        if code == 0xCB {
            println!("PC {:04X} opcode {:02X} ALT {:02X} ", self.regs.PC, code, alt_code);
        } else {
            println!("PC {:04X} opcode {:02X}", self.regs.PC, code);
        }
        println!("==== CPU ====");
        println!("A : {:02X}\tB : {:02X}\tC : {:02X}\tD : {:02X}", self.regs.A, self.regs.B, self.regs.C, self.regs.D);
        println!("E : {:02X}\tF : {:02X}\tH : {:02X}\tL : {:02X}", self.regs.E, self.regs.F, self.regs.H, self.regs.L);
        println!("PC: {:04X} SP: {:04X}  Z:{} N:{} H:{} C:{}", self.regs.get_PC(), self.regs.get_SP(),
        self.regs.get_FZ(), self.regs.get_FN(),self.regs.get_FH(),self.regs.get_FC());
        println!("----------------------------------------");
    }

    pub fn print_status_small(&mut self) {
        let code = self.mem.read8(self.regs.PC) as usize;
        let alt_code = self.mem.read8(self.regs.PC+1) as usize;
        let opcode;
        let codestr =
            if code == 0xCB {
                opcode = self.alt_opcodes[alt_code];
                format!("{:02X} {:02X}", code, alt_code)
            } else {
                opcode = self.opcodes[code];
                format!("{:02X}", code)
            };
        let foo = (self.regs.get_SP(), self.regs.get_FZ(), self.regs.get_FN(),self.regs.get_FH(),self.regs.get_FC());
        let disas = self.get_opcode_args(opcode.name);
        println!("{:04X}: {: <16}\t{}\tA {:02X} B {:02X} C {:02X} D {:02X} E {:02X} F {:02X} H {:02X} L {:02X}\tSP: {:04X} Z:{: <5} N:{: <5} H:{: <5} C:{: <5}", self.regs.PC, disas, codestr,
                 self.regs.A,self.regs.B,self.regs.C,self.regs.D,
                 self.regs.E,self.regs.F,self.regs.H,self.regs.L,
                 foo.0, foo.1, foo.2, foo.3, foo.4
                );
    }
    pub fn print_dump(&mut self) {
        let pc = self.regs.get_PC();
        println!("A: {:02X} F: {:02X} B: {:02X} C: {:02X} D: {:02X} E: {:02X} H: {:02X} L: {:02X} SP: {:04X} PC: 00:{:04X} ({:02X} {:02X} {:02X} {:02X})",
                 self.regs.A, self.regs.F, self.regs.B,self.regs.C,self.regs.D,
                 self.regs.E,self.regs.H,self.regs.L, self.regs.get_SP(), self.regs.get_PC(),
                 self.mem.read8(pc), self.mem.read8(pc+1),self.mem.read8(pc+2),self.mem.read8(pc+3));

    }
    pub fn interrupts_enabled(&mut self) -> bool {
        self.regs.I
    }

    pub fn irq_vblank(&mut self) {
        DI(self);
        let addr = self.regs.PC;
        PushStack(self, addr);
        self.regs.PC = 0x0040;
    }

    pub fn reset(&mut self) {
        println!("JYJY RESET");
        self.regs.PC = 0x0000;
    }

    pub fn step(&mut self) -> u8 {
        let code = self.mem.read8(self.regs.PC) as usize;

        let opcode;
        if code == 0xCB {
            let code = self.mem.read8(self.regs.PC+1) as usize;
            opcode = self.alt_opcodes[code];
        } else {
            opcode = self.opcodes[code];
        }
        if self.regs.PC > 0x00FF {
  //          self.print_status_small();
        }
        if self.regs.PC > 0x00FF {
            self.print_dump();
        }

        if self.regs.PC == 0xC83D {
//        process::exit(3);
        }

        (opcode.execute)(self);


        self.total_cyles = self.total_cyles + opcode.cycles as u64;
        if !opcode.jump {
            self.regs.PC = self.regs.PC.wrapping_add(opcode.len);
        }

        if self.mem.read8(0xFF02) == 0x81 {
            let c = self.mem.read8(0xFF01);
            //println!("SERIAL got {}", c as char);
//            print!("{}", c as char);
            self.mem.write8(0xff02, 0x0);
        }
        opcode.cycles as u8
    }


}
