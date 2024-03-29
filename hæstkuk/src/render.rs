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
        let operation = cpu.mem.lcd.operation();
        let window_tile_table_address = cpu.mem.lcd.window_tile_table_address();
        let window_display = cpu.mem.lcd.window_display();
        let tile_pattern_table_address = cpu.mem.lcd.tile_pattern_table_address();
        let background_tile_table_address = cpu.mem.lcd.background_tile_table_address() as u32;
        let sprite_size = cpu.mem.lcd.sprite_size();
        let color_0_transparency = cpu.mem.lcd.color_0_transparency();
        let bg_display = cpu.mem.lcd.background_display();
        println!("LCD Operation: {}", operation);
        println!("window_tile_table_address : {:04X}", window_tile_table_address);
        println!("Window Display: {}", window_display);
        println!("Tile Pattern Table Address : {:04X}", tile_pattern_table_address);
        println!("Background Tile Table Address : {:04X}", background_tile_table_address);
        println!("Sprite Size Double: {}", sprite_size);
        println!("Color 0 Transparency: {}", color_0_transparency);
        println!("BG Display: {}", bg_display);

        println!("Scroll : {}x{}", cpu.mem.lcd.scroll_x(), cpu.mem.lcd.scroll_y());
        println!("Curline : {}", cpu.mem.lcd.curline());
        println!("Win Position : {}x{}", cpu.mem.lcd.win_pos_x(), cpu.mem.lcd.win_pos_y());

        let pump = &self.sdl_context.event_pump().unwrap();
        let mut surface = self.window.surface(pump).unwrap();

        /* Background and Window patterns are drawn line by line */
        /* Background */
        if bg_display {
            for _y in 0..32 {
                for _x in 0..32 {
                    for _line in 0..8 {
                        for _pixel in 0..8 {
                            let tile_address = background_tile_table_address + _x + (_y*32);
                            let tile_num     = cpu.readMem8((tile_address) as u16) as u16;

                            let pixel_address = (tile_pattern_table_address+(tile_num*16)+(_line*2)) as u16;
                            let b1 = cpu.readMem8(pixel_address);
                            let b2 = cpu.readMem8(pixel_address+1);

                            let _v1 = b1.wrapping_shr((7u16-_pixel) as u32)&0x01;
                            let _v2 = b2.wrapping_shr((7u16-_pixel) as u32)&0x01;
                            let _value = (_v1<<1)|_v2;

                            let color8 = ((_value)*64) as u8;
                            let color = Color::RGB(color8, color8, color8);

                            surface.fill_rect(
                                Rect::new(
                                    ((((_x * 8) as u32)+ (_pixel as u32)) * SCALE) as i32,
                                    ((((_y * 8) as u32)+ (_line as u32 )) * SCALE) as i32,
                                    SCALE, SCALE), color).unwrap();
                        }
                    }
                }
            }
        }
        if window_display {
            for _y in 0..32 {
                for _x in 0..32 {
                    for _line in 0..8 {
                        for _pixel in 0..8 {
                            let tile_address = window_tile_table_address + _x + (_y*32);
                            let tile_num     = cpu.readMem8((tile_address) as u16) as u16;

                            let pixel_address = (tile_pattern_table_address+(tile_num*16)+(_line*2)) as u16;
                            let b1 = cpu.readMem8(pixel_address);
                            let b2 = cpu.readMem8(pixel_address+1);

                            let _v1 = b1.wrapping_shr((7u16-_pixel) as u32)&0x01;
                            let _v2 = b2.wrapping_shr((7u16-_pixel) as u32)&0x01;
                            let _value = (_v1<<1)|_v2;

                            let color8 = ((_value)*64) as u8;
                            let color = Color::RGB(color8, color8, color8);

                            surface.fill_rect(
                                Rect::new(
                                    ((((_x * 8) as u32)+ (_pixel as u32)) * SCALE) as i32,
                                    ((((_y * 8) as u32)+ (_line as u32 )) * SCALE) as i32,
                                    SCALE, SCALE), color).unwrap();
                        }
                    }
                }
            }

        }
            surface.finish().unwrap();

        }

        pub fn display_tile_pattern_tables(&mut self, cpu: &mut Cpu<'a> ) {
            let mut x = 0;
            let mut y = 0;

            let pump = &self.sdl_context.event_pump().unwrap();
            let mut surface = self.window.surface(pump).unwrap();

            for i in (0x8000..=0x97FF).step_by(2) {

                let b1 = cpu.readMem8(i as u16);
                let b2 = cpu.readMem8(i+1 as u16);
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

    }
