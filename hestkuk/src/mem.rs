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
        self.rom.buffer[addr as usize]
    }
}
