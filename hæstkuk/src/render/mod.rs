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

    }
}
