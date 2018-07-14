// ROM
use std::io;
use std::io::prelude::*;
use std::fs::File;
use std::fs;

#[derive(Clone, Debug)]
pub struct ROM {
    filename: String,
    size: usize,
}


pub fn read_rom_from_file(filename: &String) -> io::Result<ROM> {
    let mut rom = ROM{
        filename: String::from(filename.clone()),
        size: 0,
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
        let mut buffer = vec![0; self.size];
        let mut f = File::open(&self.filename)?;
        f.read_to_end(&mut buffer)?;
        Ok(())
    }
    pub fn get_size(&self) -> usize {
        self.size
    }
}
