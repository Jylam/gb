// Graphical; renderer
#![allow(non_snake_case)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

extern crate minifb;
use minifb::{Key, Window, WindowOptions};
use std::thread::sleep;

use std::marker::PhantomData;
use std::process;

use std::time::Duration;

use lr35902::Cpu;

const SCALE : u32 = 3;


#[allow(dead_code)]
pub struct Render<'a> {
    render_window: Window,
    bg_window: Window,
    tiles_window: Window,
    width: usize,
    height: usize,
    buffer_render: Vec<u32>,
    buffer_bg: Vec<u32>,
    buffer_tiles: Vec<u32>,
    phantom: PhantomData<&'a u8>,
}


impl<'a> Render<'a> {
    pub fn new() -> Render<'a> {
        let tiles_window = Window::new(
            "Tiles - ESC to exit",
            256,
            256,
            WindowOptions::default(),
            )
            .unwrap_or_else(|e| {
                panic!("{}", e);
            });
        let render_window = Window::new(
            "Render - ESC to exit",
            256,
            256,
            WindowOptions::default(),
            )
            .unwrap_or_else(|e| {
                panic!("{}", e);
            });
        let bg_window = Window::new(
            "BGMap - ESC to exit",
            256,
            256,
            WindowOptions::default(),
            )
            .unwrap_or_else(|e| {
                panic!("{}", e);
            });
        //window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));
        let render = Render {
            render_window: render_window,
            bg_window: bg_window,
            tiles_window: tiles_window,
            width: 256,
            height: 256,
            buffer_render:    vec![0x00; 256*256],
            buffer_bg:    vec![0x00; 256*256],
            buffer_tiles: vec![0x00; 256*256],
            phantom: PhantomData,
        };
        render

    }

    // Handle key pressed, returns true on quit
    pub fn get_events(&mut self, cpu: &mut Cpu<'a>) -> bool {
        cpu.mem.joypad.set_a(self.render_window.is_key_down(Key::A));
        cpu.mem.joypad.set_b(self.render_window.is_key_down(Key::B));
        cpu.mem.joypad.set_select(self.render_window.is_key_down(Key::Space));
        cpu.mem.joypad.set_start(self.render_window.is_key_down(Key::Enter));

        cpu.mem.joypad.set_up(self.render_window.is_key_down(Key::Up));
        cpu.mem.joypad.set_down(self.render_window.is_key_down(Key::Down));
        cpu.mem.joypad.set_left(self.render_window.is_key_down(Key::Left));
        cpu.mem.joypad.set_right(self.render_window.is_key_down(Key::Right));

        self.bg_window.is_key_down(Key::Escape) ||
            self.tiles_window.is_key_down(Key::Escape) ||
            self.render_window.is_key_down(Key::Escape)

    }

    pub fn oam(&mut self, cpu: &mut Cpu<'a>) {
        let mut offset: u16 = 0xFE00;
        for i in 0..=40 {
            let x = cpu.readMem8(offset);
            let y = cpu.readMem8(offset+1);
            let pattern_number = cpu.readMem8(offset+2);
            let flags = cpu.readMem8(offset+3);
            if x!=0 {
                println!("Sprite {}", i);
                println!("X: {:02X} {}", x, x);
                println!("Y: {:02X} {}", y, y);
                println!("Pattern Number: {:02X}", pattern_number);
                println!("Flags: {:02X}", flags);
            }
            offset+=4;
        }
    }



    pub fn put_pixel24(&mut self, buf: &mut Vec<u32>, x: usize, y: usize, r: u8, g: u8, b: u8) {
        if x+y*self.width > 65535 {
            return;
        };


        buf[x+y*self.width] = (((r as u32)<<16) |
                               (( g as u32)<<8) |
                               (( b as u32))) as u32;

    }
    pub fn put_pixel8(&mut self, buf: &mut Vec<u32>, x: usize, y: usize, c: u8) {
        if x+y*self.width > 65535 {
            //    println!("put_pixel8 error");
            return;
        };

        let v;
        if c == 0x00 {v=0x00;}
        else if c == 0x01 {v=0x55;}
        else if c == 0x02 {v=0xAA;}
        else {v=0xFF;}
        self.put_pixel24(buf, x, y, v, v, v);
    }

    pub fn get_tile_by_id(&mut self, cpu: &mut Cpu<'a>, id: u8, is_sprite: bool) -> Vec<u8> {

        let addr = cpu.mem.lcd.get_tile_addr(id, is_sprite);

        self.get_tile_at_addr(cpu, addr, is_sprite)
    }

    pub fn get_tile_at_addr(&mut self, cpu: &mut Cpu<'a>, addr: u16, is_sprite: bool) -> Vec<u8> {

        let colors = cpu.mem.lcd.get_bw_palette();
        let mut ret = vec![0; 8*8];
        let mut offset = addr;
        for i in 0..8 {
            let a = cpu.readMem8(offset);
            let b = cpu.readMem8(offset+1);

            let p1 = ((a&0b10000000)>>6) | (b&0b10000000)>>7;
            let p2 = ((a&0b01000000)>>5) | (b&0b01000000)>>6;
            let p3 = ((a&0b00100000)>>4) | (b&0b00100000)>>5;
            let p4 = ((a&0b00010000)>>3) | (b&0b00010000)>>4;
            let p5 = ((a&0b00001000)>>2) | (b&0b00001000)>>3;
            let p6 = ((a&0b00000100)>>1) | (b&0b00000100)>>2;
            let p7 = ((a&0b00000010)>>0) | (b&0b00000010)>>1;
            let p8 = ((a&0b00000001)<<1) | (b&0b00000001)>>0;

            offset+=2;

            // Put 0xFF if the color is transparent
            ret[0+i*8] = if is_sprite && p1==0 {0xFF}else{colors[p1 as usize]};
            ret[1+i*8] = if is_sprite && p2==0 {0xFF}else{colors[p2 as usize]};
            ret[2+i*8] = if is_sprite && p3==0 {0xFF}else{colors[p3 as usize]};
            ret[3+i*8] = if is_sprite && p4==0 {0xFF}else{colors[p4 as usize]};
            ret[4+i*8] = if is_sprite && p5==0 {0xFF}else{colors[p5 as usize]};
            ret[5+i*8] = if is_sprite && p6==0 {0xFF}else{colors[p6 as usize]};
            ret[6+i*8] = if is_sprite && p7==0 {0xFF}else{colors[p7 as usize]};
            ret[7+i*8] = if is_sprite && p8==0 {0xFF}else{colors[p8 as usize]};
        }

        ret
    }

    pub fn display_tile(&mut self, buf: &mut Vec<u32>, x: usize, y: usize, buft: Vec<u8>) {

        for ty in 0..8 {
            for tx in 0..8 {
                self.put_pixel8(buf, x+tx, y+ty, buft[tx+ty*8]);
            }
        }
    }

    pub fn display_tile_pattern_tables(&mut self, cpu: &mut Cpu<'a> ) {
        let mut x = 0;
        let mut y = 0;

        let mut buffer = vec![0x00; self.width*self.height];
        for j in (0x8000..0x97FF).step_by(16) {
            let tile = self.get_tile_at_addr(cpu, j, false);
            self.display_tile(&mut buffer, x, y, tile);
            x = x+8;
            if x > 200 {
                x = 0;
                y = y+8;
            }
        }

        self.tiles_window.update_with_buffer(&mut buffer, self.width, self.height)
            .unwrap();
    }


    pub fn get_bg_pixel_at(&mut self, cpu: &mut Cpu<'a>, x: usize, y: usize) -> u8 {
        let bgmap = 0x9800; // End at 0x9BFF, 32x32 of 8x8 tiles

        // X and Y offset in the 32x32 BGMAP
        let xoff = (x / 8)%160;
        let yoff = (y / 8)%144;
        // Pixel in the tile
        let xrest = (x%256)-(xoff*8);
        let yrest = (y%256)-(yoff*8);
        // Offset in the BGMAP
        let bgoff = xoff+yoff*32;
        // Get ID
        let id = cpu.readMem8(bgmap+bgoff as u16);
        // Get Tile data
        let tile = self.get_tile_by_id(cpu, id, false);
        // Get Pixel value
        //println!("X {} Y {}   XOFF {} YOFF {} XREST {}, YREST {}", x,y, xoff, yoff, xrest, yrest);
        tile[(xrest)+(yrest)*8]
    }

    pub fn gen_BG_map_pixel(&mut self, cpu: &mut Cpu<'a>, buffer: &mut Vec<u32>) {
        let SCY  = cpu.mem.read8(0xFF42) as usize;
        let SCX  = cpu.mem.read8(0xFF43) as usize;

        for y in 0..144 {
            for x in 0..160 {
                let c = self.get_bg_pixel_at(cpu, x + SCX, y + SCY);
                self.put_pixel8(buffer, x, y, c);
            }
        }
    }

    pub fn gen_BG_map(&mut self, cpu: &mut Cpu<'a>, buffer: &mut Vec<u32>) {
        let mut x = 0;
        let mut y = 0;

        for offset in 0x9800..=0x9BFF {
            let id = cpu.readMem8(offset);
            let tile = self.get_tile_by_id(cpu, id, false);
            self.display_tile(buffer, x, y, tile);
            x+=8;
            if x>=255 {
                x = 0;
                y += 8;
            }
        }
    }

    pub fn display_BG_map(&mut self, cpu: &mut Cpu<'a> ) {
        let mut buffer = vec![0x00; self.width*self.height];
        self.gen_BG_map(cpu, &mut buffer);
        self.display_scroll(cpu, &mut buffer);
        self.bg_window.update_with_buffer(&mut buffer, self.width, self.height)
            .unwrap();
    }

    pub fn display_WIN_map(&mut self, cpu: &mut Cpu<'a> ) {
        let mut x = 0;
        let mut y = 0;

        let mut buffer = vec![0x00; self.width*self.height];
        for offset in 0x9C00..0x9FFF {
            let id = cpu.readMem8(offset);
            let tile = self.get_tile_by_id(cpu, id, false);
            self.display_tile(&mut buffer, x, y, tile);

            x+=8;
            if x>=255 {
                x = 0;
                y += 8;
            }
        }
        self.bg_window.update_with_buffer(&mut buffer, self.width, self.height)
            .unwrap();
    }
    pub fn display_sprite(&mut self, buf: &mut Vec<u32>, x: usize, y: usize, buft: Vec<u8>, flags: u8) {
        let xflip = flags&0b0010_0000 != 0;
        let yflip = flags&0b0100_0000 != 0;

        let sx: i32;
        let sy: i32;
        let stepx: i32;
        let stepy: i32;

        if !xflip { sx = 0; stepx = 1; } else { sx = 7; stepx = -1; }
        if !yflip { sy = 0; stepy = 1; } else { sy = 7; stepy = -1; }

        let mut ty = sy;
        let mut tx;
        let mut iy = 0;
        let mut ix;

        while iy<8 {
            tx = sx;
            ix = 0;
            while ix<8 {
                // If the color is 0xFF, it is transparent
                if buft[(tx+ty*8) as usize] != 0xFF {
                self.put_pixel8(buf, (x+ix-8) as usize, (y+iy-16) as usize, buft[(tx+ty*8) as usize]);
                }
                tx+=stepx;
                ix+=1;
            }
            ty+=stepy;
            iy+=1;
        }
    }

    pub fn render_screen(&mut self, cpu: &mut Cpu<'a> ) {
        let lcdc = cpu.mem.read8(0xFF40);
        let mut buffer = vec![0x00; self.width*self.height];
//        self.gen_BG_map(cpu, &mut buffer);
        self.gen_BG_map_pixel(cpu, &mut buffer);


        // OAM
        let SCY = cpu.mem.read8(0xFF42) as usize;
        let SCX = cpu.mem.read8(0xFF43) as usize;
        let mut offset: u16 = 0xFE00;
        for i in 0..=40 {
            let y = cpu.readMem8(offset) as usize;
            let x = cpu.readMem8(offset+1) as usize;
            let pattern_number = cpu.readMem8(offset+2);
            let flags = cpu.readMem8(offset+3);
            if x!=0 {
                let tile = self.get_tile_by_id(cpu, pattern_number, true);
                self.display_sprite(&mut buffer, x+SCX, y+SCY, tile, flags);
                if (lcdc&0b0000_0100)!=0 {
                    let tile = self.get_tile_by_id(cpu, pattern_number+1, true);
                    self.display_sprite(&mut buffer, x+SCX, y+SCY+8, tile, flags);
                }
            }
            offset+=4;
        }
        self.render_window.update_with_buffer(&mut buffer, self.width, self.height)
            .unwrap();
    }

    pub fn display_scroll(&mut self, cpu: &mut Cpu<'a>, buf: &mut Vec<u32>) {
        let SCY  = cpu.mem.read8(0xFF42) as usize;
        let SCX  = cpu.mem.read8(0xFF43) as usize;
        let cury = cpu.mem.read8(0xFF44) as usize;

        for y in SCY..SCY+144 {
            self.put_pixel24(buf, SCX, y, 255, 0, 0);
        }
        for y in SCY..SCY+144 {
            self.put_pixel24(buf, SCX+160, y , 255, 0, 0);
        }
        for x in SCX..SCX+160 {
            self.put_pixel24(buf, x, SCY, 255, 0, 0);
        }
        for x in SCX..SCX+160 {
            self.put_pixel24(buf, x, SCY+144, 255, 0, 0);
        }
        for x in 0..255 {
            self.put_pixel24(buf, x, cury, 0, 0, 255);
        }
    }
}
