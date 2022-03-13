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
    div_cycle: u64,
    tima_cycle: u64,
    timer_enable: bool,
    tima_freq: u64,

    interrupt: bool,
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
            div_cycle: 0,
            tima_cycle: 0,
            tima_freq: 0,
            timer_enable: false,
            interrupt: false,
        }
    }

    pub fn update(&mut self, cycles: u64) {
        self.div_cycle+=cycles;

        // 16384 Hz DIV timer
        if self.div_cycle >= (self.mhz / 16384) {
            self.div_cycle = 0;
            self.div = self.div.wrapping_add(1);
        }
        if self.timer_enable {
            self.tima_cycle+=cycles;
            if self.tima_cycle >= (self.mhz / self.tima_freq) {
                self.tima_cycle = 0;
                self.tima = self.tima.wrapping_add(1);
                if self.tima == 0x00 { // Overflow
                    self.tima = self.tma;
                    self.interrupt = true;
                } else {
                    self.interrupt = false;
                }
            }
        }
    }
    pub fn int_timer(&mut self) -> bool {
        if self.interrupt {
            self.interrupt = false;
            true
        } else {
            false
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
        println!("TIMER write {:02X} at {:04X}", v, addr);
        match addr {
            0xFF04 => {self.div = 0;},
            0xFF05 => {self.tima = v},
            0xFF06 => {self.tma = v},
            0xFF07 => {self.tac = v;
                if self.tac&0b0000_0100 != 0 {
                    self.timer_enable = true;
                } else {
                    self.timer_enable = false;
                }
                self.tima_freq = match self.tac&0b0000_0011 {
                    0b00 => {self.mhz/1024},
                    0b01 => {self.mhz/16},
                    0b10 => {self.mhz/64},
                    0b11 => {self.mhz/256},
                    _ => {0},
                }
            },
            _ => {error!("Timer write8 range error");}
        }

    }

}
