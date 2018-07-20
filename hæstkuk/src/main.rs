use std::io;
use std::env;
use std::process;
mod mem;
mod rom;
mod lr35902;
mod lcd;


fn main() {
    let lcd: lcd::LCD;
    let rom: rom::ROM;
    let mut cpu: lr35902::Cpu;
    let mem: mem::Mem;

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

    /* Print informations about the loaded ROM */
    // FIXME rom.validate_checkchum();
    println!("ROM Size:\t {:?}",         rom.get_size());
    println!("ROM Name:\t '{}'",         rom.get_name());
    println!("RAM Size:\t {}kB",         rom.get_ram_size_kb());
    println!("Logo:\t\t {:02X?}",        rom.get_logo());
    println!("CGB Flag:\t {:02X}",       rom.get_cgb_flag());
    println!("Cartridge Type:\t {:02X}", rom.get_cartridge_type());
    println!("Cartridge Size:\t {}kB",   rom.get_cartridge_size_kb());
    println!("Destination:\t {}",        rom.get_destination_code());

    lcd = lcd::LCD::new();
    /* Create Memory Controller */
    mem = mem::Mem::new(rom, lcd);
    /* Create Sharp LR35902 CPU instance */
    cpu = lr35902::Cpu::new(mem);



    cpu.reset();
    //cpu.print_status();
    loop {
        cpu.step();
    }
}
