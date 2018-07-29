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
    phantom: PhantomData<&'a u8>,
}


impl<'a> Render<'a> {
    pub fn new() -> Render<'a> {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem  = sdl_context.video().unwrap();
        let window = video_subsystem.window("rust-sdl2 demo: No Renderer", WINDOW_WIDTH*SCALE, WINDOW_HEIGHT*SCALE)
            .position_centered()
            .build()
            .unwrap();

        let render = Render {
            sdl_context: sdl_context,
            video_subsystem: video_subsystem,
            window: window,
            buffer: vec![0x00; (BUF_WIDTH*BUF_HEIGHT) as usize],
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
            if y!=0 {
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
        let mut x = 0;
        let mut y = 0;
        for i in 0x9800..=0x9BFF {
            let v = cpu.readMem8(i as u16);
            print!("{:02X} ", v);
            x+=1;
            if x == 32 {
                println!(" BUF");
                x = 0;
                y+=1;
            }
        }
    }
}


