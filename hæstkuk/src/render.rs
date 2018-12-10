// Graphical; renderer
#![allow(non_snake_case)]
#![allow(unused_imports)]
use std::marker::PhantomData;
use std::process;
extern crate sdl2;

use render::sdl2::pixels::Color;
use render::sdl2::event::Event;
use render::sdl2::keyboard::Keycode;

use render::sdl2::video::Window;
use render::sdl2::rect::Rect;
use std::time::Duration;

use lr35902::Cpu;

const WINDOW_WIDTH : u32 = 256;//160;
const WINDOW_HEIGHT : u32 = 256;//144;
const SCALE : u32 = 3;

const BUF_WIDTH: u32 = 256;
const BUF_HEIGHT: u32 = 256;

#[allow(dead_code)]
pub struct Render<'a> {
    sdl_context: sdl2::Sdl,
    video_subsystem: sdl2::VideoSubsystem,
    window: sdl2::video::Window,
    buffer: Vec<u8>,
    lcd_regs: Vec<u8>,
    phantom: PhantomData<&'a u8>,
}


impl<'a> Render<'a> {
    pub fn new() -> Render<'a> {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem  = sdl_context.video().unwrap();
        let window = video_subsystem.window("Hæstkuk dev", WINDOW_WIDTH*SCALE, WINDOW_HEIGHT*SCALE)
            .position_centered()
            .build()
            .unwrap();

        let render = Render {
            sdl_context: sdl_context,
            video_subsystem: video_subsystem,
            window: window,
            buffer: vec![0x00; (BUF_WIDTH*BUF_HEIGHT) as usize],
            lcd_regs: vec![0x00; 0x15],
            phantom: PhantomData,
        };
        render
    }
    pub fn get_events(&mut self) {
        let mut keypress : bool = false;
        for event in self.sdl_context.event_pump().unwrap().poll_iter() {
            match event {
                Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    process::exit(3);
                },
                Event::KeyDown { repeat: false, .. } => {
                    println!("KeyDown {:}, event {:?}", keypress, event);
                    keypress = true;
                },
                _ => {}
            }
        }
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
        let pump = &self.sdl_context.event_pump().unwrap();
        let mut surface = self.window.surface(pump).unwrap();

        let mut offset: u32 = 0x0000;
        for y in 0 .. (WINDOW_HEIGHT) {
            for x in 0 .. (WINDOW_WIDTH) {
                if offset <= 0xFFFF {
                    let r = cpu.readMem8(offset as u16);
                    let color = Color::RGB(r, r, r);
                    surface.fill_rect(Rect::new((x * SCALE) as i32, (y * SCALE) as i32, SCALE, SCALE), color).unwrap();
                } else {
                    let color = Color::RGB(255, 0, 0);
                    surface.fill_rect(Rect::new((x *SCALE) as i32, (y * SCALE) as i32, SCALE, SCALE), color).unwrap();
                }
                offset = offset+1;
            }
        }
        surface.finish().unwrap();
    }

    pub fn render_screen(&mut self, cpu: &mut Cpu<'a> ) {
<<<<<<< HEAD
        let bg_display = cpu.mem.lcd.background_display();
        println!("BG Display: {}", bg_display);
=======
>>>>>>> parent of 99f7223... LCD registers read/write
    }

    pub fn display_tile_pattern_tables(&mut self, cpu: &mut Cpu<'a> ) {
        let mut x = 0;
        let mut y = 0;

        let pump = &self.sdl_context.event_pump().unwrap();
        let mut surface = self.window.surface(pump).unwrap();

        for mut i in 0x8000..=0x97FF {

            let b1 = cpu.readMem8(i as u16);
            let b2 = cpu.readMem8(i+1 as u16);
            i+=1;
            for _pixel in 0..=7 {
                let _v1 = b1.wrapping_shr(7-_pixel)&0x01;
                let _v2 = b2.wrapping_shr(7-_pixel)&0x01;
                let _value = (_v1<<1)|_v2;

                let color8 = ((_value)*64) as u8;
                let color = Color::RGB(color8, color8, color8);
                //println!("{:}x{:}->{:}", (x+_pixel) * SCALE , (y*SCALE), color8);
                surface.fill_rect(Rect::new(((x+_pixel) * SCALE) as i32, (y * SCALE) as i32, SCALE, SCALE), color).unwrap();
            }
            y+=1;
            if y>=WINDOW_HEIGHT {
                y=0;
                x+=8;
            }



        }
        surface.finish().unwrap();

    }
    pub fn write8(&mut self, addr: u16, v: u8)  {
        debug!("LCD Write8 {:02X} at {:04X}", v, addr);
        match addr {
            0..=15 => {self.regs[(addr) as usize] = v;}
            _ => {error!("LCD Write8 range error")}
        }
    }

    pub fn read8(&self, addr: u16) -> u8 {
        match addr {
            _ => {debug!("LCD read8 at {:04X}", addr); self.regs[addr as usize]}
        }
    }

    pub fn update(&mut self) {
        self.regs[(0x04)] = self.regs[(0x04)].wrapping_add(1);
    }


    pub fn background_display(&self) -> bool {
        (self.regs[0x00]&1)==1
    }

    /*
     FF40 -- LCDCONT [RW] LCD Control              | when set to 1 | when set to 0
     Bit7  LCD operation                           | ON            | OFF
     Bit6  Window Tile Table address               | 9C00-9FFF     | 9800-9BFF
     Bit5  Window display                          | ON            | OFF
     Bit4  Tile Pattern Table address              | 8000-8FFF     | 8800-97FF
     Bit3  Background Tile Table address           | 9C00-9FFF     | 9800-9BFF
     Bit2  Sprite size                             | 8x16          | 8x8
     Bit1  Color #0 transparency in the window     | SOLID         | TRANSPARENT
     Bit0  Background display                      | ON            | OFF
     */

}
