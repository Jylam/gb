// ROM
use std::io;
use std::io::prelude::*;
use std::fs::File;

pub struct ROM {
    pub filename: String,
}

impl ROM {
    pub fn readfile(self) {
        println!("Reading {}", self.filename);
    }
}
