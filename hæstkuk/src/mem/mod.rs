#![allow(unused_variables)]
#![allow(dead_code)]
use std::fs::File;
use std::io::Read;
use rom;
use lcd;
use joypad;
use timer;
use MBC1;
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
    mbc1_bank: u8,
    ram_bank: u8,
    ram_mode: bool,
    ram_enabled: bool,
    mbc1: MBC1::MBC1<'a>,
}

impl<'a> Mem<'a>{
    pub fn new(arom: rom::ROM<'a>, alcd: lcd::LCD<'a>, ajoypad: joypad::Joypad<'a>, atimer: timer::Timer<'a>) -> Mem<'a> {
        let mut mem = Mem{
            _size: 0xFFFF,
            rom: arom,
            ram: vec![0x00; 16384*200],
            lcd: alcd,
            joypad: ajoypad,
            timer: atimer,
            bootrom_enable: true,
            mbc1_bank: 0x01,
            ram_bank: 0x00,
            ram_mode: false,
            ram_enabled: false,
            mbc1: MBC1::MBC1::new(),
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
            0x0000..=0x7FFF => {
                if self.rom.get_mbc() == 0x00 {
                    self.read8_rom(addr)
                }  else if self.rom.get_mbc() == 0x01 {
                    self.read8_mbc1(addr)
                } else {
                    self.read8_mbc1(addr) // Use MBC1 FIXME
                }
            }
            0xA000..= 0xBFFF => {
                if self.rom.get_mbc() == 0x00 {
                    self.ram[addr as usize]
                }  else if self.rom.get_mbc() == 0x01 {
                    self.read8_mbc1(addr)
                } else {
                    self.read8_mbc1(addr) // Use MBC1 FIXME
                }
            }
            // LCD
            0xFF40..=0xFF4F => { self.lcd.read8(addr) },
            // Joypad
            0xFF00          => { self.joypad.read8() },
            // Timer
            0xFF04..=0xFF07 => { self.timer.read8(addr) },
            // IF
            0xFF0F          => { self.ram[addr as usize]}, // IF - Interrupt Flag (R/W)
            0xFFFF          => { self.ram[addr as usize]}, // IE - Interrupt Enable (R/W)

            _ => {
                self.ram[addr as usize]},
        }
    }

    pub fn dump_mem(&mut self, addr: u16, len: u16) {
        print!("{:04X}: ", addr);
        for i in addr..addr+len {
            if i%16==0 {
                print!("\n{:04X}: ", i);
            }
            print!("{:02X} ", self.read8(i));
        }
        println!("");
    }

    // Read from the ROM without any MBC
    pub fn read8_rom(&mut self, addr: u16) -> u8 {
        match addr {
            // BOOTROM or Interrupt Vectors
            0x0000..=0x00FF => if self.bootrom_enable {self.bootrom[addr as usize] } else {self.rom.buffer[addr as usize]},
            // Cartridge ROM
            0x0100..=0x7FFF => self.rom.buffer[addr as usize],
            _ => {println!("ERROR READING AT {:02X}", addr); 0x00}
        }
    }

    // Read from MBC1
    pub fn read8_mbc1(&mut self, addr: u16) -> u8 {
        match addr {
            // BOOTROM or Interrupt Vectors
            0x0000..=0x00FF => if self.bootrom_enable {
                self.bootrom[addr as usize]
            } else {
                self.rom.buffer[addr as usize]
            },
            // Cartridge ROM, Bank 0
            0x0100..=0x3FFF => self.rom.buffer[addr as usize],
            // Cartridge ROM, selected bank
            0x4000..=0x7FFF => {
                let offset = (addr as u32 - 0x4000)+(0x4000*self.mbc1_bank as u32);
                self.rom.buffer[offset as usize]
            },
            // Cartridge RAM
            0xA000..= 0xBFFF=>
            {
                if self.ram_enabled == false {
                    println!("Reading while RAM disabled");
                    return 0;
                }
                let mut bank = 0;
                if self.ram_mode {
                    bank = self.ram_bank
                }
                self.ram[(bank as usize * 0x2000) | ((addr & 0x1FFF) as usize)]
            },
            _ => {println!("ERROR READING AT {:02X}", addr); 0x00}
        }
    }

    pub fn write8(&mut self, addr: u16, v: u8)  {
        match addr {
            0x0000..=0x00FF => {
                if addr<=0x00FF {
                    if self.bootrom_enable {
                        return;
                    }
                }
                if v == 0x00 {
                    self.ram_enabled = false;
                } else if (v&0x0F) == 0x0A {
                    self.ram_enabled = true;
                } else {
                    self.ram[addr as usize] = v;
                }
            },
            // Bank select register
            0x2000..=0x3FFF => {
                if self.rom.get_mbc() == 0x01 {
                    self.mbc1_bank = (self.mbc1_bank & 0x60) | (v & 0x1F);
                    if (self.mbc1_bank == 0x20) ||
                        (self.mbc1_bank == 0x40) ||
                            (self.mbc1_bank == 0x60) {
                                self.mbc1_bank+=1;
                            }
                } else {
                    println!("WRITE ROM WITH NO MBC {:02X}", addr);
                }
            }
            0x4000..=0x5FFF => {
                if !self.ram_mode {
                    self.mbc1_bank = self.mbc1_bank & 0x1F | (((v as u8) & 0x03) << 5);
                    println!("Changing MBC1 bank to {}", self.mbc1_bank);
                } else {
                    self.ram_bank = (v as u8) & 0x03;
                    println!("Changing RAM bank to {}", self.ram_bank);
                }
            },
            // Cartridge RAM enable
            0x6000..=0x7FFF => {
                self.ram_mode = (v & 0x01) == 0x01;
                println!("Changing RAM mode to {}", self.ram_mode);
            }
            // Cartridge RAM
            0xA000..= 0xBFFF=> {
                if self.ram_enabled == false {
                    println!("RAM NOT ENABLED");
                    return;
                }
                let mut bank = 0;
                if self.ram_mode {
                    bank = self.ram_bank
                }
                self.ram[(bank as usize * 0x2000) | ((addr & 0x1FFF) as usize)] = v
            },
            0xFF40..=0xFF4F => {
                // OAM DMA
                if addr == 0xFF46 {
                    let start = (v as u16)<<8;
                    for i in 0x00..0x9F {
                        let value = self.read8(start+i);
                        self.write8(0xFE00+i, value);
                    }
                } else {
                    self.lcd.write8(addr, v)
                }
            },
            0xFF50 =>          { self.bootrom_enable = false; println!("Disabling BOOTROM");}
            0xFF00 =>          { self.joypad.write8(v);},
            0xFF04..=0xFF07 => { self.timer.write8(addr, v) },
            // IE
            0xFFFF => {self.ram[addr as usize] = v;}
            _ => {self.ram[addr as usize] = v;},
        }
    }
    pub fn read16(&mut self, addr: u16) -> u16 {
        let v = ((self.read8(addr+1) as u16)<<8)|(self.read8(addr) as u16);
        v
    }
    pub fn write16(&mut self, addr: u16, v: u16)  {
        self.write8(addr+1,  ((v&0xFF00)>>8) as u8);
        self.write8(addr, (v&0xFF)       as u8);
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
