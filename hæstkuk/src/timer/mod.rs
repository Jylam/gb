#![allow(dead_code)]
use std::marker::PhantomData;

// Timer
#[derive(Clone, Debug, Default)]
pub struct Timer<'a> {
	phantom: PhantomData<&'a u8>,
	div:  u8,
    tima: u8,
    tma:  u8,
    tac:  u8
}


impl<'a> Timer<'a>{
	pub fn new() -> Timer<'a> {
		Timer{
			phantom: PhantomData,
			div: 0,
            tima: 0,
            tma: 0,
            tac: 0
		}
	}
}
