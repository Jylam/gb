use std::env;
use std::process;
mod rom;

fn main() {
    println!("Hestkuk.");

    /* Parse arguments */
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
            println!("Usage:\n\t{} <rom.gb>", args[0]);
            process::exit(2);
    }

    /* Read first argument and create a ROM from that */
    let rom: rom::ROM;
    match rom::read_rom_from_file(&String::from(args[1].clone())) {
        Ok(_v) => rom = _v.clone(),
        Err(_e) => {
            println!("Error: {:?}", _e);
            process::exit(1)
        },
    }

    // FIXME rom.validate_checkchum();
    println!("ROM Size:\t {:?}",         rom.get_size());
    println!("ROM Name:\t '{}'",           rom.get_name());
    println!("Logo:\t\t {:02X?}",        rom.get_logo());
    println!("CGB Flag:\t {:02X}",       rom.get_cgb_flag());
    println!("Cartridge Type:\t {:02X}", rom.get_cartridge_type());
    println!("Cartridge Size:\t {}kB",   rom.get_cartridge_size_kb());
    println!("Destination:\t {}",   rom.get_destination_code());

    process::exit(0)
}
