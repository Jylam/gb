use std::env;
use std::process;
mod rom;

fn main() {
    println!("Hestkuk.");
    let args: Vec<String> = env::args().collect();

    let rom: rom::ROM;
    /* Read first argument and create a ROM from that */
    match rom::read_rom_from_file(&String::from(args[1].clone())) {
        Ok(_v) => rom = _v.clone(),
        Err(_e) => {
            println!("Error: {:?}", _e);
            process::exit(1)
        },
    }
    println!("ROM Size:\t {:?}", rom.get_size());

    process::exit(0)
}
