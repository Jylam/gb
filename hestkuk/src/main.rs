mod rom;
use rom::ROM;
/* struct Opcode {
    x: u8,
    y: u8,
    z: u8,
    p: u8,
    q: u8,
    name: &String,
    length: u8,
    cycles: u8,

}

static g_opcode : Opcode = Opcode {
        x: 0,
        y: 0,
        z: 0,
        p: 0,
        q: 0,
        name: &String::from("NOP"),
        length: 1,
        cycles: 4,
    };
*/
fn main() {
    println!("Hestkuk.");

    let rom = ROM{
        filename: String::from("test.gb"),
    };

    rom.readfile();


}
