#[macro_use]
extern crate log;
extern crate env_logger;
use std::io;
use std::env;
use std::process;

mod mem;
mod rom;
mod lr35902;
mod lcd;
mod render;

const VBLANK_FREQ_CYCLES : u32 = 17555;
const REFRESH_CYCLES : u32 = 1000;

fn main() {
    env_logger::init();

    let lcd: lcd::LCD;
    let rom: rom::ROM;
    let mut cpu: lr35902::Cpu;
    let mem: mem::Mem;
    let mut render: render::Render;

    println!("Hestkuk.");

    /* Parse arguments */
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
            println!("Usage:\n\t{} <rom.gb>", args[0]);
            process::exit(2);
    }

    let v: io::Result<rom::ROM> = rom::ROM::new(String::from(args[1].clone()));
    /* Read first argument and create a ROM from that */
    match v {
        Ok(_v) => rom = _v.clone(),
        Err(_e) => {
            println!("Error: {:?}", _e);
            process::exit(1)
        },
    }
    rom.print_infos();

    lcd = lcd::LCD::new();
    mem = mem::Mem::new(rom, lcd);
    cpu = lr35902::Cpu::new(mem);

    render = render::Render::new(); // Open SDL window

    let mut y: u8 = 0;
    let mut refresh_count: u32 = 1;
    let mut vblank_counter: u32 = 1;

    cpu.reset();

    'running : loop {
        vblank_counter-=1;
        if vblank_counter == 0 {
            vblank_counter = VBLANK_FREQ_CYCLES;
            if cpu.interrupts_enabled() {
                println!("VBLANK !!!");
                cpu.irq_vblank();
            } else {
            }
        }


        refresh_count-=1;
        if refresh_count == 0 {
            render.get_events();
            //render.show_memory(&mut cpu);
            //render.oam(&mut cpu);
            render.display_tile_pattern_tables(&mut cpu);
            //render.render_screen(&mut cpu);
            refresh_count = REFRESH_CYCLES;
            cpu.writeMem8(0xFF44, y);
            y=y.wrapping_add(1);
        }
        cpu.step();
    }
}
