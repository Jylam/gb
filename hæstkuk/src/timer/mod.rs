#![allow(dead_code)]
use std::marker::PhantomData;

// Timer
#[derive(Clone, Debug, Default)]
pub struct Timer<'a> {
	phantom: PhantomData<&'a u8>,
	div:  u8,
    tima: u8,
    tma:  u8,
    tac:  u8,

    mhz: u64,
    cur_cycle: u64,

}


impl<'a> Timer<'a>{
	pub fn new(mhz: u64) -> Timer<'a> {
		Timer{
			phantom: PhantomData,
			div: 0,
            tima: 0,
            tma: 0,
            tac: 0,
            mhz: mhz,
            cur_cycle: 0,
		}
	}

    pub fn update(&mut self, cycles: u64) {
        self.cur_cycle+=cycles;

        // 16384 Hz DIV timer
        if self.cur_cycle >= (self.mhz / 16384) {
            self.cur_cycle = 0;
            self.div = self.div.wrapping_add(1);
        }
    }

    pub fn read8(&mut self, addr: u16) -> u8 {
        match addr {
            0xFF04 => {self.div},
            0xFF05 => {self.tima},
            0xFF06 => {self.tma},
            0xFF07 => {self.tac},
			_ => {error!("Timer read8 range error"); 0}
        }
    }
    pub fn write8(&mut self, addr: u16, v: u8) {

    }

}
