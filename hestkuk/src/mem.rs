use rom;

// Memory controller
#[derive(Clone, Debug, Default)]
pub struct Mem<'a> {
    size: u16,
    rom:  rom::ROM<'a>,
    ram: Vec<u8>,
}

impl<'a> Mem<'a>{
    pub fn new(arom: rom::ROM) -> Mem {
        Mem{
            size: 0xFFFF,
            rom: arom,
            ram: vec![0x00; 65536],
        }
    }
    pub fn read8(&self, addr: u16) -> u8 {
        //println!("[{:04X}] >>> {:02X}", addr, self.rom.buffer[addr as usize]);

        match addr {
            0x0100..=0x3FFF => self.rom.buffer[addr as usize],
            //0xFF00 ... 0xFF7F => { println!("Unsupported read8 in Hardware area {:04X}", addr); 0xFF},
            _ => {self.ram[addr as usize]},
        }

    }
    pub fn write8(&mut self, addr: u16, v: u8)  {
        println!(">>> Writing {:02X} at {:04X}", v, addr);
        match addr {
            0x0100..=0x3FFF => { self.rom.buffer[addr as usize] = v;},
            //0xFF00 ... 0xFF7F => { println!("Unsupported write8 in Hardware area {:04X}", addr);},
            _ => {self.ram[addr as usize] = v;},
        }
    }
    pub fn read16(&self, addr: u16) -> u16 {
        let v = (((self.rom.buffer[(addr+1) as usize] as u16)<<8)|(self.rom.buffer[addr as usize]) as u16) as u16;
        //println!("[{:04X}] >>> {:04X}", addr, v);
        v
    }
    pub fn write16(&mut self, addr: u16, v: u16)  {
        if addr <= 0x7FFF {
            println!("[{:04X}] <<< {:02X}", addr, v);
            self.rom.buffer[addr as usize] = ((v&0xFF00)>>8) as u8;
            self.rom.buffer[(addr+1) as usize] = ((v&0x00FF)) as u8;
        } else {
            println!("!!!! Non-existent memory location ${:04X}", addr)
        }
    }

    pub fn print_infos(&mut self) {
        println!("Zero Page   (0xFF80..0xFFFF) : {:02X?}", self.ram[0xFF80..=0xFFFF].to_vec());
        println!("Harware I/O (0xFF00..0xFF7F) : {:02X?}", self.ram[0xFF00..=0xFF7F].to_vec())
    }
}
