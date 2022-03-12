#![allow(dead_code)]
use std::marker::PhantomData;

// Joypad controller
#[derive(Clone, Debug, Default)]
pub struct Joypad<'a> {
	input: u8,
	phantom: PhantomData<&'a u8>,
    debug: bool,
    btn_a:       bool,
    btn_b:       bool,
    btn_select:  bool,
    btn_start:   bool,
    btn_left:    bool,
    btn_right:   bool,
    btn_up:      bool,
    btn_down:    bool,
    interrupt:   bool,
}


impl<'a> Joypad<'a>{
	pub fn new() -> Joypad<'a> {
		Joypad{
			input: 0b0011_1111,
			phantom: PhantomData,
            debug: false,
            btn_a:      false,
            btn_b:      false,
            btn_select: false,
            btn_start:  false,
            btn_left:   false,
            btn_right:  false,
            btn_up:     false,
            btn_down:   false,
            interrupt:  false,
		}
	}
	pub fn write8(&mut self, v: u8)  {
        self.input = v & 0b0011_0000;
    }

    pub fn read8(&mut self) -> u8 {
        self.input
    }

    pub fn update(&mut self) {
        if ((self.input&0b0010_0000) >> 5) == 0 { // Action
            let old_input = self.input;
            self.input = (self.input&0b1111_0000) |
                ((!self.btn_start  as u8) << 3) |
                ((!self.btn_select as u8) << 2) |
                ((!self.btn_b      as u8) << 1) |
                !self.btn_a        as u8;
            if self.input&0b0000_1111 != old_input&0b0000_1111 {
                self.interrupt = true;
            } else {
                self.interrupt = false;
            }

        } else if ((self.input&0b0001_0000) >> 4) == 0 {  // Direction
            let old_input = self.input;
            self.input = (self.input&0b1111_0000) |
                ((!self.btn_down  as u8) << 3) |
                ((!self.btn_up as u8) << 2) |
                ((!self.btn_left      as u8) << 1) |
                !self.btn_right        as u8;
            if self.input&0b0000_1111 != old_input&0b0000_1111 {
                self.interrupt = true;
            } else {
                self.interrupt = false;
            }
        }
    }

    pub fn int_joypad(&mut self) -> bool {
        self.interrupt
    }

    pub fn set_a(&mut self, val: bool) {
        self.btn_a = val;
    }
    pub fn set_b(&mut self, val: bool) {
        self.btn_b = val;
    }
    pub fn set_select(&mut self, val: bool) {
        self.btn_select = val;
    }
    pub fn set_start(&mut self, val: bool) {
        self.btn_start = val;
    }
    pub fn set_left(&mut self, val: bool) {
        self.btn_left = val;
    }
    pub fn set_right(&mut self, val: bool) {
        self.btn_right = val;
    }
    pub fn set_up(&mut self, val: bool) {
        self.btn_up = val;
    }
    pub fn set_down(&mut self, val: bool) {
        self.btn_down = val;
    }
}


