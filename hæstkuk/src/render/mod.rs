// Graphical; renderer
#![allow(non_snake_case)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

extern crate minifb;
use minifb::{Key, Window, WindowOptions};
use std::thread::sleep;

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
            "Tiles - ESC to exit",
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


    pub fn put_pixel8(&mut self, x: usize, y: usize, c: u8) {
        self.buffer[x+y*self.width] = (((c as u32*64)<<16) |
                                      (( c as u32*64)<<8) |
                                      (( c as u32*64))) as u32;

    }

    pub fn get_tile_by_id(&mut self, cpu: &mut Cpu<'a>, id: u8) -> Vec<u8> {
        self.get_tile_at_addr(cpu, 0x8000+((id as usize)*16) as u16)
    }

    pub fn get_tile_at_addr(&mut self, cpu: &mut Cpu<'a>, addr: u16) -> Vec<u8> {
        let mut ret = vec![0; 8*8];
        let mut offset = addr;
        for i in 0..8 {
            let a = cpu.readMem8(offset);
            let b = cpu.readMem8(offset+1);

            let p1 = ((a&0b10000000)>>6) | (b&0b10000000)>>7;
            let p2 = ((a&0b01000000)>>5) | (b&0b01000000)>>6;
            let p3 = ((a&0b00100000)>>4) | (b&0b00100000)>>5;
            let p4 = ((a&0b00010000)>>3) | (b&0b00010000)>>4;
            let p5 = ((a&0b00001000)>>2) | (b&0b00001000)>>3;
            let p6 = ((a&0b00000100)>>1) | (b&0b00000100)>>2;
            let p7 = ((a&0b00000010)>>0) | (b&0b00000010)>>1;
            let p8 = ((a&0b00000001)<<1) | (b&0b00000001)>>0;

            offset+=2;

            ret[0+i*8] = p1;
            ret[1+i*8] = p2;
            ret[2+i*8] = p3;
            ret[3+i*8] = p4;
            ret[4+i*8] = p5;
            ret[5+i*8] = p6;
            ret[6+i*8] = p7;
            ret[7+i*8] = p8;
        }

        ret
    }

    pub fn display_tile(&mut self, x: usize, y: usize, buf: Vec<u8>) {

        for ty in 0..8 {
            for tx in 0..8 {
                self.put_pixel8(x+tx, y+ty, buf[tx+ty*8]);
            }
        }
    }

    pub fn display_tile_pattern_tables(&mut self, cpu: &mut Cpu<'a> ) {
        let mut x = 0;
        let mut y = 0;

        for j in (0x8000..0x8FFF).step_by(16) {
            let tile = self.get_tile_at_addr(cpu, j);
            self.display_tile(x, y, tile);
            x = x+8;
            if x > 200 {
                x = 0;
                y = y+8;
            }
        }

        self.window.update_with_buffer(&self.buffer, self.width, self.height)
            .unwrap();
        //sleep(Duration::from_secs(5));
    }

    pub fn display_background_map(&mut self, cpu: &mut Cpu<'a> ) {
        let mut x = 0;
        let mut y = 0;

        for offset in 0x9800..0x9BFF {
            let id = cpu.readMem8(offset);
            let tile = self.get_tile_by_id(cpu, id);
            self.display_tile(x, y, tile);

            x+=8;
            if x>=255 {
                x = 0;
                y += 8;
            }
        }
        self.window.update_with_buffer(&self.buffer, self.width, self.height)
            .unwrap();
    }

    pub fn vtoc(v: u8)->char {
        match v {
            0 => {' '}
            1 => {'-'}
            2 => {'+'}
            3 => {'#'}
            _ => {println!("GOT {}", v); '?'}
        }
    }
}
