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
mod joypad;
mod timer;

extern crate minifb;

const VBLANK_FREQ_CYCLES : u64 = 17555;
const CPU_MHZ: u64 = 4_194_304;
// 4.194304 MHz
fn main() {
    env_logger::init();

    let lcd: lcd::LCD;
    let timer: timer::Timer;
    let joypad: joypad::Joypad;
    let rom: rom::ROM;
    let mut cpu: lr35902::Cpu;
    let mem: mem::Mem;
    let mut render: render::Render;

    println!("Hæstkuk.");

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

    timer = timer::Timer::new(CPU_MHZ);
    lcd = lcd::LCD::new();
    joypad = joypad::Joypad::new();
    mem = mem::Mem::new(rom, lcd, joypad, timer);
    cpu = lr35902::Cpu::new(mem);

    render = render::Render::new();

    let mut vblank_counter: u64 = 1;
    let mut total_cycles: u64 = 0;

    cpu.reset();

    loop {
        vblank_counter-=1;
        if vblank_counter == 0 {
            vblank_counter = VBLANK_FREQ_CYCLES;
            render.display_BG_map(&mut cpu);
            render.display_tile_pattern_tables (&mut cpu);
            cpu.mem.lcd.update();
        }

        let cur_cycles = cpu.step() as u64;
        total_cycles += cur_cycles;

        cpu.mem.timer.update(cur_cycles);

        if render.get_events(&mut cpu) {
            println!("EXIT");
            break;
        }

    }
}
