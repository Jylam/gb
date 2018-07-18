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
    pub fn read8(&self, addr: u16) -> u8 {
        match addr {
            _ => {println!("LCD read8 at {:04X}", addr); self.regs[(addr-0xFF40) as usize]}
        }
    }
}
