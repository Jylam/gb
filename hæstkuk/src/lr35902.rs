// Sharp LR35902 CPU emulator
#![allow(non_snake_case)]
#![allow(dead_code)]

//use std::process;
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
    fn HLd(&mut self) -> u16 {
        let h = self.get_HL();
        self.set_HL(h-1);
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

pub struct Cpu<'a> {
    pub mem: mem::Mem<'a>,
    regs: Registers,
    total_cyles: u64,
}




pub fn PushStack(cpu: &mut Cpu, v: u16) {
    debug!("Pushing {:04X} into stack at {:04X}", v, cpu.regs.SP);
    cpu.mem.write16(cpu.regs.SP, v);
    cpu.regs.SP.wrapping_sub(2);
}
pub fn PopStack(cpu: &mut Cpu) -> u16 {
    cpu.regs.SP.wrapping_add(2);
    let addr = cpu.mem.read16(cpu.regs.SP);
    debug!("Poping {:04X} from stack at {:04X}", addr, cpu.regs.SP);
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
        PushStack(self, addr);
        self.regs.PC = 0x0040;
    }

    pub fn reset(&mut self) {
        self.regs.PC = 0x0000
    }

    pub fn step(&mut self) -> u8 {
        let code = self.mem.read8(self.regs.PC) as usize;
        let (length, cur_cycle_count) = match code {
            0x00 => { (1, 1) },
            0x21 => {let v = self.imm16(); self.regs.set_HL(v); (3, 1)},
            0x31 => {let sp = self.imm16(); self.regs.SP = sp; (3, 1)},
            0x32 => {self.mem.write8(self.regs.HLd(), self.regs.A); (1, 1)}
            0xAF => {let A  = self.regs.A; self.alu_xor(A); (1, 1)},
            _    => { (0, 1) }
        };
        let cur_pc = self.regs.get_PC();
        self.regs.set_PC(cur_pc + length);

        debug!("----------------------------------------");
        debug!("{:04X}: {:02X} -> ", self.regs.PC, code);
        self.print_status();
        debug!("----------------------------------------");
        self.total_cyles = self.total_cyles + cur_cycle_count as u64;
        1
    }


    fn alu_xor(&mut self, b: u8) {
        let r = self.regs.A ^ b;
        self.regs.set_FZ(r == 0);
        self.regs.set_FC(false);
        self.regs.set_FH(false);
        self.regs.set_FN(false);
        self.regs.A = r;
    }
}
