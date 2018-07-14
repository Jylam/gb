// ROM
use std::io;
use std::io::prelude::*;
use std::fs::File;
use std::fs;

#[derive(Debug)]
pub struct ROM {
    filename: String,
    size: usize,
}


pub fn read_ROM(filename: &String) -> io::Result<ROM> {
    let mut rom = ROM{
        filename: String::from(filename.clone()),
        size: 0,
    };
    rom.read_from_file();
    Ok(rom)
}

impl ROM {
    pub fn read_from_file(&mut self) -> io::Result<()> {
        let metadata = try!(fs::metadata(&self.filename));
        self.size = metadata.len() as usize;
        println!("Reading {}, {} bytes", &self.filename, self.size);

        let mut buffer = vec![0; self.size];
        let mut f = File::open(&self.filename)?;
        f.read_to_end(&mut buffer)?;



        Ok(())
    }
}
