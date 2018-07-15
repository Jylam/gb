use rom;

// Memory controller
#[derive(Clone, Debug, Default)]
pub struct Mem<'a> {
    size: u16,
    rom:  &'a rom::ROM<'a>,
}

impl<'a> Mem<'a>{
    pub fn new(arom: rom::ROM) -> Mem<'a> {
        Mem{
            size: 0xFFFF,
            rom: &arom,
        }
    }
    pub fn read8(self) -> u8 {
        0
    }
}
