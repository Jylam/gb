#![allow(dead_code)]
use std::marker::PhantomData;

// LCD controller
#[derive(Clone, Debug, Default)]
pub struct LCD<'a> {
	regs: Vec<u8>,
	phantom: PhantomData<&'a u8>,
    debug: bool,
    vblank: bool,
    max_cycles: u64,
    counter: u64,
    mode: usize,
    mode0_counter: u64,
    mode1_counter: u64,
    mode2_counter: u64,
    mode3_counter: u64,
    need_render: bool,
    need_new_line: bool,
    t: u32,
}


impl<'a> LCD<'a>{
    pub fn new() -> LCD<'a> {
        LCD{
            regs: vec![0x00; 0x15],
            phantom: PhantomData,
            debug: false,
            vblank: false,
            max_cycles: 70224,
            counter: 0,
            mode: 0,
            mode0_counter: 0,
            mode1_counter: 0,
            mode2_counter: 0,
            mode3_counter: 0,
            need_render: true,
            need_new_line: true,
            t: 0
        }
    }
    pub fn write8(&mut self, addr: u16, v: u8)  {
        //println!("LCD write8 {:02X} at {:04X}", v, addr);
        match addr {
            // DMA OAM, handled in mem.rs
            0xFF46 => {println!("ERROR OAM DMA {:04X} -> {:02X}", addr, v);}
            0xFF47..=0xFF49 => {self.regs[(addr-0xFF40) as usize] = v; println!("Writing {:04X}: {:08b}", addr, v);}
            _ => {self.regs[(addr-0xFF40) as usize] = v;}
        }
    }

    pub fn read8(&self, addr: u16) -> u8 {
        //println!("LCD read8 at {:04X}", addr);
        match addr {
            0xFF40..=0xFF4F => {self.regs[(addr-0xFF40) as usize]}
            _ => {error!("LCD read8 range error"); 0}
        }
    }

    pub fn int_vblank(&mut self) -> bool {
        if self.vblank {
            self.vblank = false;
            true
        } else {
            false
        }
    }

    pub fn get_tile_addr(&mut self, id: u8, is_sprite: bool) -> u16 {
        if is_sprite {
            0x8000+((id as usize)*16) as u16
        } else {
            if self.regs[0]&0b0001_0000 == 0 {
                let offset = ((id as i8) as i16 *16 as i16) as i16;
                let a = ((0x9000 as i32 + offset as i32) as u32) as u16;
                a
            } else {
                0x8000+((id as usize)*16) as u16
            }
        }
    }

    pub fn update(&mut self, cur_cycles: u64) {
        self.counter += cur_cycles;

        match self.mode {
            0=>{
                self.mode0_counter+=cur_cycles;
                if self.mode0_counter >= 201 {
                    // Update LY at FF44
                    let mut ly = self.read8(0xFF44) as u8;
                    if ly==153 {
                        ly = 0;
                    } else {
                        ly = ly.wrapping_add(1);
                    }
                    self.need_new_line = true;
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
                    if ly == 144 {
                        self.mode = 1;
                        stat &= 0b1111_1100;
                        stat = stat | 0x01;
                        self.vblank = true;
                        self.need_render = true;
                    }
                    if ly == 0 {
                        self.mode = 2;
                        stat &= 0b1111_1100;
                        stat |= 0b0000_0010;
                        self.vblank = false;
                    }

                    self.write8(0xFF41, stat);
                    self.mode0_counter = 0;
                }
            },
            1=>{
                self.mode1_counter+=cur_cycles;
                if self.mode1_counter >= 4560 {
                    self.mode1_counter = 0;
                    self.mode = 2;
                    let mut stat = self.read8(0xFF41) as u8;
                    stat &= 0b1111_1100;
                    stat |= 0b0000_0010;
                    self.write8(0xFF41, stat);
                }
            },
            2=>{
                self.mode2_counter+=cur_cycles;
                if self.mode2_counter >= 80 {
                    self.mode2_counter = 0;
                    self.mode = 3;
                    let mut stat = self.read8(0xFF41) as u8;
                    stat &= 0b1111_1100;
                    stat |= 0b0000_0011;
                    self.write8(0xFF41, stat);
                }
            },
            3=>{
                self.mode3_counter+=cur_cycles;
                if self.mode3_counter >= 169 {
                    self.mode3_counter = 0;
                    self.mode = 0;
                    let mut stat = self.read8(0xFF41) as u8;
                    stat &= 0b1111_1100;
                    self.write8(0xFF41, stat);
                }
            },
            _=>{println!("Wrong mode !");}
        }


        if self.counter >= self.max_cycles {
            self.counter = 0;
        }
    }

    pub fn need_new_line(&mut self) -> bool {
        if self.need_new_line {
            self.need_new_line = false;
            true
        } else {
            false
        }
    }
    pub fn need_render(&mut self) -> bool {
        if self.need_render {
            self.need_render = false;
            true
        } else {
            false
        }
    }
    pub fn get_cur_y(&mut self) -> u8 {
        self.read8(0xFF44) as u8
    }
    pub fn get_scy(&mut self) -> u8 {
        self.regs[2]
    }
    pub fn get_scx(&mut self) -> u8 {
        self.regs[3]
    }

    pub fn int_stat(&mut self) -> bool {
        let mut s = self.read8(0xFF41) & 0b0000_0100;
        // TODO
        if s != 0 {
            s = s & !0x04;
            self.write8(0xFF41, s);
            true
        } else {
            false
        }
    }

    // Get palette and return the color between 0..3 (0 White, 1 Light gray, 2 Dark gray, 3 Black)
    pub fn get_palette(&mut self, addr: u16) -> Vec<u8> {
        let pal =  self.read8(addr);
        let col0 = (pal&0b00000011) >> 0;
        let col1 = (pal&0b00001100) >> 2;
        let col2 = (pal&0b00110000) >> 4;
        let col3 = (pal&0b11000000) >> 6;

        let convert = vec![0b11, 0b01, 0b10, 0b00];

        vec![convert[col0 as usize], convert[col1 as usize], convert[col2 as usize], convert[col3 as usize]]
    }
    pub fn get_bw_palette(&mut self) -> Vec<u8> {
        self.get_palette(0xFF47)
    }
    pub fn get_sprite_palette(&mut self, id: u16) -> Vec<u8> {
        self.get_palette(0xFF48+id)
    }

}
