#![allow(dead_code)]
use std::marker::PhantomData;


// LCD controller
#[derive(Clone, Debug, Default)]
pub struct LCD<'a> {
	regs: Vec<u8>,
	phantom: PhantomData<&'a u8>,
}


impl<'a> LCD<'a>{
	pub fn new() -> LCD<'a> {
		LCD{
			regs: vec![0x00; 0x15],
			phantom: PhantomData,
		}
	}
	pub fn write8(&mut self, addr: u16, v: u8)  {
		debug!("LCD Write8 {:02X} at {:04X}", v, addr);
		match addr {
			0..=15 => {self.regs[(addr) as usize] = v;}
			_ => {error!("LCD Write8 range error")}
		}
	}

	pub fn read8(&self, addr: u16) -> u8 {
		match addr {
			_ => {debug!("LCD read8 at {:04X}", addr); self.regs[addr as usize]}
		}
	}

	pub fn update(&mut self) {
		self.regs[(0x04)] = self.regs[(0x04)].wrapping_add(1);

    }


	pub fn operation(&self) -> bool {
		!((self.regs[0x00]&(1<<7))==0)
	}
	pub fn window_tile_table_address(&self) -> u16 {
		if !((self.regs[0x00]&(1<<6))==0) {
			0x9C00
		} else {
			0x9800
		}
	}
	pub fn window_display(&self) -> bool {
		!((self.regs[0x00]&(1<<5))==0)
	}
	pub fn tile_pattern_table_address(&self) -> u16 {
		if !((self.regs[0x00]&(1<<4))==0) {
			0x8000
		} else {
			0x8800
		}
	}

	pub fn background_tile_table_address(&self) -> u16 {
		if !((self.regs[0x00]&(1<<3))==0) {
			0x9C00
		} else {
			0x9800
		}
	}
	pub fn sprite_size(&self) -> bool {
		!((self.regs[0x00]&(1<<2))==0)
	}
	pub fn color_0_transparency(&self) -> bool {
		(self.regs[0x00]&(1<<1))==0
	}

	pub fn background_display(&self) -> bool {
		!((self.regs[0x00]&1)==0)
	}
	pub fn scroll_x(&self) -> u8 {
		self.regs[0x03]
	}
	pub fn scroll_y(&self) -> u8 {
		self.regs[0x02]
	}
	pub fn curline(&self) -> u8 {
		self.regs[0x04]
	}
	pub fn cmpline(&self) -> u8 {
		self.regs[0x05]
	}
	pub fn win_pos_y(&self) -> u8 {
		self.regs[0x0A]
	}
	pub fn win_pos_x(&self) -> u8 {
		self.regs[0x0B]
	}

	/*
	   FF40 -- LCDCONT [RW] LCD Control              | when set to 1 | when set to 0
	   Bit7  LCD operation                           | ON            | OFF
	   Bit6  Window Tile Table address               | 9C00-9FFF     | 9800-9BFF
	   Bit5  Window display                          | ON            | OFF
	   Bit4  Tile Pattern Table address              | 8000-8FFF     | 8800-97FF
	   Bit3  Background Tile Table address           | 9C00-9FFF     | 9800-9BFF
	   Bit2  Sprite size                             | 8x16          | 8x8
	   Bit1  Color #0 transparency in the window     | SOLID         | TRANSPARENT
	   Bit0  Background display                      | ON            | OFF
	   */
}
