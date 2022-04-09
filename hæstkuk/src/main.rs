#[macro_use]
extern crate log;
extern crate env_logger;
use std::io;
use std::env;
use std::process;
use std::time::{SystemTime};
use std::thread;
mod mem;
mod rom;
mod lr35902;
mod lcd;
mod render;
mod joypad;
mod timer;

extern crate minifb;

const CPU_MHZ: u64 = 4_194_304;
const REFRESH_CYCLES : u64 = (CPU_MHZ  as f64 / 59.727500569606) as u64;

// 4.194304 MHz
fn main() {
    env_logger::init();

    let lcd: lcd::LCD;
    let timer: timer::Timer;
    let joypad: joypad::Joypad;
    let mut rom: rom::ROM;
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

    timer  = timer::Timer::new(CPU_MHZ);
    lcd    = lcd::LCD::new();
    joypad = joypad::Joypad::new();
    mem    = mem::Mem::new(rom, lcd, joypad, timer);
    cpu    = lr35902::Cpu::new(mem);
    render = render::Render::new();

    let mut refresh_counter: i64 = REFRESH_CYCLES as i64;

    cpu.reset();


    loop {
        let start = SystemTime::now();
        let cur_cycles = cpu.step() as u64;

        cpu.mem.timer.update(cur_cycles);
        cpu.mem.lcd.update(cur_cycles);

        if cpu.mem.lcd.need_new_line() {
            render.update_screen(&mut cpu);
        }
        if cpu.mem.lcd.need_render() {
            render.render_screen();
        }

        cpu.mem.joypad.update();

        refresh_counter-=cur_cycles as i64;
        if refresh_counter <= 0 {
            refresh_counter = REFRESH_CYCLES as i64;
            render.display_BG_map(&mut cpu);
            render.display_tile_pattern_tables (&mut cpu);
        }
        if render.get_events(&mut cpu) {
            println!("EXIT");
            break;
        }

        let end = SystemTime::now();
        let diff = end.duration_since(start).expect("Error").as_secs_f64();
//        thread::sleep(0.01674270629882807812-diff);
//        println!("{}", diff);

    }
}
