// Sharp LR35902 CPU emulator
#![allow(non_snake_case)]
#![allow(dead_code)]

use std::process;
use std::thread::sleep;
use std::time::Duration;
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
    //sleep(Duration::from_secs(5));
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
    cpu.regs.set_FH((a & 0x07FF) + (b & 0x07FF) > 0x07FF);
    cpu.regs.set_FN(false);
    cpu.regs.set_FC(a > 0xFFFF - b);
    cpu.regs.set_HL(r);
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
    let r = a >> 1;
    alu_srflagupdate(cpu, r, c);
    r
}
fn alu_srl(cpu: &mut Cpu, a: u8) -> u8 {
    let c = a & 0x01 == 0x01;
    let r = (a >> 1) | (a & 0x80);
    alu_srflagupdate(cpu, r, c);
    r
}


pub fn NOP(_cpu: &mut Cpu) {
    debug!("NOP")
}
pub fn XORd8(cpu: &mut Cpu) {
    let imm = imm8(cpu);
    alu_xor(cpu, imm);
    debug!("XOR {:02X}", imm);
}
pub fn XORc(cpu: &mut Cpu) {
    alu_xor(cpu, cpu.regs.C);
    debug!("XOR C");
}
pub fn XORa(cpu: &mut Cpu) {
    alu_xor(cpu, cpu.regs.A);
    debug!("XOR A");
}
pub fn XOR_hl(cpu: &mut Cpu) {
    let hl = cpu.mem.read8(cpu.regs.get_HL());
    alu_xor(cpu, hl);
    debug!("XOR A, [HL]");
}
pub fn ORd(cpu: &mut Cpu) {
    alu_or(cpu, cpu.regs.D);
    debug!("OR D");
}
pub fn ORc(cpu: &mut Cpu) {
    alu_or(cpu, cpu.regs.C);
    debug!("OR C");
}
pub fn ORb(cpu: &mut Cpu) {
    alu_or(cpu, cpu.regs.B);
    debug!("OR B");
}
pub fn ORa(cpu: &mut Cpu) {
    alu_or(cpu, cpu.regs.A);
    debug!("OR A");
}
pub fn ORhl(cpu: &mut Cpu) {
    let v = cpu.mem.read8(cpu.regs.get_HL());
    alu_xor(cpu, v);
    debug!("OR (hl)");
}
pub fn ORd8(cpu: &mut Cpu) {
    let v = imm8(cpu);
    alu_or(cpu, v);
    debug!("OR imm8");
}
pub fn ANDc(cpu: &mut Cpu) {
    alu_and(cpu, cpu.regs.C);
    debug!("AND C");
}
pub fn ANDa(cpu: &mut Cpu) {
    alu_and(cpu, cpu.regs.A);
    debug!("AND A");
}
pub fn ANDhl(cpu: &mut Cpu) {
    let hl = cpu.mem.read8(cpu.regs.get_HL());
    alu_and(cpu, hl);
    debug!("AND A");
}
pub fn ANDd8(cpu: &mut Cpu) {
    let imm = imm8(cpu);
    alu_and(cpu, imm);
    debug!("AND {:02}", imm);
}
pub fn SUBad8(cpu: &mut Cpu) {
    let imm = imm8(cpu);
    alu_sub(cpu, imm, false);
    debug!("SUB A, {:02X}", imm);
}

pub fn ADCac(cpu: &mut Cpu) {
    alu_add(cpu, cpu.regs.C, true);
    debug!("ADC A, C {:02X}", cpu.regs.C);
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
pub fn ADDhlbc(cpu: &mut Cpu) {
    let bc = cpu.regs.get_BC();
    alu_add16(cpu, bc);
    debug!("ADD HL, BC");
}
pub fn ADDhlhl(cpu: &mut Cpu) {
    let hl = cpu.regs.get_HL();
    alu_add16(cpu, hl);

    debug!("ADD HL,HL");
}
pub fn ADDhlsp(cpu: &mut Cpu) {
    let sp = cpu.regs.get_SP();
    alu_add16(cpu, sp);
    debug!("ADD HL,HL");
}
pub fn DECc(cpu: &mut Cpu) {
    cpu.regs.C = alu_dec(cpu, cpu.regs.C);
    debug!("DEC C, F is {:b}", cpu.regs.F);
}
pub fn DECb(cpu: &mut Cpu) {
    cpu.regs.B = alu_dec(cpu, cpu.regs.B);
    debug!("DEC B");
}
pub fn DECh(cpu: &mut Cpu) {
    cpu.regs.H = alu_dec(cpu, cpu.regs.H);
    debug!("DEC H");
}
pub fn DECa(cpu: &mut Cpu) {
    cpu.regs.A = alu_dec(cpu, cpu.regs.A);
    debug!("DEC A");
}
pub fn DECe(cpu: &mut Cpu) {
    cpu.regs.E = alu_dec(cpu, cpu.regs.E);
    debug!("DEC E");
}
pub fn DECl(cpu: &mut Cpu) {
    cpu.regs.L = alu_dec(cpu, cpu.regs.L);
    debug!("DEC D");
}
pub fn DECd(cpu: &mut Cpu) {
    cpu.regs.D = alu_dec(cpu, cpu.regs.D);
    debug!("DEC D");
}
pub fn DECbc(cpu: &mut Cpu) {
    let bc = cpu.regs.get_BC();
    cpu.regs.set_BC(bc.wrapping_sub(1));
    debug!("DEC BC");
}
pub fn DECde(cpu: &mut Cpu) {
    let de = cpu.regs.get_DE();
    cpu.regs.set_BC(de.wrapping_sub(1));
    debug!("DEC DE");
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
pub fn DEC_hl(cpu: &mut Cpu) {
    let hl = cpu.mem.read8(cpu.regs.get_HL());
    cpu.mem.write8(cpu.regs.get_HL(), hl.wrapping_sub(1));
    debug!("DEC (HL)");
}
pub fn INCde(cpu: &mut Cpu) {
    let de = cpu.regs.get_DE();
    cpu.regs.set_DE(de.wrapping_add(1));
    debug!("INC DE");
}
pub fn INCbc(cpu: &mut Cpu) {
    let bc = cpu.regs.get_BC();
    cpu.regs.set_BC(bc.wrapping_add(1));
    debug!("INC BC");
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
pub fn CPa(cpu: &mut Cpu) {
    alu_cp(cpu, cpu.regs.A);
    debug!("CP A")
}
pub fn CPhl(cpu: &mut Cpu) {
    let hl = cpu.mem.read8(cpu.regs.get_HL());
    alu_cp(cpu, hl);
    debug!("CP HL");
    let imm = imm8(cpu);
}
pub fn CPL(cpu: &mut Cpu) {
    let A = cpu.regs.A;
    cpu.regs.A = !A;
    cpu.regs.set_FN(true);
    cpu.regs.set_FH(true);
    debug!("CPL")
}
pub fn CPd8(cpu: &mut Cpu) {
    let imm = imm8(cpu);
    alu_cp(cpu, imm);
    debug!("CP {:02X}", imm)
}

pub fn RRb(cpu: &mut Cpu) {
    cpu.regs.B = alu_rr(cpu, cpu.regs.B);
    debug!("RR B");
}
pub fn RRc(cpu: &mut Cpu) {
    cpu.regs.C = alu_rr(cpu, cpu.regs.C);
    debug!("RR C");
}
pub fn RRd(cpu: &mut Cpu) {
    cpu.regs.D = alu_rr(cpu, cpu.regs.D);
    debug!("RR D");
}
pub fn RRe(cpu: &mut Cpu) {
    cpu.regs.E = alu_rr(cpu, cpu.regs.E);
    debug!("RR E");
}
pub fn RRh(cpu: &mut Cpu) {
    cpu.regs.H = alu_rr(cpu, cpu.regs.H);
    debug!("RR H");
}
pub fn RRl(cpu: &mut Cpu) {
    cpu.regs.L = alu_rr(cpu, cpu.regs.L);
    debug!("RR L");
}
pub fn RRa(cpu: &mut Cpu) {
    cpu.regs.A = alu_rr(cpu, cpu.regs.A);
    debug!("RR A");
}
pub fn RRCa(cpu: &mut Cpu) {
    cpu.regs.A = alu_rrc(cpu, cpu.regs.A);
    cpu.regs.set_FZ(false);
    debug!("RRC A");
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
pub fn LDhd8(cpu: &mut Cpu) {
    let imm = imm8(cpu);
    cpu.regs.H = imm;
    debug!("LD H, {:04X}", imm)
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
pub fn LDhld8(cpu: &mut Cpu) {
    let imm = imm8(cpu);
    cpu.mem.write8(cpu.regs.get_HL(), imm);
    debug!("LD (HL), {:02X}", imm)
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
pub fn LDdea(cpu: &mut Cpu) {
    cpu.mem.write8(cpu.regs.get_DE(), cpu.regs.A);
    debug!("LD (DE), A")
}
pub fn LDda(cpu: &mut Cpu) {
    cpu.regs.D = cpu.regs.A;
    debug!("LD D, A")
}
pub fn LDae(cpu: &mut Cpu) {
    cpu.regs.A = cpu.regs.B;
    debug!("LD A, E")
}
pub fn LDha(cpu: &mut Cpu) {
    cpu.regs.H = cpu.regs.A;
    debug!("LD H, A")
}
pub fn LDla(cpu: &mut Cpu) {
    cpu.regs.L = cpu.regs.A;
    debug!("LD L, A")
}
pub fn LDad(cpu: &mut Cpu) {
    cpu.regs.A = cpu.regs.B;
    debug!("LD A, D")
}
pub fn LDab(cpu: &mut Cpu) {
    cpu.regs.A = cpu.regs.B;
    debug!("LD A, B")
}
pub fn LDac(cpu: &mut Cpu) {
    cpu.regs.A = cpu.regs.C;
    debug!("LD A, C")
}
pub fn LDhb(cpu: &mut Cpu) {
    cpu.regs.H = cpu.regs.B;
    debug!("LD H, B")
}
pub fn LDlh(cpu: &mut Cpu) {
    cpu.regs.L = cpu.regs.H;
    debug!("LD L, H")
}
pub fn LDca(cpu: &mut Cpu) {
    cpu.regs.C = cpu.regs.A;
    debug!("LD C, A")
}
pub fn LDpca(cpu: &mut Cpu) {
    let C = cpu.regs.C as u16;
    cpu.mem.write8(0xFF00 + C, cpu.regs.A);
    debug!("LD (C), A")
}
pub fn LDded16(cpu: &mut Cpu) {
    let imm = imm16(cpu);
    cpu.regs.set_DE(imm);
    debug!("LD DE, {:04X}", imm)
}
pub fn LDbcd16(cpu: &mut Cpu) {
    let imm = imm16(cpu);
    cpu.regs.set_BC(imm);
    debug!("LD BC, {:04X}", imm)
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
pub fn LDcd8(cpu: &mut Cpu) {
    let imm = imm8(cpu);
    cpu.regs.C = imm;
    debug!("LD C, {:02X}", imm)
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
pub fn LDbd8(cpu: &mut Cpu) {
    let imm = imm8(cpu);
    cpu.regs.B = imm;
    debug!("LD B, {:02X}", imm)
}
pub fn LDha8a(cpu: &mut Cpu) {
    let imm = imm8(cpu);
    cpu.mem.write8(0xFF00+imm as u16, cpu.regs.A);
    debug!("LDH (FF{:02X}), A", imm)
}
pub fn LDhaa8(cpu: &mut Cpu) {
    let imm = imm8(cpu);
    cpu.regs.A = cpu.mem.read8(0xFF00+imm as u16);
    debug!("LDH A, ({:02X})", imm)
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
pub fn LDdhl(cpu: &mut Cpu) {
    let m = cpu.mem.read8(cpu.regs.get_HL());
    cpu.regs.D = m;
    debug!("LD D, {:04X}", m);
}
pub fn JPa16(cpu: &mut Cpu) {
    let addr = addr16(cpu);
    cpu.regs.PC = addr;
    debug!("JP {:04X}", addr)
}
pub fn JPZa16(cpu: &mut Cpu) {
    let addr = addr16(cpu);
    if cpu.regs.get_FZ() == true {
        cpu.regs.PC = addr;
    }
    debug!("JP {:04X}", addr)
}
pub fn JRr8(cpu: &mut Cpu) {
    let offset = cpu.regs.PC + 2;
    let v      = imm8(cpu) as i8;
    cpu.regs.PC = if v < 0 { offset - (-v) as u16 } else { offset + v as u16 };
    debug!("JR {:04X} (PC (after +{:}))", cpu.regs.PC, v)
}
pub fn JPhl(cpu: &mut Cpu) {
    let addr = cpu.regs.get_HL();
    cpu.regs.PC = addr;
    debug!("JP ({:04X})", addr)
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
    cpu.regs.set_AF(sp);
    debug!("POP AF");
}
pub fn PUSHde(cpu: &mut Cpu) {
    let v = cpu.regs.get_DE();
    PushStack(cpu, v);
    debug!("PUSH DE");
}
pub fn PUSHbc(cpu: &mut Cpu) {
    let v = cpu.regs.get_BC();
    PushStack(cpu, v);
    debug!("PUSH BC");
}
pub fn PUSHaf(cpu: &mut Cpu) {
    let v = cpu.regs.get_AF();
    PushStack(cpu, v);
    debug!("PUSH AF");
}
pub fn PUSHhl(cpu: &mut Cpu) {
    let v = cpu.regs.get_HL();
    PushStack(cpu, v);
    debug!("PUSH HL");
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
    let offset = cpu.regs.PC + 2;
    let v:i8      = imm8(cpu) as i8;
    if cpu.regs.get_FC() == false {
        cpu.regs.PC = if v < 0 { offset - (-v) as u16 } else { offset + v as u16 }
    } else {
        cpu.regs.PC = offset;
    }
    debug!("JRNC {:02X}", v)
}
pub fn JRnzr8(cpu: &mut Cpu) {
    let offset = cpu.regs.PC + 2;
    let v:i8      = imm8(cpu) as i8;
    if cpu.regs.get_FZ() == false {
        cpu.regs.PC = if v < 0 { offset - (-v) as u16 } else { offset + v as u16 }
    } else {
        cpu.regs.PC = offset;
    }
    debug!("JRNZ {:02X}", v)
}
pub fn JRcr8(cpu: &mut Cpu) {
    let offset = cpu.regs.PC + 2;
    let v:i8      = imm8(cpu) as i8;
    if cpu.regs.get_FC() == true {
        cpu.regs.PC = if v < 0 { offset - (-v) as u16 } else { offset + v as u16 }
    } else {
        cpu.regs.PC = offset;
    }
    debug!("JR C {:02X}", v)
}
pub fn JRzr8(cpu: &mut Cpu) {
    let offset = cpu.regs.PC + 2;
    let v:i8      = imm8(cpu) as i8;
    if cpu.regs.get_FZ() == true {
        cpu.regs.PC = if v < 0 { offset - (-v) as u16 } else { offset + v as u16 }
    } else {
        cpu.regs.PC = offset;
    }
    debug!("JRZ {:02X}", v)
}
pub fn CALLa16(cpu: &mut Cpu) {
    let addr = addr16(cpu);
    let next = cpu.regs.PC + 3;
    PushStack(cpu, next);
    cpu.regs.PC = addr;
    debug!("CALL {:04X}", addr)
}
pub fn CALLNZa16(cpu: &mut Cpu) {
    let addr = addr16(cpu);
    if cpu.regs.get_FZ() == true {
        let next = cpu.regs.PC + 3;
        PushStack(cpu, next);
        cpu.regs.PC = addr;
    } else {
        cpu.regs.PC = cpu.regs.PC.wrapping_add(3);
    }
    debug!("CALL {:04X}", addr)
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
pub fn RETNC(cpu: &mut Cpu) {
    if cpu.regs.get_FC() == true {
        let addr = PopStack(cpu);
        cpu.regs.PC = addr;
        debug!("RET NC (-> {:04X})", addr)
    } else {
        cpu.regs.PC = cpu.regs.PC.wrapping_add(1);
        debug!("RET NC (-> continue)")
    }
}
pub fn RETZ(cpu: &mut Cpu) {
    let mut addr = 0;
    if cpu.regs.get_FZ() == false {
        addr = PopStack(cpu);
        cpu.regs.PC = addr;
    } else {
        cpu.regs.PC = cpu.regs.PC.wrapping_add(1);
    }
    debug!("RET Z (-> {:04X})", addr)
}
pub fn RETNZ(cpu: &mut Cpu) {
    let mut addr = 0;
    if cpu.regs.get_FZ() == true {
        addr = PopStack(cpu);
        cpu.regs.PC = addr;
    } else {
        cpu.regs.PC = cpu.regs.PC.wrapping_add(1);
    }
    debug!("RET NZ (-> {:04X})", addr)
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
pub fn SET7hl(cpu: &mut Cpu) {
    let hl = cpu.regs.get_HL();
    let mut v =  cpu.mem.read8(hl);
    v|=0b1000_0000;
    cpu.mem.write8(hl, v);
    debug!("SET 7, HL")
}
pub fn BIT0c(cpu: &mut Cpu) {
    let v = cpu.regs.C&0b0000_0001;
    cpu.regs.set_FZ(v==0);
    cpu.regs.set_FN(false);
    cpu.regs.set_FH(true);
}
pub fn BIT5a(cpu: &mut Cpu) {
    let v = cpu.regs.A&0b0010_0000;
    cpu.regs.set_FZ(v==0);
    cpu.regs.set_FN(false);
    cpu.regs.set_FH(true);
}
pub fn BIT6a(cpu: &mut Cpu) {
    let v = cpu.regs.A&0b0100_0000;
    cpu.regs.set_FZ(v==0);
    cpu.regs.set_FN(false);
    cpu.regs.set_FH(true);
}
pub fn BIT7h(cpu: &mut Cpu) {
    let v = cpu.regs.A&0b1000_0000;
    cpu.regs.set_FZ(v==0);
    cpu.regs.set_FN(false);
    cpu.regs.set_FH(true);
}
pub fn RLCa(cpu: &mut Cpu) {

    let c = cpu.regs.A >> 7;
    cpu.regs.A = (cpu.regs.A << 1) | c;

    cpu.regs.set_FZ(cpu.regs.A == 0);
    cpu.regs.set_FN(false);
    cpu.regs.set_FH(false);
    cpu.regs.set_FC(c==1);
}

pub fn RES0A(cpu: &mut Cpu) {
    let mut a = cpu.regs.A;
    a &= 0b1111_1110;
    cpu.regs.A = a;
}
pub fn RES1E(cpu: &mut Cpu) {
    let mut e = cpu.regs.E;
    e &= 0b1111_1101;
    cpu.regs.E = e;
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
    debug!("Pushing {:04X} into stack at {:04X}", v, cpu.regs.SP);
    cpu.mem.write16(cpu.regs.SP, v);
    cpu.regs.SP = cpu.regs.SP.wrapping_sub(2);
}
pub fn PopStack(cpu: &mut Cpu) -> u16 {
    cpu.regs.SP = cpu.regs.SP.wrapping_add(2);
    let addr = cpu.mem.read16(cpu.regs.SP);
    debug!("Poping {:04X} from stack at {:04X}", addr, cpu.regs.SP);
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
            execute: NOP,
            jump: false,
        };
        cpu.opcodes[0x01] = Opcode {
            name: "LD BC, d16",
            len: 3,
            cycles: 12,
            execute: LDbcd16,
            jump: false,
        };
        cpu.opcodes[0x03] = Opcode {
            name: "INC BC",
            len: 1,
            cycles: 8,
            execute: INCbc,
            jump: false,
        };
        cpu.opcodes[0x05] = Opcode {
            name: "DEC B",
            len: 1,
            cycles: 4,
            execute: DECb,
            jump: false,
        };
        cpu.opcodes[0x06] = Opcode {
            name: "LD B,d8",
            len: 2,
            cycles: 8,
            execute: LDbd8,
            jump: false,
        };
        cpu.opcodes[0x07] = Opcode {
            name: "RLCA",
            len: 1,
            cycles: 4,
            execute: RLCa,
            jump: false,
        };
        cpu.opcodes[0x09] = Opcode {
            name: "ADD HL, BC",
            len: 1,
            cycles: 8,
            execute: ADDhlbc,
            jump: false,
        };
        cpu.opcodes[0x0B] = Opcode {
            name: "DEC BC",
            len: 1,
            cycles: 4,
            execute: DECbc,
            jump: false,
        };
        cpu.opcodes[0x0C] = Opcode {
            name: "INC C",
            len: 1,
            cycles: 4,
            execute: INCc,
            jump: false,
        };
        cpu.opcodes[0x0D] = Opcode {
            name: "DEC C",
            len: 1,
            cycles: 4,
            execute: DECc,
            jump: false,
        };
        cpu.opcodes[0x0E] = Opcode {
            name: "LD C, d8",
            len: 2,
            cycles: 8,
            execute: LDcd8,
            jump: false,
        };
        cpu.opcodes[0x0F] = Opcode {
            name: "RRCA",
            len: 1,
            cycles: 4,
            execute: RRCa,
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
            execute: INCd,
            jump: false,
        };
        cpu.opcodes[0x15] = Opcode {
            name: "DEC D",
            len: 1,
            cycles: 4,
            execute: DECd,
            jump: false,
        };
        cpu.opcodes[0x16] = Opcode {
            name: "LD D, d8",
            len: 2,
            cycles: 8,
            execute: LDdd8,
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
            execute: DECde,
            jump: false,
        };
        cpu.opcodes[0x1C] = Opcode {
            name: "INC E",
            len: 1,
            cycles: 4,
            execute: INCe,
            jump: false,
        };
        cpu.opcodes[0x1D] = Opcode {
            name: "DEC E",
            len: 1,
            cycles: 4,
            execute: DECe,
            jump: false,
        };
        cpu.opcodes[0x1F] = Opcode {
            name: "RRA",
            len: 1,
            cycles: 4,
            execute: RRa,
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
            execute: INCh,
            jump: false,
        };
        cpu.opcodes[0x25] = Opcode {
            name: "DEC H",
            len: 1,
            cycles: 4,
            execute: DECh,
            jump: false,
        };
        cpu.opcodes[0x26] = Opcode {
            name: "LD H, d8",
            len: 2,
            cycles: 8,
            execute: LDhd8,
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
            execute: ADDhlhl,
            jump: false,
        };
        cpu.opcodes[0x2A] = Opcode {
            name: "LDI A, (HL+)",
            len: 1,
            cycles: 8,
            execute: LDIahlp,
            jump: false,
        };
        cpu.opcodes[0x2C] = Opcode {
            name: "INC L",
            len: 1,
            cycles: 4,
            execute: INCl,
            jump: false,
        };
        cpu.opcodes[0x2D] = Opcode {
            name: "DEC L",
            len: 1,
            cycles: 4,
            execute: DECl,
            jump: false,
        };
        cpu.opcodes[0x2F] = Opcode {
            name: "CPL",
            len: 1,
            cycles: 4,
            execute: CPL,
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
            name: "LDD (HL), a",
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
            execute: DEC_hl,
            jump: false,
        };
        cpu.opcodes[0x36] = Opcode {
            name: "LD (HL), d8",
            len: 2,
            cycles: 12,
            execute: LDhld8,
            jump: false,
        };
        cpu.opcodes[0x38] = Opcode {
            name: "JR C r8",
            len: 2,
            cycles: 12,
            execute: JRcr8,
            jump: false,
        };
        cpu.opcodes[0x39] = Opcode {
            name: "ADD HL, SP",
            len: 1,
            cycles: 8,
            execute: ADDhlsp,
            jump: false,
        };
        cpu.opcodes[0x3C] = Opcode {
            name: "INC A",
            len: 1,
            cycles: 4,
            execute: INCa,
            jump: false,
        };
        cpu.opcodes[0x3D] = Opcode {
            name: "DEC A",
            len: 1,
            cycles: 4,
            execute: DECa,
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
            execute: LDbb,
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
            execute: LDba,
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
            execute: LDca,
            jump: false,
        };
        cpu.opcodes[0x57] = Opcode {
            name: "LD D, A",
            len: 1,
            cycles: 4,
            execute: LDda,
            jump: false,
        };
        cpu.opcodes[0x56] = Opcode {
            name: "LD D, (HL)",
            len: 1,
            cycles: 8,
            execute: LDdhl,
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
            execute: LDhb,
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
            execute: LDha,
            jump: false,
        };
        cpu.opcodes[0x6C] = Opcode {
            name: "LD L, H",
            len: 1,
            cycles: 4,
            execute: LDlh,
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
            execute: LDla,
            jump: false,
        };
        cpu.opcodes[0x70] = Opcode {
            name: "LD (HL),B",
            len: 1,
            cycles: 8,
            execute: LDhlb,
            jump: false,
        };
        cpu.opcodes[0x71] = Opcode {
            name: "LD (HL),C",
            len: 1,
            cycles: 8,
            execute: LDhlc,
            jump: false,
        };
        cpu.opcodes[0x72] = Opcode {
            name: "LD (HL),D",
            len: 1,
            cycles: 8,
            execute: LDhld,
            jump: false,
        };
        cpu.opcodes[0x77] = Opcode {
            name: "LD (HL),A",
            len: 1,
            cycles: 8,
            execute: LDhla,
            jump: false,
        };
        cpu.opcodes[0x78] = Opcode {
            name: "LD A, B",
            len: 1,
            cycles: 4,
            execute: LDab,
            jump: false,
        };
        cpu.opcodes[0x7A] = Opcode {
            name: "LD A, D",
            len: 1,
            cycles: 4,
            execute: LDad,
            jump: false,
        };
        cpu.opcodes[0x7B] = Opcode {
            name: "LD A, E",
            len: 1,
            cycles: 4,
            execute: LDae,
            jump: false,
        };
        cpu.opcodes[0x79] = Opcode {
            name: "LD A, C",
            len: 1,
            cycles: 4,
            execute: LDac,
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
        cpu.opcodes[0x7E] = Opcode {
            name: "LD A, C",
            len: 1,
            cycles: 4,
            execute: LDac,
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
            execute: ADCac,
            jump: false,
        };
        cpu.opcodes[0xA1] = Opcode {
            name: "AND C",
            len: 1,
            cycles: 4,
            execute: ANDc,
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
            execute: ANDa,
            jump: false,
        };
        cpu.opcodes[0xA9] = Opcode {
            name: "XOR C",
            len: 1,
            cycles: 4,
            execute: XORc,
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
            execute: XORa,
            jump: false,
        };
        cpu.opcodes[0xB0] = Opcode {
            name: "OR B",
            len: 1,
            cycles: 4,
            execute: ORb,
            jump: false,
        };
        cpu.opcodes[0xB1] = Opcode {
            name: "OR C",
            len: 1,
            cycles: 4,
            execute: ORc,
            jump: false,
        };
        cpu.opcodes[0xB2] = Opcode {
            name: "OR D",
            len: 1,
            cycles: 4,
            execute: ORc,
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
            execute: ORa,
            jump: false,
        };
        cpu.opcodes[0xB9] = Opcode {
            name: "CPC",
            len: 1,
            cycles: 4,
            execute: CPc,
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
            execute: CPa,
            jump: false,
        };
        cpu.opcodes[0xCA] = Opcode {
            name: "JP Z a16",
            len: 3,
            cycles: 16,
            execute: JPZa16,
            jump: false,
        };
        cpu.opcodes[0xD9] = Opcode {
            name: "RETI",
            len: 1,
            cycles: 20,
            execute: RETI,
            jump: true,
        };
        cpu.opcodes[0xD5] = Opcode {
            name: "PUSH DE",
            len: 1,
            cycles: 16,
            execute: PUSHde,
            jump: false,
        };
        cpu.opcodes[0xE0] = Opcode {
            name: "LDH (a8),A",
            len: 2,
            cycles: 12,
            execute: LDha8a,
            jump: false,
        };
        cpu.opcodes[0xE5] = Opcode {
            name: "PUSH HL",
            len: 1,
            cycles: 16,
            execute: PUSHhl,
            jump: false,
        };
        cpu.opcodes[0xEA] = Opcode {
            name: "LD (a16),A",
            len: 3,
            cycles: 16,
            execute: LDa16a,
            jump: false,
        };
        cpu.opcodes[0xE9] = Opcode {
            name: "JP (HL)",
            len: 1,
            cycles: 4,
            execute: JPhl,
            jump: true,
        };
        cpu.opcodes[0xF3] = Opcode {
            name: "DI",
            len: 1,
            cycles: 4,
            execute: DI,
            jump: false,
        };
        cpu.opcodes[0xC0] = Opcode {
            name: "RET NZ",
            len: 1,
            cycles: 20,
            execute: RETNZ,
            jump: true,
        };
        cpu.opcodes[0xC1] = Opcode {
            name: "POP BC",
            len: 1,
            cycles: 12,
            execute: POPbc,
            jump: false,
        };
        cpu.opcodes[0xC3] = Opcode {
            name: "JP a16",
            len: 3,
            cycles: 16,
            execute: JPa16,
            jump: true,
        };
        cpu.opcodes[0xC4] = Opcode {
            name: "CALL NZ a16",
            len: 3,
            cycles: 24,
            execute: CALLNZa16,
            jump: true,
        };
        cpu.opcodes[0xC5] = Opcode {
            name: "PUSH BC",
            len: 1,
            cycles: 16,
            execute: PUSHbc,
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
            execute: RETZ,
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
        cpu.opcodes[0xD6] = Opcode {
            name: "SUB A,d8",
            len: 2,
            cycles: 8,
            execute: SUBad8,
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
        cpu.opcodes[0xE6] = Opcode {
            name: "AND d8",
            len: 2,
            cycles: 8,
            execute: ANDd8,
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
            execute: LDhaa8,
            jump: false,
        };
        cpu.opcodes[0xF1] = Opcode {
            name: "POP AF",
            len: 1,
            cycles: 12,
            execute: POPaf,
            jump: false,
        };
        cpu.opcodes[0xF5] = Opcode {
            name: "PUSH AF",
            len: 1,
            cycles: 16,
            execute: PUSHaf,
            jump: false,
        };
        cpu.opcodes[0xF6] = Opcode {
            name: "OR d8",
            len: 2,
            cycles: 8,
            execute: ORd8,
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
        cpu.alt_opcodes[0x18] = Opcode {
            name: "RR B",
            len: 2,
            cycles: 8,
            execute: RRb,
            jump: false,
        };
        cpu.alt_opcodes[0x19] = Opcode {
            name: "RR C",
            len: 2,
            cycles: 8,
            execute: RRc,
            jump: false,
        };
        cpu.alt_opcodes[0x1A] = Opcode {
            name: "RR D",
            len: 2,
            cycles: 8,
            execute: RRd,
            jump: false,
        };
        cpu.alt_opcodes[0x1B] = Opcode {
            name: "RR E",
            len: 2,
            cycles: 8,
            execute: RRe,
            jump: false,
        };
        cpu.alt_opcodes[0x1C] = Opcode {
            name: "RR H",
            len: 2,
            cycles: 8,
            execute: RRh,
            jump: false,
        };
        cpu.alt_opcodes[0x1D] = Opcode {
            name: "RR L",
            len: 2,
            cycles: 8,
            execute: RRl,
            jump: false,
        };
        cpu.alt_opcodes[0x41] = Opcode {
            name: "BIT 0,C",
            len: 2,
            cycles: 8,
            execute: BIT0c,
            jump: false,
        };
        cpu.alt_opcodes[0x6F] = Opcode {
            name: "BIT 5,A",
            len: 2,
            cycles: 8,
            execute: BIT5a,
            jump: false,
        };
        cpu.alt_opcodes[0x77] = Opcode {
            name: "BIT 6,A",
            len: 2,
            cycles: 8,
            execute: BIT6a,
            jump: false,
        };
        cpu.alt_opcodes[0x7C] = Opcode {
            name: "BIT 7,H",
            len: 2,
            cycles: 8,
            execute: BIT7h,
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
        cpu.alt_opcodes[0x87] = Opcode {
            name: "RES 0, A",
            len: 2,
            cycles: 8,
            execute: RES0A,
            jump: false,
        };
        cpu.alt_opcodes[0x8B] = Opcode {
            name: "RES 1, E",
            len: 2,
            cycles: 8,
            execute: RES1E,
            jump: false,
        };
        cpu.alt_opcodes[0xFE] = Opcode {
            name: "SET 7, (HL)",
            len: 2,
            cycles: 8,
            execute: SET7hl,
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

    pub fn print_status(&mut self) {
        let code = self.mem.read8(self.regs.PC) as usize;
        let alt_code = self.mem.read8(self.regs.PC+1) as usize;

        println!("----------------------------------------");
        if code == 0xCB {
            println!("PC {:04X} opcode {:02X} ALT {:02X} ", self.regs.PC, code, alt_code);
        } else {
            println!("PC {:04X} opcode {:02X}", self.regs.PC, code);
        }
        println!("==== CPU ====");
        println!("A : {:02X}\tB : {:02X}\tC : {:02X}\tD : {:02X}", self.regs.A, self.regs.B, self.regs.C, self.regs.D);
        println!("E : {:02X}\tF : {:02X}\tH : {:02X}\tL : {:02X}", self.regs.E, self.regs.F, self.regs.H, self.regs.L);
        println!("PC: {:04X} SP: {:04X}", self.regs.get_PC(), self.regs.get_SP());
        println!("==== END ====");
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
        println!("JYJY PC {:04X}", self.regs.PC);
        if self.regs.PC == 0x00FE {
            process::exit(3);

        }
        let code = self.mem.read8(self.regs.PC) as usize;

        let opcode;
        if code == 0xCB {
            let code = self.mem.read8(self.regs.PC+1) as usize;
            debug!("Alternate opcode {:02X}", code);
            opcode = self.alt_opcodes[code];
        } else {
            opcode = self.opcodes[code];
        }
        (opcode.execute)(self);
        self.print_status();

        self.total_cyles = self.total_cyles + opcode.cycles as u64;
        if !opcode.jump {
            self.regs.PC = self.regs.PC.wrapping_add(opcode.len);
        }

        if self.mem.read8(0xFF02) == 0x81 {
            let c = self.mem.read8(0xFF01);
            println!("SERIAL got {}", c as char);
            self.mem.write8(0xff02, 0x0);
        }


        opcode.cycles as u8
    }


}
