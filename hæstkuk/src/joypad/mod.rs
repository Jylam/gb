#![allow(dead_code)]
use std::marker::PhantomData;

// Joypad controller
#[derive(Clone, Debug, Default)]
pub struct Joypad<'a> {
	input: u8,
	phantom: PhantomData<&'a u8>,
    debug: bool
}


impl<'a> Joypad<'a>{
	pub fn new() -> Joypad<'a> {
		Joypad{
			input: 0b00111111,
			phantom: PhantomData,
            debug: false
		}
	}
	pub fn write8(&mut self, v: u8)  {
        self.input = v & 0b00110000;
    }

	pub fn read8(&self) -> u8 {
        if ((self.input&0b00100000) >> 5) == 0 { // Action

        } else if ((self.input&0b00010000) >> 4) == 0 {  // Direction

        }
	    self.input
    }
}


