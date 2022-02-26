#![allow(dead_code)]
use std::marker::PhantomData;

// LCD controller
#[derive(Clone, Debug, Default)]
pub struct LCD<'a> {
	regs: Vec<u8>,
	phantom: PhantomData<&'a u8>,
    debug: bool
}


impl<'a> LCD<'a>{
	pub fn new() -> LCD<'a> {
		LCD{
			regs: vec![0x00; 0x15],
			phantom: PhantomData,
            debug: false
		}
	}
	pub fn write8(&mut self, addr: u16, v: u8)  {
        let addr = addr-0xFF40;
		debug!("LCD Write8 {:02X} at {:04X}", v, addr);
		match addr {
			0..=15 => {if self.debug {println!("LCD write8 {:02X} at {:04X}", v, addr+0xFF40);}; self.regs[(addr) as usize] = v;}
			_ => {error!("LCD Write8 range error")}
		}
	}

	pub fn read8(&self, addr: u16) -> u8 {
        let addr = addr-0xFF40;
		match addr {
			0..=15 => {if self.debug {println!("LCD read8 {:02X} at {:04X}", self.regs[addr as usize], addr+0xFF40);}; self.regs[addr as usize]}
			_ => {error!("LCD Read8 range error"); 0}
		}
	}

	pub fn update(&mut self) {

        // Update LY at FF44
        let mut ly = self.read8(0xFF44) as u8;
        if ly==153 {
            ly = 0;
        } else {
            ly = ly.wrapping_add(1);
        }
        self.write8(0xFF44, ly);

        // Update LYC 0xFF45  at STAT 0xFF41
        let lyc = self.read8(0xFF45) as u8;
        let mut stat = self.read8(0xFF41) as u8;
        if ly == lyc {
            stat = stat | (1 << 2);
        } else {
            stat = stat & !(1 << 2)
        }

        // Update mode
        if ly>=144 {
            stat = stat | 0x01; // VBlank
        } else {
            stat = stat & !0x01;
        }

        self.write8(0xFF45, stat);

    }

    // Get palette and return the color between 0..3 (3 being white, 0 black)
	pub fn get_bw_palette(&mut self) -> Vec<u8> {
        let pal = self.read8(0xFF47) as u8;
        let col3 = (pal&0b11000000) >> 6;
        let col2 = (pal&0b00110000) >> 4;
        let col1 = (pal&0b00001100) >> 2;
        let col0 = (pal&0b00000011) >> 0;

        let convert = vec![0b11, 0b10, 0b01, 0b00];

        vec![convert[col0 as usize], convert[col1 as usize], convert[col2 as usize], convert[col3 as usize]]
    }

}
