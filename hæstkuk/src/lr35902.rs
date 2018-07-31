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
    fn set_FZ(&mut self) {
        self.F |= 0b1000_0000;
    }
    fn unset_FZ(&mut self) {
        self.F &= 0b0111_1111;
    }
    fn get_FZ(&mut self) -> bool{
        (((self.F&(0b1000_0000))>>7)==1) as bool
    }
    fn set_FN(&mut self) {
        self.F |= 0b0100_0000
    }
    fn unset_FN(&mut self) {
        self.F &= 0b1011_1111
    }
    fn get_FN(&mut self) -> bool{
        (((self.F&(0b0100_0000))>>6)==1) as bool
    }
    fn set_FH(&mut self) {
        self.F |= 0b0010_0000
    }
    fn unset_FH(&mut self) {
        self.F &= 0b1101_1111
    }
    fn get_FH(&mut self) -> bool{
        (((self.F&(0b0010_0000))>>5)==1) as bool
    }
    fn set_FC(&mut self) {
        self.F |= 0b0001_0000
    }
    fn unset_FC(&mut self) {
        self.F &= 0b1110_1111
    }
    fn get_FC(&mut self) -> bool{
        (((self.F&(0b0001_0000))>>4)==1) as bool
    }

}

pub struct Cpu<'a> {
    mem: mem::Mem<'a>,
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
    process::exit(3);
}
pub fn ALTUNK(cpu: &mut Cpu) {
    println!("*** Unknow alternative instruction [{:02X}] at {:04X}", cpu.mem.read8(cpu.regs.get_PC()+1), cpu.regs.get_PC());
    cpu.print_status();
    process::exit(3);
}
pub fn NOP(_cpu: &mut Cpu) {
    println!("NOP")
}
pub fn XORd8(cpu: &mut Cpu) {
    let imm = imm8(cpu);
    cpu.regs.A = cpu.regs.A^imm;
    if cpu.regs.A == 0 {
        cpu.regs.set_FZ();
    } else {
        cpu.regs.unset_FZ();
    }
    cpu.regs.unset_FN();
    cpu.regs.unset_FH();
    cpu.regs.unset_FC();
    println!("XOR {:02X}", imm);
}
pub fn XORc(cpu: &mut Cpu) {
    cpu.regs.A = cpu.regs.A^cpu.regs.C;
    if cpu.regs.A == 0 {
        cpu.regs.set_FZ();
    } else {
        cpu.regs.unset_FZ();
    }
    cpu.regs.unset_FN();
    cpu.regs.unset_FH();
    cpu.regs.unset_FC();
    println!("XOR C");
}
pub fn XORa(cpu: &mut Cpu) {
    cpu.regs.A = cpu.regs.A^cpu.regs.A;
    if cpu.regs.A == 0 {
        cpu.regs.set_FZ();
    } else {
        cpu.regs.unset_FZ();
    }
    cpu.regs.unset_FN();
    cpu.regs.unset_FH();
    cpu.regs.unset_FC();
    println!("XOR A");
}
pub fn XOR_hl(cpu: &mut Cpu) {
    let hl = cpu.mem.read8(cpu.regs.get_HL());
    cpu.regs.A = cpu.regs.A^hl;
    if cpu.regs.A == 0 {
        cpu.regs.set_FZ();
    } else {
        cpu.regs.unset_FZ();
    }
    cpu.regs.unset_FN();
    cpu.regs.unset_FH();
    cpu.regs.unset_FC();
    println!("XOR A, [HL]");
}
pub fn ORc(cpu: &mut Cpu) {
    cpu.regs.A = cpu.regs.C|cpu.regs.A;
    if cpu.regs.A == 0 {
        cpu.regs.set_FZ();
    } else {
        cpu.regs.unset_FZ();
    }
    cpu.regs.unset_FN();
    cpu.regs.unset_FH();
    cpu.regs.unset_FC();
    println!("OR C");
}
pub fn ORb(cpu: &mut Cpu) {
    cpu.regs.A = cpu.regs.B|cpu.regs.A;
    if cpu.regs.A == 0 {
        cpu.regs.set_FZ();
    } else {
        cpu.regs.unset_FZ();
    }
    cpu.regs.unset_FN();
    cpu.regs.unset_FH();
    cpu.regs.unset_FC();
    println!("OR B");
}
pub fn ORa(cpu: &mut Cpu) {
    let v = cpu.regs.A;
    cpu.regs.A = cpu.regs.A|v;
    if cpu.regs.A == 0 {
        cpu.regs.set_FZ();
    } else {
        cpu.regs.unset_FZ();
    }
    cpu.regs.unset_FN();
    cpu.regs.unset_FH();
    cpu.regs.unset_FC();
    println!("OR A");
}
pub fn ORhl(cpu: &mut Cpu) {
    let v = cpu.mem.read8(cpu.regs.get_HL());
    cpu.regs.A = cpu.regs.A|v;
    if cpu.regs.A == 0 {
        cpu.regs.set_FZ();
    } else {
        cpu.regs.unset_FZ();
    }
    cpu.regs.unset_FN();
    cpu.regs.unset_FH();
    cpu.regs.unset_FC();
    println!("OR (hl)");
}
pub fn ORd8(cpu: &mut Cpu) {
    let v = imm8(cpu);
    cpu.regs.A = cpu.regs.A|v;
    if cpu.regs.A == 0 {
        cpu.regs.set_FZ();
    } else {
        cpu.regs.unset_FZ();
    }
    cpu.regs.unset_FN();
    cpu.regs.unset_FH();
    cpu.regs.unset_FC();
    println!("OR imm8");
}
pub fn ANDc(cpu: &mut Cpu) {
    cpu.regs.A = cpu.regs.A&cpu.regs.C;
    if cpu.regs.A == 0 {
        cpu.regs.set_FZ();
    } else {
        cpu.regs.unset_FZ();
    }
    cpu.regs.unset_FN();
    cpu.regs.set_FH();
    cpu.regs.unset_FC();
    println!("AND C");
}
pub fn ANDa(cpu: &mut Cpu) {
    cpu.regs.A = cpu.regs.A&cpu.regs.A;
    if cpu.regs.A == 0 {
        cpu.regs.set_FZ();
    } else {
        cpu.regs.unset_FZ();
    }
    cpu.regs.unset_FN();
    cpu.regs.set_FH();
    cpu.regs.unset_FC();
    println!("AND A");
}
pub fn ANDd8(cpu: &mut Cpu) {
    let imm = imm8(cpu);
    cpu.regs.A = cpu.regs.A & imm;

    if cpu.regs.A == 0 {
        cpu.regs.set_FZ();
    } else {
        cpu.regs.unset_FZ();
    }
    cpu.regs.unset_FN();
    cpu.regs.set_FH();
    cpu.regs.unset_FC();
    println!("AND {:02}", imm);
}
pub fn SUBad8(cpu: &mut Cpu) {
    let imm = imm8(cpu);
    if imm>cpu.regs.A {
        cpu.regs.set_FC();
    } else {
        cpu.regs.unset_FC();
    }

    cpu.regs.A = cpu.regs.A.wrapping_sub(imm);
    if cpu.regs.A == 0 {
        cpu.regs.set_FZ();
    } else {
        cpu.regs.unset_FZ();
    }
    cpu.regs.unset_FN();
    //TODO
    //      H - Set if carry from bit 3.
    println!("SUB A, {:02X}", imm);
}
pub fn ADCad8(cpu: &mut Cpu) {
    let mut c = 0;
    if cpu.regs.get_FC() == true {
        c=1;
    }
    let imm = imm8(cpu)+c;
    if (imm as u16)+(cpu.regs.A as u16) > 255 {
        cpu.regs.set_FC();
    } else {
        cpu.regs.unset_FC();
    }
    cpu.regs.A = cpu.regs.A.wrapping_add(imm);
    if cpu.regs.A == 0 {
        cpu.regs.set_FZ();
    } else {
        cpu.regs.unset_FZ();
    }
    cpu.regs.unset_FN();
    //TODO
    //      H - Set if carry from bit 3.
    println!("ADC A, {:02X}", imm);
}
pub fn ADDad8(cpu: &mut Cpu) {
    let imm = imm8(cpu);
    if (imm as u16)+(cpu.regs.A as u16) > 255 {
        cpu.regs.set_FC();
    } else {
        cpu.regs.unset_FC();
    }
    cpu.regs.A = cpu.regs.A.wrapping_add(imm);
    if cpu.regs.A == 0 {
        cpu.regs.set_FZ();
    } else {
        cpu.regs.unset_FZ();
    }
    cpu.regs.unset_FN();
    //TODO
    //      H - Set if carry from bit 3.
    println!("ADD A, {:02X}", imm);
}
pub fn ADDaa(cpu: &mut Cpu) {
    if (cpu.regs.A as u16)+(cpu.regs.A as u16) > 255 {
        cpu.regs.set_FC();
    } else {
        cpu.regs.unset_FC();
    }
    cpu.regs.A = cpu.regs.A.wrapping_add(cpu.regs.A);
    if cpu.regs.A == 0 {
        cpu.regs.set_FZ();
    } else {
        cpu.regs.unset_FZ();
    }
    cpu.regs.unset_FN();
    //TODO
    //      H - Half-Carry.
    println!("ADD A,A");
}
pub fn ADDac(cpu: &mut Cpu) {
    if (cpu.regs.A as u16)+(cpu.regs.C as u16) > 255 {
        cpu.regs.set_FC();
    } else {
        cpu.regs.unset_FC();
    }
    cpu.regs.A = cpu.regs.A.wrapping_add(cpu.regs.C);
    if cpu.regs.A == 0 {
        cpu.regs.set_FZ();
    } else {
        cpu.regs.unset_FZ();
    }
    cpu.regs.unset_FN();
    //TODO
    //      H - Set if carry from bit 3.
    println!("ADD A,C");
}
pub fn ADDhlde(cpu: &mut Cpu) {
    let hl = cpu.regs.get_HL();
    let de = cpu.regs.get_DE();

    cpu.regs.set_HL(hl.wrapping_add(de));

    cpu.regs.unset_FN();
    //TODO
    //      H - Set if carry from bit 11.
    //      C - Set if carry from bit 15.
    println!("ADD HL,DE");
}
pub fn DECc(cpu: &mut Cpu) {
    cpu.regs.C = cpu.regs.C.wrapping_sub(1);
    if cpu.regs.C == 0 {
        cpu.regs.set_FZ();
    } else {
        cpu.regs.unset_FZ();
    }
    cpu.regs.set_FN();
    // Z 0 H -
    //Z N H C
    println!("DEC C, F is {:b}", cpu.regs.F);
}
pub fn DECb(cpu: &mut Cpu) {
    cpu.regs.B = cpu.regs.B.wrapping_sub(1);
    if cpu.regs.B == 0 {
        cpu.regs.set_FZ();
    } else {
        cpu.regs.unset_FZ();
    }
    cpu.regs.set_FN();
    // Z 0 H -
    //Z N H C
    println!("DEC B");
}
pub fn DECh(cpu: &mut Cpu) {
    cpu.regs.H = cpu.regs.H.wrapping_sub(1);
    if cpu.regs.H == 0 {
        cpu.regs.set_FZ();
    } else {
        cpu.regs.unset_FZ();
    }
    cpu.regs.unset_FN();
    // Z 0 H -
    //Z N H C
    println!("DEC H");
}
pub fn DECa(cpu: &mut Cpu) {
    cpu.regs.A = cpu.regs.A.wrapping_sub(1);
    if cpu.regs.A == 0 {
        cpu.regs.set_FZ();
    } else {
        cpu.regs.unset_FZ();
    }
    cpu.regs.unset_FN();
    // Z 0 H -
    //Z N H C
    println!("DEC A");
}
pub fn DECe(cpu: &mut Cpu) {
    cpu.regs.E = cpu.regs.E.wrapping_sub(1);
    if cpu.regs.E == 0 {
        cpu.regs.set_FZ();
    } else {
        cpu.regs.unset_FZ();
    }
    cpu.regs.unset_FN();
    // Z 0 H -
    //Z N H C
    println!("DEC E");
}
pub fn DECl(cpu: &mut Cpu) {
    cpu.regs.L = cpu.regs.L.wrapping_sub(1);
    if cpu.regs.L == 0 {
        cpu.regs.set_FZ();
    } else {
        cpu.regs.unset_FZ();
    }
    cpu.regs.unset_FN();
    // Z 0 H -
    //Z N H C
    println!("DEC D");
}
pub fn DECd(cpu: &mut Cpu) {
    cpu.regs.D = cpu.regs.D.wrapping_sub(1);
    if cpu.regs.D == 0 {
        cpu.regs.set_FZ();
    } else {
        cpu.regs.unset_FZ();
    }
    cpu.regs.unset_FN();
    // Z 0 H -
    //Z N H C
    println!("DEC D");
}
pub fn DECbc(cpu: &mut Cpu) {
    let bc = cpu.regs.get_BC();
    cpu.regs.set_BC(bc.wrapping_sub(1));
    println!("DEC BC");
}
pub fn INChl(cpu: &mut Cpu) {
    let hl = cpu.regs.get_HL();
    cpu.regs.set_HL(hl.wrapping_add(1));
    println!("INC HL");
}
pub fn DEC_hl(cpu: &mut Cpu) {
    let hl = cpu.mem.read8(cpu.regs.get_HL());
    cpu.mem.write8(cpu.regs.get_HL(), hl.wrapping_sub(1));
    println!("DEC (HL)");
}
pub fn INCde(cpu: &mut Cpu) {
    let de = cpu.regs.get_DE();
    cpu.regs.set_DE(de.wrapping_add(1));
    println!("INC DE");
}
pub fn INCbc(cpu: &mut Cpu) {
    let bc = cpu.regs.get_BC();
    cpu.regs.set_BC(bc.wrapping_add(1));
    println!("INC BC");
}
pub fn INCa(cpu: &mut Cpu) {
    cpu.regs.A = cpu.regs.A.wrapping_add(1);
    if cpu.regs.A == 0 {
        cpu.regs.set_FZ();
    } else {
        cpu.regs.unset_FZ();
    }
    cpu.regs.unset_FN();
    // Z 0 H -
    //Z N H C
    println!("INC A");
}
pub fn INCh(cpu: &mut Cpu) {
    cpu.regs.H = cpu.regs.H.wrapping_add(1);
    if cpu.regs.H == 0 {
        cpu.regs.set_FZ();
    } else {
        cpu.regs.unset_FZ();
    }
    cpu.regs.unset_FN();
    println!("INC H");
}
pub fn INCl(cpu: &mut Cpu) {
    cpu.regs.L = cpu.regs.L.wrapping_add(1);
    if cpu.regs.L == 0 {
        cpu.regs.set_FZ();
    } else {
        cpu.regs.unset_FZ();
    }
    cpu.regs.unset_FN();
    println!("INC L");
}
pub fn INCc(cpu: &mut Cpu) {
    cpu.regs.C = cpu.regs.C.wrapping_add(1);
    if cpu.regs.C == 0 {
        cpu.regs.set_FZ();
    } else {
        cpu.regs.unset_FZ();
    }
    cpu.regs.unset_FN();
    println!("INC C");
}
pub fn INCd(cpu: &mut Cpu) {
    cpu.regs.D = cpu.regs.D.wrapping_add(1);
    if cpu.regs.D == 0 {
        cpu.regs.set_FZ();
    } else {
        cpu.regs.unset_FZ();
    }
    cpu.regs.unset_FN();
    // Z 0 H -
    //Z N H C
    println!("INC D");
}
pub fn INCe(cpu: &mut Cpu) {
    cpu.regs.E = cpu.regs.E.wrapping_add(1);
    if cpu.regs.E == 0 {
        cpu.regs.set_FZ();
    } else {
        cpu.regs.unset_FZ();
    }
    cpu.regs.unset_FN();
    println!("INC E");
}
pub fn CPc(cpu: &mut Cpu) {
    let c = cpu.regs.C;
    cpu.regs.set_FN();
    cpu.regs.unset_FZ();
    if cpu.regs.A == c {
        cpu.regs.set_FZ();
    }
    if cpu.regs.A < c {
        cpu.regs.set_FC();
    }
    println!("CPD")
}
pub fn CPL(cpu: &mut Cpu) {
    let A = cpu.regs.A;
    cpu.regs.A = !A;

    cpu.regs.set_FN();
    cpu.regs.set_FH();

    println!("CPL")
}
pub fn CPd8(cpu: &mut Cpu) {
    let imm = imm8(cpu);
    cpu.regs.set_FN();
    cpu.regs.unset_FZ();
    if cpu.regs.A == imm {
        cpu.regs.set_FZ();
    }
    if cpu.regs.A < imm {
        cpu.regs.set_FC();
    }
    println!("CP {:02X}", imm)
}

pub fn RRb(cpu: &mut Cpu) {
    let c = cpu.regs.B&0b00000001;
    cpu.regs.B = cpu.regs.B>>1;
    if cpu.regs.get_FC() == true {
        cpu.regs.B |= 1<<7;
    }
    if c == 1 {
        cpu.regs.set_FC();
    } else {
        cpu.regs.unset_FC();
    }
    if cpu.regs.B == 0 {
        cpu.regs.set_FZ();
    } else {
        cpu.regs.unset_FZ();
    }
    cpu.regs.unset_FN();
    cpu.regs.unset_FH();
    println!("RR B");
}
pub fn RRc(cpu: &mut Cpu) {
    let c = cpu.regs.A&0b00000001;
    cpu.regs.C = cpu.regs.C>>1;
    if cpu.regs.get_FC() == true {
        cpu.regs.C |= 1<<7;
    }
    if c == 1 {
        cpu.regs.set_FC();
    } else {
        cpu.regs.unset_FC();
    }
    if cpu.regs.C == 0 {
        cpu.regs.set_FZ();
    } else {
        cpu.regs.unset_FZ();
    }
    cpu.regs.unset_FN();
    cpu.regs.unset_FH();
    println!("RR C");

}
pub fn RRd(cpu: &mut Cpu) {
    let c = cpu.regs.D&0b00000001;
    cpu.regs.D = cpu.regs.D>>1;
    if cpu.regs.get_FC() == true {
        cpu.regs.D |= 1<<7;
    }
    if c == 1 {
        cpu.regs.set_FC();
    } else {
        cpu.regs.unset_FC();
    }
    if cpu.regs.D == 0 {
        cpu.regs.set_FZ();
    } else {
        cpu.regs.unset_FZ();
    }
    cpu.regs.unset_FN();
    cpu.regs.unset_FH();
    println!("RR D");

}
pub fn RRe(cpu: &mut Cpu) {
    let c = cpu.regs.E&0b00000001;
    cpu.regs.E = cpu.regs.E>>1;
    if cpu.regs.get_FC() == true {
        cpu.regs.E |= 1<<7;
    }
    if c == 1 {
        cpu.regs.set_FC();
    } else {
        cpu.regs.unset_FC();
    }
    if cpu.regs.E == 0 {
        cpu.regs.set_FZ();
    } else {
        cpu.regs.unset_FZ();
    }
    cpu.regs.unset_FN();
    cpu.regs.unset_FH();
    println!("RR E");

}
pub fn RRh(cpu: &mut Cpu) {
    let c = cpu.regs.H&0b00000001;
    cpu.regs.H = cpu.regs.H>>1;
    if cpu.regs.get_FC() == true {
        cpu.regs.H |= 1<<7;
    }
    if c == 1 {
        cpu.regs.set_FC();
    } else {
        cpu.regs.unset_FC();
    }
    if cpu.regs.H == 0 {
        cpu.regs.set_FZ();
    } else {
        cpu.regs.unset_FZ();
    }
    cpu.regs.unset_FN();
    cpu.regs.unset_FH();
    println!("RR H");

}
pub fn RRl(cpu: &mut Cpu) {

    let c = cpu.regs.L&0b00000001;
    cpu.regs.L = cpu.regs.L>>1;
    if cpu.regs.get_FC() == true {
        cpu.regs.L |= 1<<7;
    }
    if c == 1 {
        cpu.regs.set_FC();
    } else {
        cpu.regs.unset_FC();
    }
    if cpu.regs.L == 0 {
        cpu.regs.set_FZ();
    } else {
        cpu.regs.unset_FZ();
    }
    cpu.regs.unset_FN();
    cpu.regs.unset_FH();
    println!("RR L");

}
pub fn RRa(cpu: &mut Cpu) {

    let c = cpu.regs.A&0b00000001;
    cpu.regs.A = cpu.regs.A>>1;
    if cpu.regs.get_FC() == true {
        cpu.regs.A |= 1<<7;
    }
    if c == 1 {
        cpu.regs.set_FC();
    } else {
        cpu.regs.unset_FC();
    }
    if cpu.regs.A == 0 {
        cpu.regs.set_FZ();
    } else {
        cpu.regs.unset_FZ();
    }
    cpu.regs.unset_FN();
    cpu.regs.unset_FH();
    println!("RR A");

}
pub fn RRCa(cpu: &mut Cpu) {

    if ((cpu.regs.A&0b10000000)>>7) == 1 {
        cpu.regs.set_FC();
    } else {
        cpu.regs.unset_FC();
    }
    cpu.regs.A = cpu.regs.A>>1;

    if cpu.regs.A == 0 {
        cpu.regs.set_FZ();
    } else {
        cpu.regs.unset_FZ();
    }
    cpu.regs.unset_FN();
    cpu.regs.unset_FH();
    println!("OR C");

}
pub fn LDade(cpu: &mut Cpu) {
    let addr = cpu.regs.get_DE();
    cpu.regs.A = cpu.mem.read8(addr);
    println!("LD A, (DE) ({:04X})", addr);
}
pub fn LDlhl(cpu: &mut Cpu) {
    let addr = cpu.regs.get_HL();
    cpu.regs.L = cpu.mem.read8(addr);
    println!("LD L, (HL) ({:04X})", addr);
}
pub fn LDbhl(cpu: &mut Cpu) {
    let addr = cpu.regs.get_HL();
    cpu.regs.B = cpu.mem.read8(addr);
    println!("LD B, (HL) ({:04X})", addr);
}
pub fn LDchl(cpu: &mut Cpu) {
    let addr = cpu.regs.get_HL();
    cpu.regs.C = cpu.mem.read8(addr);
    println!("LD C, (HL) ({:04X})", addr);
}
pub fn LDaa16(cpu: &mut Cpu) {
    let addr = addr16(cpu);
    cpu.regs.A = cpu.mem.read8(addr);
    println!("LD A, (a16) ({:04X})", addr);

}
pub fn LDhld16(cpu: &mut Cpu) {
    let imm = imm16(cpu);
    cpu.regs.set_HL(imm);
    println!("LD HL, {:04X}", imm)
}
pub fn LDhd8(cpu: &mut Cpu) {
    let imm = imm8(cpu);
    cpu.regs.H = imm;
    println!("LD H, {:04X}", imm)
}
pub fn LDhla(cpu: &mut Cpu) {
    let hl = cpu.regs.get_HL();
    cpu.mem.write8(hl, cpu.regs.A);
    println!("LD {:04X}, A", hl);
}
pub fn LDhlpa(cpu: &mut Cpu) {
    let hl = cpu.regs.get_HL();
    cpu.mem.write8(hl, cpu.regs.A);
    cpu.regs.set_HL(hl.wrapping_add(1));
    println!("LD {:04X}+, A", hl);
}
pub fn LDhld8(cpu: &mut Cpu) {
    let imm = imm8(cpu);
    cpu.mem.write8(cpu.regs.get_HL(), imm);
    println!("LD (HL), {:02X}", imm)
}
pub fn LDIahlp(cpu: &mut Cpu) {
    let hl = cpu.regs.get_HL();
    cpu.regs.A = cpu.mem.read8(hl);
    cpu.regs.set_HL(hl.wrapping_add(1));
    println!("LD A, (HL+)  {:02X}<-({:04X})", cpu.regs.A, hl);
}
pub fn LDhhl(cpu: &mut Cpu) {
    let hl = cpu.regs.get_HL();
    cpu.regs.H = cpu.mem.read8(hl);
    println!("LD H, (HL)")
}
pub fn LDhlb(cpu: &mut Cpu) {
    let B = cpu.regs.B;
    cpu.regs.set_HL(B as u16);
    println!("LD (HL), B")
}
pub fn LDdea(cpu: &mut Cpu) {
    cpu.mem.write8(cpu.regs.get_DE(), cpu.regs.A);
    println!("LD (DE), A")
}
pub fn LDda(cpu: &mut Cpu) {
    cpu.regs.D = cpu.regs.A;
    println!("LD D, A")
}
pub fn LDae(cpu: &mut Cpu) {
    cpu.regs.A = cpu.regs.B;
    println!("LD A, E")
}
pub fn LDad(cpu: &mut Cpu) {
    cpu.regs.A = cpu.regs.B;
    println!("LD A, D")
}
pub fn LDab(cpu: &mut Cpu) {
    cpu.regs.A = cpu.regs.B;
    println!("LD A, B")
}
pub fn LDac(cpu: &mut Cpu) {
    cpu.regs.A = cpu.regs.C;
    println!("LD A, C")
}
pub fn LDhb(cpu: &mut Cpu) {
    cpu.regs.H = cpu.regs.B;
    println!("LD H, B")
}
pub fn LDlh(cpu: &mut Cpu) {
    cpu.regs.L = cpu.regs.H;
    println!("LD L, H")
}
pub fn LDca(cpu: &mut Cpu) {
    cpu.regs.C = cpu.regs.A;
    println!("LD C, A")
}
pub fn LDpca(cpu: &mut Cpu) {
    let C = cpu.regs.C as u16;
    cpu.mem.write8(0xFF00 + C, cpu.regs.A);
    println!("LD (C), A")
}
pub fn LDded16(cpu: &mut Cpu) {
    let imm = imm16(cpu);
    cpu.regs.set_DE(imm);
    println!("LD DE, {:04X}", imm)
}
pub fn LDbcd16(cpu: &mut Cpu) {
    let imm = imm16(cpu);
    cpu.regs.set_BC(imm);
    println!("LD BC, {:04X}", imm)
}
pub fn LDspd16(cpu: &mut Cpu) {
    let imm = imm16(cpu);
    cpu.regs.set_SP(imm);
    println!("LD SP, {:04X}", imm)
}
pub fn LDDhmla(cpu: &mut Cpu) {
    let hl = cpu.regs.get_HL();
    cpu.mem.write8(hl, cpu.regs.A);
    cpu.regs.set_HL(hl.wrapping_sub(1));
    println!("LD- [{:04X}], a", hl);
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
pub fn LDdd8(cpu: &mut Cpu) {
    let imm = imm8(cpu);
    cpu.regs.D = imm;
    println!("LD D, {:02X}", imm)
}
pub fn LDbd8(cpu: &mut Cpu) {
    let imm = imm8(cpu);
    cpu.regs.B = imm;
    println!("LD B, {:02X}", imm)
}
pub fn LDha8a(cpu: &mut Cpu) {
    let imm = imm8(cpu);
    cpu.mem.write8(0xFF00+imm as u16, cpu.regs.A);
    println!("LDH (FF{:02X}), A", imm)
}
pub fn LDhaa8(cpu: &mut Cpu) {
    let imm = imm8(cpu);
    cpu.regs.A = cpu.mem.read8(0xFF00+imm as u16);
    println!("LDH A, ({:02X})", imm)
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
pub fn LDea(cpu: &mut Cpu) {
    cpu.regs.E = cpu.regs.A;
    println!("LD E, A")
}
pub fn LDah(cpu: &mut Cpu) {
    cpu.regs.A = cpu.regs.H;
    println!("LDH A, H")
}
pub fn LDehl(cpu: &mut Cpu) {
    let m = cpu.mem.read8(cpu.regs.get_HL());
    cpu.regs.E = m;
    println!("LD E, {:04X}", m);
}
pub fn LDdhl(cpu: &mut Cpu) {
    let m = cpu.mem.read8(cpu.regs.get_HL());
    cpu.regs.D = m;
    println!("LD D, {:04X}", m);
}
pub fn JPa16(cpu: &mut Cpu) {
    let addr = addr16(cpu);
    cpu.regs.PC = addr;
    println!("JP {:04X}", addr)
}
pub fn JPZa16(cpu: &mut Cpu) {
    let addr = addr16(cpu);
    if cpu.regs.get_FZ() == true {
        cpu.regs.PC = addr;
    }
    println!("JP {:04X}", addr)
}
pub fn JRr8(cpu: &mut Cpu) {
    let offset = cpu.regs.PC + 2;
    let v      = imm8(cpu) as i16;
    cpu.regs.PC = if v < 0 { offset - (-v) as u16 } else { offset + v as u16 };
    println!("JR {:04X} (PC (after +{:}))", cpu.regs.PC, v)
}
pub fn JPhl(cpu: &mut Cpu) {
    let addr = cpu.regs.get_HL();
    cpu.regs.PC = addr;
    println!("JP ({:04X})", addr)
}
pub fn POPhl(cpu: &mut Cpu) {
    let sp = PopStack(cpu);
    cpu.regs.set_HL(sp);
    println!("POP HL");
}
pub fn POPde(cpu: &mut Cpu) {
    let de = PopStack(cpu);
    cpu.regs.set_DE(de);
    println!("POP DE");
}
pub fn POPbc(cpu: &mut Cpu) {
    let sp = PopStack(cpu);
    cpu.regs.set_BC(sp);
    println!("POP BC");
}
pub fn POPaf(cpu: &mut Cpu) {
    let sp = PopStack(cpu);
    cpu.regs.set_AF(sp);
    println!("POP AF");
}
pub fn PUSHde(cpu: &mut Cpu) {
    let v = cpu.regs.get_DE();
    PushStack(cpu, v);
    println!("PUSH DE");
}
pub fn PUSHbc(cpu: &mut Cpu) {
    let v = cpu.regs.get_BC();
    PushStack(cpu, v);
    println!("PUSH BC");
}
pub fn PUSHaf(cpu: &mut Cpu) {
    let v = cpu.regs.get_AF();
    PushStack(cpu, v);
    println!("PUSH AF");
}
pub fn PUSHhl(cpu: &mut Cpu) {
    let v = cpu.regs.get_HL();
    PushStack(cpu, v);
    println!("PUSH HL");
}

pub fn RST28h(cpu: &mut Cpu) {
    let PC = cpu.regs.PC;
    PushStack(cpu, PC);
    cpu.regs.PC = 0x28;
    println!("RST 28h")
}
pub fn RST38h(cpu: &mut Cpu) {
    let PC = cpu.regs.PC;
    PushStack(cpu, PC);
    cpu.regs.PC = 0x38;
    println!("RST 38h")
}
pub fn JRncr8(cpu: &mut Cpu) {
    let offset = cpu.regs.PC + 2;
    let v:i8      = imm8(cpu) as i8;
    if cpu.regs.get_FC() == false {
        cpu.regs.PC = if v < 0 { offset - (-v) as u16 } else { offset + v as u16 }
    } else {
        cpu.regs.PC = offset;
    }
    println!("JRNC {:02X}", v)
}
pub fn JRnzr8(cpu: &mut Cpu) {
    let offset = cpu.regs.PC + 2;
    let v:i8      = imm8(cpu) as i8;
    if cpu.regs.get_FZ() == false {
        cpu.regs.PC = if v < 0 { offset - (-v) as u16 } else { offset + v as u16 }
    } else {
        cpu.regs.PC = offset;
    }
    println!("JRNZ {:02X}", v)
}
pub fn JRcr8(cpu: &mut Cpu) {
    let offset = cpu.regs.PC + 2;
    let v:i8      = imm8(cpu) as i8;
    if cpu.regs.get_FC() == true {
        cpu.regs.PC = if v < 0 { offset - (-v) as u16 } else { offset + v as u16 }
    } else {
        cpu.regs.PC = offset;
    }
    println!("JR C {:02X}", v)
}
pub fn JRzr8(cpu: &mut Cpu) {
    let offset = cpu.regs.PC + 2;
    let v:i8      = imm8(cpu) as i8;
    if cpu.regs.get_FZ() == true {
        cpu.regs.PC = if v < 0 { offset - (-v) as u16 } else { offset + v as u16 }
    } else {
        cpu.regs.PC = offset;
    }
    println!("JRZ {:02X}", v)
}
pub fn CALLa16(cpu: &mut Cpu) {
    let addr = addr16(cpu);
    let next = cpu.regs.PC + 3;
    PushStack(cpu, next);
    cpu.regs.PC = addr;
    println!("CALL {:04X}", addr)
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
    println!("CALL {:04X}", addr)
}
pub fn RET(cpu: &mut Cpu) {
    let addr = PopStack(cpu);
    cpu.regs.PC = addr;
    println!("RET (-> {:04X})", addr)
}
pub fn RETI(cpu: &mut Cpu) {
    let addr = PopStack(cpu);
    cpu.regs.PC = addr;
    EI(cpu);
    println!("RETI (-> {:04X})", addr)
}
pub fn RETNC(cpu: &mut Cpu) {
    if cpu.regs.get_FC() == true {
        let addr = PopStack(cpu);
        cpu.regs.PC = addr;
        println!("RET NC (-> {:04X})", addr)
    } else {
        cpu.regs.PC = cpu.regs.PC.wrapping_add(1);
        println!("RET NC (-> continue)")
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
    println!("RET Z (-> {:04X})", addr)
}
pub fn RETNZ(cpu: &mut Cpu) {
    let mut addr = 0;
    if cpu.regs.get_FZ() == true {
        addr = PopStack(cpu);
        cpu.regs.PC = addr;
    } else {
        cpu.regs.PC = cpu.regs.PC.wrapping_add(1);
    }
    println!("RET NZ (-> {:04X})", addr)
}
pub fn DI(cpu: &mut Cpu) {
    cpu.regs.I = false;
    println!("DI")
}
pub fn EI(cpu: &mut Cpu) {
    cpu.regs.I = true;
    println!("EI")
}

pub fn SWAPa(cpu: &mut Cpu) {
    cpu.regs.A = ((cpu.regs.A&0xF0)>>4)|(cpu.regs.A<<4);
    if cpu.regs.A == 0 {
        cpu.regs.set_FZ();
    } else {
        cpu.regs.unset_FZ();
    }
    cpu.regs.unset_FN();
    cpu.regs.unset_FH();
    cpu.regs.unset_FC();

    println!("SWAP A");
}
pub fn SET7hl(cpu: &mut Cpu) {
    let hl = cpu.regs.get_HL();
    let mut v =  cpu.mem.read8(hl);
    v|=0b1000_0000;
    cpu.mem.write8(hl, v);
    println!("SET 7, HL")
}
pub fn RLCa(cpu: &mut Cpu) {

    let c = cpu.regs.A >> 7;
    cpu.regs.A = (cpu.regs.A << 1) | c;

    if cpu.regs.A == 0 {
        cpu.regs.set_FZ();
    } else {
        cpu.regs.unset_FZ();
    }
    cpu.regs.unset_FN();
    cpu.regs.unset_FH();
    if c == 1 {
        cpu.regs.set_FC();
    } else {
        cpu.regs.unset_FC();
    }
}
pub fn SRLa(cpu: &mut Cpu) {

    let c = cpu.regs.A & 1;
    cpu.regs.A = cpu.regs.A >> 1;

    if cpu.regs.A == 0 {
        cpu.regs.set_FZ();
    } else {
        cpu.regs.unset_FZ();
    }
    cpu.regs.unset_FN();
    cpu.regs.unset_FH();
    if c == 1 {
        cpu.regs.set_FC();
    } else {
        cpu.regs.unset_FC();
    }
}
pub fn SRLb(cpu: &mut Cpu) {

    let c = cpu.regs.B & 1;
    cpu.regs.B = cpu.regs.B >> 1;

    if cpu.regs.B == 0 {
        cpu.regs.set_FZ();
    } else {
        cpu.regs.unset_FZ();
    }
    cpu.regs.unset_FN();
    cpu.regs.unset_FH();
    if c == 1 {
        cpu.regs.set_FC();
    } else {
        cpu.regs.unset_FC();
    }
}
pub fn SRLc(cpu: &mut Cpu) {

    let c = cpu.regs.C & 1;
    cpu.regs.C = cpu.regs.C >> 1;

    if cpu.regs.C == 0 {
        cpu.regs.set_FZ();
    } else {
        cpu.regs.unset_FZ();
    }
    cpu.regs.unset_FN();
    cpu.regs.unset_FH();
    if c == 1 {
        cpu.regs.set_FC();
    } else {
        cpu.regs.unset_FC();
    }
}
pub fn SRLd(cpu: &mut Cpu) {

    let c = cpu.regs.D & 1;
    cpu.regs.D = cpu.regs.D >> 1;

    if cpu.regs.D == 0 {
        cpu.regs.set_FZ();
    } else {
        cpu.regs.unset_FZ();
    }
    cpu.regs.unset_FN();
    cpu.regs.unset_FH();
    if c == 1 {
        cpu.regs.set_FC();
    } else {
        cpu.regs.unset_FC();
    }
}
pub fn SRLe(cpu: &mut Cpu) {

    let c = cpu.regs.E & 1;
    cpu.regs.E = cpu.regs.E >> 1;

    if cpu.regs.E == 0 {
        cpu.regs.set_FZ();
    } else {
        cpu.regs.unset_FZ();
    }
    cpu.regs.unset_FN();
    cpu.regs.unset_FH();
    if c == 1 {
        cpu.regs.set_FC();
    } else {
        cpu.regs.unset_FC();
    }
}
pub fn SRLh(cpu: &mut Cpu) {

    let c = cpu.regs.H & 1;
    cpu.regs.H = cpu.regs.H >> 1;

    if cpu.regs.H == 0 {
        cpu.regs.set_FZ();
    } else {
        cpu.regs.unset_FZ();
    }
    cpu.regs.unset_FN();
    cpu.regs.unset_FH();
    if c == 1 {
        cpu.regs.set_FC();
    } else {
        cpu.regs.unset_FC();
    }
}
pub fn SRLl(cpu: &mut Cpu) {

    let c = cpu.regs.L & 1;
    cpu.regs.L = cpu.regs.L >> 1;

    if cpu.regs.L == 0 {
        cpu.regs.set_FZ();
    } else {
        cpu.regs.unset_FZ();
    }
    cpu.regs.unset_FN();
    cpu.regs.unset_FH();
    if c == 1 {
        cpu.regs.set_FC();
    } else {
        cpu.regs.unset_FC();
    }
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
                I: false,
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
            name: "RRC A",
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
        cpu.opcodes[0x70] = Opcode {
            name: "LD (HL),B",
            len: 1,
            cycles: 8,
            execute: LDhlb,
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
        cpu.opcodes[0x81] = Opcode {
            name: "ADD A,C",
            len: 1,
            cycles: 4,
            execute: ADDac,
            jump: false,
        };
        cpu.opcodes[0x87] = Opcode {
            name: "ADD A,A",
            len: 1,
            cycles: 4,
            execute: ADDaa,
            jump: false,
        };
        cpu.opcodes[0xA1] = Opcode {
            name: "AND C",
            len: 1,
            cycles: 4,
            execute: ANDc,
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
    pub fn writeMem8(&mut self, addr: u16, v: u8)  {
        self.mem.write8(addr, v)
    }

    pub fn print_status(&mut self) {
        println!("==== CPU ====");
        println!("PC: {:04X}", self.regs.get_PC());
        println!("SP: {:04X}", self.regs.get_SP());
        println!("A : {:02X}\tF : {:02X}", self.regs.A, self.regs.F);
        println!("B : {:02X}\tC : {:02X}", self.regs.B, self.regs.C);
        println!("D : {:02X}\tE : {:02X}", self.regs.D, self.regs.E);
        println!("H : {:02X}\tL : {:02X}", self.regs.H, self.regs.L);
        println!("RST Vectors : ");
/*        for i in vec![0x00,0x08,0x10,0x18,0x20,0x28,0x30,0x38].iter() {
            println!("0x00{:02X}:  {:02X} {:02X}", i, self.mem.read8(*i as u16), self.mem.read8((i+1) as u16));
        }*/
        println!("==== END ====");
//        self.mem.print_infos();
    }

    pub fn interrupts_enabled(&mut self) -> bool {
        self.regs.I
    }

    pub fn irq_vblank(&mut self) {
        DI(self);
        let addr = addr16(self);
        PushStack(self, addr);
        self.regs.PC = 0x0040;
    }

    pub fn reset(&mut self) {
        self.regs.PC = 0x0100
    }

    pub fn step(&mut self) -> u8 {
        let code = self.mem.read8(self.regs.PC) as usize;

        let opcode;
        if code == 0xCB {
            let code = self.mem.read8(self.regs.PC+1) as usize;
            println!("Alternate opcode {:02X}", code);
            opcode = self.alt_opcodes[code];
        } else {
            opcode = self.opcodes[code];
        }
        println!("----------------------------------------");
        print!("{:04X}: {:02X} -> ", self.regs.PC, code);
        (opcode.execute)(self);
        self.print_status();
        //println!("----------------------------------------");
        self.total_cyles = self.total_cyles + opcode.cycles as u64;
        if !opcode.jump {
            self.regs.PC = self.regs.PC.wrapping_add(opcode.len);
        }
        opcode.cycles as u8
    }


}
