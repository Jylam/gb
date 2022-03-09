#![allow(unused_variables)]
#![allow(dead_code)]
use std::fs::File;
use std::io::Read;
use rom;
use lcd;
use joypad;
use timer;

// Memory controller
#[derive(Clone, Debug, Default)]
pub struct Mem<'a> {
    _size: u16,
    bootrom: Vec<u8>,
    bootrom_enable: bool,
    rom:  rom::ROM<'a>,
    ram: Vec<u8>,
    pub lcd:  lcd::LCD<'a>,
    pub joypad: joypad::Joypad<'a>,
    pub timer: timer::Timer<'a>,
}

impl<'a> Mem<'a>{
    pub fn new(arom: rom::ROM<'a>, alcd: lcd::LCD<'a>, ajoypad: joypad::Joypad<'a>, atimer: timer::Timer<'a>) -> Mem<'a> {
        let mut mem = Mem{
            _size: 0xFFFF,
            rom: arom,
            ram: vec![0x00; 65536],
            lcd: alcd,
            joypad: ajoypad,
            timer: atimer,
            bootrom_enable: true,
        ..Default::default()
        };

        let mut f = File::open("./DMG_ROM.bin".to_string()).expect("File not found");
        let read_size = f.read_to_end(&mut mem.bootrom).expect("Can't read bootrom");
        println!("Boot ROM: {} bytes", read_size);
        mem
    }
    pub fn is_bootrom_enabled(&mut self) -> bool {
        self.bootrom_enable
    }
    pub fn read8(&mut self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x00FF => if self.bootrom_enable {self.bootrom[addr as usize] } else {self.rom.buffer[addr as usize]},
            0x0100..=0x7FFF => self.rom.buffer[addr as usize],
            0xFF40..=0xFF4F => { self.lcd.read8(addr) },
            0xFF00          => { self.joypad.read8() },
            0xFF04..=0xFF07 => { self.timer.read8(addr) },

            0xFF0F          => { self.ram[addr as usize]}, // IF - Interrupt Flag (R/W)
            0xFFFF          => { self.ram[addr as usize]}, // IE - Interrupt Enable (R/W)

            _ => {self.ram[addr as usize]},
        }
    }
    pub fn write8(&mut self, addr: u16, v: u8)  {
        match addr {
            0x0000..=0x00FF => if self.bootrom_enable {self.bootrom[addr as usize] = v;} else {self.rom.buffer[addr as usize] = v;},
            0x0100..=0x7FFF => { self.rom.buffer[addr as usize] = v;},
            0xFF40..=0xFF4F => { self.lcd.write8(addr, v)},
            0xFF50 => {self.bootrom_enable = false; println!("Disabling BOOTROM");}
            0xFF00 => {self.joypad.write8(v);},
            0xFF04..=0xFF07 => { self.timer.write8(addr, v) },
            _ => {self.ram[addr as usize] = v;},
        }
    }
    pub fn read16(&mut self, addr: u16) -> u16 {
        let v = ((self.read8(addr+1) as u16)<<8)|(self.read8(addr) as u16);
        v
    }
    pub fn write16(&mut self, addr: u16, v: u16)  {
		match addr {
            0x0000..=0x7FFF => {
                self.write8(addr+1,  ((v&0xFF00)>>8) as u8);
                self.write8(addr, (v&0xFF)       as u8);}
            _ => {
                self.write8(addr+1,  ((v&0xFF00)>>8) as u8);
                self.write8(addr, (v&0xFF)       as u8);}
        }
    }

    pub fn display(&mut self, addr: u16, size: u16) {
        let mut cnt = 0;
        for i in addr..addr+size {
            print!("{:02X}", self.read8(i));
			if (cnt%32) == 0 {
				println!("");
			}
			cnt+=1
		}
	}


#[allow(dead_code)]
	pub fn print_infos(&mut self) {
		debug!("Zero Page    (0xFF80..0xFFFF) : {:02X?}", self.ram[0xFF80..=0xFFFF].to_vec());
		debug!("Hardware I/O (0xFF00..0xFF7F) : {:02X?}", self.ram[0xFF00..=0xFF7F].to_vec())
	}
}
