// Graphical; renderer
#![allow(non_snake_case)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

extern crate minifb;
use minifb::{Key, Window, WindowOptions};



use std::marker::PhantomData;
use std::process;

use std::time::Duration;

use lr35902::Cpu;

const SCALE : u32 = 3;


#[allow(dead_code)]
pub struct Render<'a> {
    window: Window,
    width: usize,
    height: usize,
    buffer: Vec<u32>,
    lcd_regs: Vec<u8>,
    phantom: PhantomData<&'a u8>,
}


impl<'a> Render<'a> {
    pub fn new() -> Render<'a> {
        let mut window = Window::new(
            "Test - ESC to exit",
            256,
            256,
            WindowOptions::default(),
            )
            .unwrap_or_else(|e| {
                panic!("{}", e);
            });
        window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));
        let render = Render {
            window: window,
            width: 256,
            height: 256,
            buffer: vec![0x00; 256*256],
            lcd_regs: vec![0x00; 0x15],
            phantom: PhantomData,
        };
        render

    }
    pub fn get_events(&mut self) -> bool {
        self.window.is_key_down(Key::Escape)
    }

    pub fn oam(&mut self, cpu: &mut Cpu<'a>) {
        let mut offset: u16 = 0xFE00;
        for _i in 0..=40 {
            let x = cpu.readMem8(offset);
            let y = cpu.readMem8(offset+1);
            let pattern_number = cpu.readMem8(offset+2);
            let flags = cpu.readMem8(offset+3);
            if x!=0 {
                println!("X: {:02X}", x);
                println!("Y: {:02X}", y);
                println!("Pattern Number: {:02X}", pattern_number);
                println!("Flags: {:02X}", flags);
            }
            offset+=4;
        }
    }

    pub fn show_memory(&mut self, cpu: &mut Cpu<'a> ) {

        for i in 0..0xFFFF {
            let b = cpu.readMem8(i);
            self.buffer[i as usize] = (b as u32)+((b as u32)<<8)+((b as u32)<<16);
        }
        self.window.update_with_buffer(&self.buffer, self.width, self.height)
            .unwrap();
    }

    pub fn render_screen(&mut self, cpu: &mut Cpu<'a> ) {
    }

    pub fn display_tile_pattern_tables(&mut self, cpu: &mut Cpu<'a> ) {
        println!("TILES ------------------------------------------");
        let mut offset = 0;
        for j in (0x8000..0x8FFF).step_by(16) {
            for i in 0..8 {
                let a = cpu.readMem8(j + offset);
                let b = cpu.readMem8(j + offset+1);

                let p1 = (((a&0b10000000)>>6) | (b&0b10000000)>>7);
                let p2 = (((a&0b01000000)>>5) | (b&0b01000000)>>6);
                let p3 = (((a&0b00100000)>>4) | (b&0b00100000)>>5);
                let p4 = (((a&0b00010000)>>3) | (b&0b00010000)>>4);
                let p5 = (((a&0b00001000)>>2) | (b&0b00001000)>>3);
                let p6 = (((a&0b00000100)>>1) | (b&0b00000100)>>2);
                let p7 = (((a&0b00000010)>>0) | (b&0b00000010)>>1);
                let p8 = (((a&0b00000001)<<1) | (b&0b00000001)>>0);

                println!("{}{}{}{}{}{}{}{}", vtoc(p1), vtoc(p2), vtoc(p3), vtoc(p4), vtoc(p5), vtoc(p6), vtoc(p7), vtoc(p8));
                offset+=2;
            }
        println!("");
        }
    pub fn vtoc(v: u8)->char {
        match v {
            0 => {'#'}
            1 => {'+'}
            2 => {'.'}
            3 => {' '}
            _ => {println!("GOT {}", v); '?'}
        }
    }

    }
}
