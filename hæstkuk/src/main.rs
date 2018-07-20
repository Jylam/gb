use std::io;
use std::env;
use std::process;



const WINDOW_WIDTH : u32 = 160;
const WINDOW_HEIGHT : u32 = 144;


mod mem;
mod rom;
mod lr35902;
mod lcd;
mod render;


fn main() {
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
    // Open SDL2 Window


    lcd = lcd::LCD::new();
    /* Create Memory Controller */
    mem = mem::Mem::new(rom, lcd);
    /* Create Sharp LR35902 CPU instance */
    cpu = lr35902::Cpu::new(mem);


    render = render::Render::new();



    cpu.reset();
    //cpu.print_status();
    'running : loop {

        render.get_events();

        cpu.step();
       //println!("{:02X}", cpu.readMem8(0xFFB6));
    }
}
