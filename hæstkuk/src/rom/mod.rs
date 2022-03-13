// ROM
use std::io;
use std::io::prelude::*;
use std::fs::File;
use std::fs;
use std::borrow::Cow;

#[derive(Clone, Debug, Default)]
pub struct ROM<'a> {
    filename: Cow<'a, str>,
    size:     usize,
    pub buffer:   Vec<u8>,
}


impl<'a> ROM<'a> {

    pub fn new(filename: String) -> io::Result<ROM<'a>> {

        let rom: ROM = ROM {
            filename: Cow::Owned(filename.clone()),
            size: 0,
            ..Default::default()
        };
        match rom.read_from_file() {
            Ok(_v) => Ok(_v),
            Err(e) => Err(e),
        }
    }

    pub fn read_from_file(mut self) -> io::Result<ROM<'a>> {
        let metadata = (fs::metadata(self.filename.to_mut()))?;
        self.size = metadata.len() as usize;

        let mut f = File::open(self.filename.to_mut())?;
        let read_size = f.read_to_end(&mut self.buffer)?;
        println!("Read Cartridge {} bytes", read_size);

        Ok(self)
    }
    pub fn get_size(&self) -> usize {
        self.size
    }
    pub fn get_logo(&self) -> Vec<u8> {
        let logo = self.buffer[0x104..0x133].to_vec().clone();
        logo
    }
    pub fn get_cgb_flag(&self) -> u8 {
        let cgb = self.buffer[0x143];
        cgb
    }
    pub fn get_cartridge_type(&self) -> u8 {
        let t = self.buffer[0x147];
        t
    }
    pub fn get_cartridge_type_str(&self) -> &str {
        let t = self.buffer[0x147];
        // From https://gbdev.io/pandocs/The_Cartridge_Header.html#0147---cartridge-type
        match t {
            0x00 => "ROM Only",
            0x01 => "MBC1",
            0x02 => "MBC1+RAM",
            0x03 => "MBC1+RAM+BATTERY",
            0x05 => "MBC2",
            0x06 => "MBC2+BATTERY",
            0x08 => "ROM+RAM",
            0x09 => "ROM+RAM+BATTERY",
            0x0B => "MMM01",
            0x0C => "MMM01+RAM",
            0x0D => "MMM01+RAM+BATTERY",
            0x0F => "MBC3+TIMER+BATTERY",
            0x10 => "MBC3+TIMER+RAM+BATTERY",
            0x11 => "MBC3",
            0x12 => "MBC3+RAM",
            0x13 => "MBC3+RAM+BATTERY",
            0x19 => "MBC5",
            0x1A => "MBC5+RAM",
            0x1B => "MBC5+RAM+BATTERY",
            0x1C => "MBC5+RUMBLE",
            0x1D => "MBC5+RUMBLE+RAM",
            0x1E => "MBC5+RUMBLE+RAM+BATTERY",
            0x20 => "MBC6",
            0x22 => "MBC7+SENSOR+RUMBLE+RAM+BATTERY",
            0xFC => "POCKET CAMERA",
            0xFD => "BANDAI TAMA5",
            0xFE => "HuC3",
            0xFF => "HuC1+RAM+BATTERY",
            _ => "UNK"
        }
    }
    pub fn get_cartridge_size_kb(&self) -> u32 {
        let t = 32<<self.buffer[0x148];
        t
    }
    pub fn get_ram_size_kb(&self) -> u32 {
        let t = 32<<self.buffer[0x149];
        t
    }
    pub fn get_name(&self) -> String {
        String::from_utf8(self.buffer[0x0134..0x0143].to_vec()).unwrap()
    }
    pub fn get_destination_code(&self) -> String {
        let mut ret: String = String::from("Unknown");

        if self.buffer[0x014A]==0x00 {
            ret = format!("Japanese");
        } else if self.buffer[0x014A]==0x01 {
            ret = format!("Non-Japanese");
        }
        ret
    }
    pub fn validate_checkchum(&self) -> bool{
        let orig = self.buffer[0x14D];
        let mut new: u8 = 0x00;
        for i in self.buffer[0x134..0x14D].to_vec() {
            new = new.wrapping_sub(i).wrapping_sub(1);
        }
        println!("Read: {:02X}  Computed: {:02X}", orig, new);
        orig==new
    }

    pub fn print_infos(&self) {
        /* Print informations about the loaded ROM */
        println!("Checksum valid:\t {}", self.validate_checkchum());
        println!("ROM Size:\t {:?}",         self.get_size());
        println!("ROM Name:\t '{}'",         self.get_name());
        println!("RAM Size:\t {}kB",         self.get_ram_size_kb());
        println!("Logo:\t\t {:02X?}",        self.get_logo());
        println!("CGB Flag:\t {:02X}",       self.get_cgb_flag());
        println!("Cartridge Type:\t {:02X} {}", self.get_cartridge_type(), self.get_cartridge_type_str());
        println!("Cartridge Size:\t {}kB",   self.get_cartridge_size_kb());
        println!("Destination:\t {}",        self.get_destination_code());

    }
}
