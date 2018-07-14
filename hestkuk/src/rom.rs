// ROM
use std::io;
use std::io::prelude::*;
use std::fs::File;
use std::fs;

#[derive(Clone, Debug, Default)]
pub struct ROM {
    filename: String,
    size: usize,
    buffer: Vec<u8>,
}


pub fn read_rom_from_file(filename: &String) -> io::Result<ROM> {
    let mut rom = ROM{
        filename: String::from(filename.clone()),
        size: 0,
        ..Default::default()
    };
    match rom.read_from_file() {
        Ok(_v) => Ok(rom),
        Err(e) => Err(e),
    }
}

impl ROM {
    pub fn read_from_file(&mut self) -> io::Result<()> {
        let metadata = try!(fs::metadata(&self.filename));
        self.size = metadata.len() as usize;
        let mut f = File::open(&self.filename)?;
        let read_size = f.read_to_end(&mut self.buffer)?;
        println!("Read {} bytes", read_size);
        Ok(())
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
    pub fn get_cartridge_size_kb(&self) -> u32 {
        let t = 32<<self.buffer[0x148];
        t
    }
    pub fn get_name(&self) -> String {
        String::from_utf8(self.buffer[0x0134..0x0143].to_vec()).unwrap()
    }

}
