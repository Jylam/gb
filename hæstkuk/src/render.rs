// Graphical; renderer
#![allow(non_snake_case)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]
use std::marker::PhantomData;
use std::process;

use std::time::Duration;

use lr35902::Cpu;

const WINDOW_WIDTH : u32 = 256;//160;
const WINDOW_HEIGHT : u32 = 256;//144;
const SCALE : u32 = 3;

const BUF_WIDTH: u32 = 256;
const BUF_HEIGHT: u32 = 256;

#[allow(dead_code)]
pub struct Render<'a> {
    buffer: Vec<u8>,
    lcd_regs: Vec<u8>,
    phantom: PhantomData<&'a u8>,
}


impl<'a> Render<'a> {
    pub fn new() -> Render<'a> {
        let render = Render {
            buffer: vec![0x00; (BUF_WIDTH*BUF_HEIGHT) as usize],
            lcd_regs: vec![0x00; 0x15],
            phantom: PhantomData,
        };
        render
    }
    pub fn get_events(&mut self) {
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
    }

    pub fn render_screen(&mut self, cpu: &mut Cpu<'a> ) {
    }

    pub fn display_tile_pattern_tables(&mut self, cpu: &mut Cpu<'a> ) {

    }
}
