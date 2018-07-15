use rom;

// Memory controller
#[derive(Clone, Debug, Default)]
pub struct Mem<'a> {
    size: u16,
    rom:  rom::ROM<'a>,
}

impl<'a> Mem<'a>{
    pub fn new(arom: rom::ROM) -> Mem {
        Mem{
            size: 0xFFFF,
            rom: arom,
        }
    }
    pub fn read8(&self, addr: u16) -> u8 {
        //println!("[{:04X}] >>> {:02X}", addr, self.rom.buffer[addr as usize]);
        self.rom.buffer[addr as usize]
    }
    pub fn read16(&self, addr: u16) -> u16 {
        let v = (((self.rom.buffer[(addr+1) as usize] as u16)<<8)|(self.rom.buffer[addr as usize]) as u16) as u16;
        //println!("[{:04X}] >>> {:04X}", addr, v);
        v
    }
}
