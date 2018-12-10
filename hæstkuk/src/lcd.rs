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
}
