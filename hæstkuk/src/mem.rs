use rom;
use lcd;

// Memory controller
#[derive(Clone, Debug, Default)]
pub struct Mem<'a> {
    size: u16,
    rom:  rom::ROM<'a>,
    ram: Vec<u8>,
    pub lcd:  lcd::LCD<'a>,
}

impl<'a> Mem<'a>{
    pub fn new(arom: rom::ROM<'a>, alcd: lcd::LCD<'a>) -> Mem<'a> {
        Mem{
            size: 0xFFFF,
            rom: arom,
            ram: vec![0x00; 65536],
            lcd: alcd,
        }
    }
    pub fn read8(&mut self, addr: u16) -> u8 {
        //debug!("[{:04X}] >>> {:02X}", addr, self.rom.buffer[addr as usize]);
        self.lcd.update();
        match addr {
            0x0000..=0x7FFF => self.rom.buffer[addr as usize],
            0xFF40...0xFF54 => self.lcd.read8(addr-0xFF40),
            0xFF00 ... 0xFF7F => { debug!("Unsupported read8 in Hardware area {:04X}", addr); self.ram[addr as usize]},
            _ => {self.ram[addr as usize]},
        }
    }
    pub fn write8(&mut self, addr: u16, v: u8)  {
        debug!(">>> Writing {:02X} at {:04X}", v, addr);
        match addr {
            0x0000..=0x7FFF => { self.rom.buffer[addr as usize] = v;},
            0xFF01 => {debug!("WRITE DEBUG {:02X}", v);}
            0xFF02 => {debug!("WRITE DEBUG2 {:02X}", v);}
            0xFF40...0xFF54 => {self.lcd.write8(addr-0xFF40, v)},
            //0xFF00 ... 0xFF7F => { debug!("Unsupported write8 in Hardware area {:04X}", addr);},
            _ => {self.ram[addr as usize] = v;},
        }
    }
    pub fn read16(&mut self, addr: u16) -> u16 {
        let v = ((self.read8(addr+1) as u16)<<8)|(self.read8(addr) as u16);
        v
    }
    pub fn write16(&mut self, addr: u16, v: u16)  {
		match addr {
			0x0000..=0x7FFF => {self.write8(addr+1,  ((v&0xFF00)>>8) as u8);
				self.write8(addr, (v&0xFF)       as u8);}
			0xFF40...0xFF54 => {error!("WRITE16 ON LCD !!!")},
			_ => {self.write8(addr+1,  ((v&0xFF00)>>8) as u8);
                  self.write8(addr, (v&0xFF)       as u8);}
		}
	}

#[allow(dead_code)]
		pub fn print_infos(&mut self) {
			debug!("Zero Page    (0xFF80..0xFFFF) : {:02X?}", self.ram[0xFF80..=0xFFFF].to_vec());
			debug!("Hardware I/O (0xFF00..0xFF7F) : {:02X?}", self.ram[0xFF00..=0xFF7F].to_vec())
		}
	}
