#![allow(non_snake_case)]
use std::marker::PhantomData;


#[derive(Clone, Debug, Default)]
pub struct MBC1<'a> {
    phantom: PhantomData<&'a u8>,
    rom_bank: u8,
    ram_bank: u8,
    ram_mode: bool,
    ram_enabled: bool,
}

impl<'a> MBC1<'a>{

    pub fn new() -> MBC1<'a> {
       let mbc1 = MBC1{
            phantom: PhantomData,
            rom_bank: 0,
            ram_bank: 0,
            ram_mode: false,
            ram_enabled: false
       };
       mbc1
    }

    pub fn is_valid_for_id(&mut self, id: u8) -> bool {
        if id == 0x01 || id == 0x02 || id == 0x03 {
            true
        } else {
            false
        }
    }

    pub fn read8(&mut self, addr: u16) -> u8 {
    0
    }

    pub fn write8(&mut self, addr: u16, v: u8)  {
    }
}

