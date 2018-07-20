// Graphical; renderer
#![allow(non_snake_case)]
use std::marker::PhantomData;
use std::process;
extern crate sdl2;

//use render::sdl2::pixels::Color;
use render::sdl2::event::Event;
use render::sdl2::keyboard::Keycode;
use render::sdl2::EventPump;

//use sdl2::video::Window;
//use sdl2::rect::Rect;
//use std::time::Duration;

const WINDOW_WIDTH : u32 = 160;
const WINDOW_HEIGHT : u32 = 144;


pub struct Render<'a> {
    sdl_context: &'a mut sdl2::Sdl,
    video_subsystem: sdl2::VideoSubsystem,
    window: sdl2::video::Window,
    event_pump: &'a mut sdl2::EventPump,
    phantom: PhantomData<&'a u8>,
}


impl<'a> Render<'a> {
    pub fn new() -> Render<'a> {
        let sdl_context : &'a = sdl2::init().unwrap();
        let video_subsystem  = sdl_context.video().unwrap();
        let window = video_subsystem.window("rust-sdl2 demo: No Renderer", WINDOW_WIDTH, WINDOW_HEIGHT)
            .position_centered()
            .build()
            .unwrap();

        let render = Render {
            sdl_context: &mut sdl_context,
            video_subsystem: video_subsystem,
            window: window,
            event_pump: &mut sdl_context.event_pump().unwrap(),
            phantom: PhantomData,
        };
        render
    }
    pub fn get_events(&mut self) {
        let mut keypress : bool = false;
        for event in self.event_pump.poll_iter() {
            match event {
                Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break
                },
                Event::KeyDown { repeat: false, .. } => {
                    println!("KeyDown {:}", keypress);
                    keypress = true;
                },
                _ => {}
            }
        }
    }
}


