// Graphical; renderer
#![allow(non_snake_case)]
#![allow(unused_imports)]
#![allow(dead_code)]

extern crate minifb;
extern crate image;

use minifb::{Key, KeyRepeat, Window, WindowOptions, Scale, ScaleMode};
use std::thread::sleep;

use std::marker::PhantomData;
use std::process;

use std::time::Duration;

use lr35902::Cpu;

const SCALE : u32 = 3;

#[derive(Clone, Debug, Copy)]
pub enum PixelBuffer {
        Render,
        BG,
        Tiles,
}
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
    f1_pressed: bool,
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
        let mut render_window = Window::new(
            "Render - ESC to exit",
            256,
            256,
            WindowOptions {
                borderless: false,
                title: true,
                resize: false,
                scale: Scale::X4,
                scale_mode: ScaleMode::Stretch,
                topmost: false,
                transparency: false,
                none: false,
            },
            )
            .unwrap_or_else(|e| {
                panic!("{}", e);
            });
        render_window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));
        let bg_window = Window::new(
            "BGMap - ESC to exit",
            256,
            256,
            WindowOptions::default(),
            )
            .unwrap_or_else(|e| {
                panic!("{}", e);
            });
        let render = Render {
            render_window: render_window,
            bg_window: bg_window,
            tiles_window: tiles_window,
            width: 256,
            height: 256,
            buffer_render:    vec![0x00; 256*256],
            buffer_bg:    vec![0x00; 256*256],
            buffer_tiles: vec![0x00; 256*256],
            f1_pressed: false,
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

        if self.render_window.is_key_pressed(Key::F1, KeyRepeat::No) {
            if self.f1_pressed == false {
                println!("Saving image");
                //image::save_buffer("kuk.png", self.buffer_render.as_slice(), 256, 256, image::ColorType::Rgb8).unwrap();
                self.f1_pressed = true;
            }
        }
        if self.render_window.is_key_released(Key::F1) {
            self.f1_pressed = false;
        }

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

    pub fn put_pixel24(&mut self, buf: PixelBuffer, x: usize, y: usize, r: u8, g: u8, b: u8) {
        if x+y*self.width > 65535 {
            return;
        };

        let c = (((r as u32)<<16) |
                 (( g as u32)<<8) |
                 (( b as u32))) as u32;

        match buf {
            PixelBuffer::BG => { self.buffer_bg[x+y*self.width] = c },
            PixelBuffer::Render => { self.buffer_render[x+y*self.width] = c },
            PixelBuffer::Tiles => { self.buffer_tiles[x+y*self.width] = c },
        }
    }
    pub fn put_pixel8(&mut self, buf: PixelBuffer, x: usize, y: usize, c: u8) {
        let r;
        let g;
        let b;

        if      c == 0x00 {r=0x00; g=0x00; b=0x00;}
        else if c == 0x01 {r=0x55; g=0x55; b=0x55;}
        else if c == 0x02 {r=0xAA; g=0xAA; b=0xAA;}
        else if c == 0x03 {r=0xFF; g=0xFF; b=0xFF;}
        else if c == 0x55 {r=0xFF; g=0x00; b=0x00;}
        else if c == 0xAA {r=0x00; g=0xFF; b=0x00;}
        else if c == 0xBB {r=0x00; g=0x00; b=0xFF;}
        else {r=0xFF; g=0xFF; b=0xFF;}

        self.put_pixel24(buf, x, y, r, g, b);
    }

    pub fn get_tile_by_id(&mut self, cpu: &mut Cpu<'a>, id: u8, is_sprite: bool, palette: Vec<u8>) -> Vec<u8> {

        let addr = cpu.mem.lcd.get_tile_addr(id, is_sprite);

        self.get_tile_at_addr(cpu, addr, is_sprite, palette)
    }

    pub fn get_tile_at_addr(&mut self, cpu: &mut Cpu<'a>, addr: u16, is_sprite: bool, colors: Vec<u8>) -> Vec<u8> {

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

    pub fn display_tile(&mut self, buf: PixelBuffer, x: usize, y: usize, buft: Vec<u8>) {

        for ty in 0..8 {
            for tx in 0..8 {
                self.put_pixel8(buf, x+tx, y+ty, buft[tx+ty*8]);
            }
        }
    }

    pub fn display_tile_pattern_tables(&mut self, cpu: &mut Cpu<'a> ) {
        let mut x = 0;
        let mut y = 0;

        for j in (0x8000..0x97FF).step_by(16) {
            let palette = cpu.mem.lcd.get_bw_palette();
            let tile = self.get_tile_at_addr(cpu, j, false, palette);
            self.display_tile(PixelBuffer::Tiles, x, y, tile);
            x = x+8;
            if x > 200 {
                x = 0;
                y = y+8;
            }
        }

        self.tiles_window.update_with_buffer(&mut self.buffer_tiles, self.width, self.height)
            .unwrap();
    }


    pub fn get_bg_pixel_at(&mut self, cpu: &mut Cpu<'a>, x: usize, y: usize) -> u8 {
        let bgmap = 0x9800; // End at 0x9BFF, 32x32 of 8x8 tiles

        // X and Y offset in the 32x32 BGMAP
        let xoff = (x / 8)%32;
        let yoff = (y / 8)%32;
        // Pixel in the tile
        let xrest = ((x)-(xoff*8))%256;
        let yrest = ((y)-(yoff*8))%256;
        // Offset in the BGMAP
        let bgoff = xoff+yoff*32;
        // Tile ID
        let id = cpu.readMem8(bgmap+bgoff as u16);
        // Tile Pixels
        let palette = cpu.mem.lcd.get_bw_palette();
        let tile = self.get_tile_by_id(cpu, id, false, palette);
        // Get Pixel value
        tile[xrest+yrest*8]
    }

    pub fn gen_BG_map_line(&mut self, cpu: &mut Cpu<'a>, buffer: PixelBuffer, line: usize) {
        if line>144 {
            return;
        }
        let SCY  = cpu.mem.lcd.get_scy() as usize;
        let SCX  = cpu.mem.lcd.get_scx() as usize;
        let lcdc = cpu.mem.read8(0xFF40);

        for x in 0..160 {
            if (lcdc & 0b0000_0001) == 0x01 {
                let c = self.get_bg_pixel_at(cpu, x + SCX, line + SCY);
                self.put_pixel8(buffer, x, line, c);
            } else {
                self.put_pixel8(buffer, x, line, 0x03);
            }
        }
    }

    pub fn gen_OBJ_map_line(&mut self, cpu: &mut Cpu<'a>, buffer: PixelBuffer, line: usize) {
        if line>144 {
            return;
        }
        let mut offset: u16 = 0xFE00;
        let lcdc = cpu.mem.read8(0xFF40);

        // OBJ Disabled
        if (lcdc&0b0000_0010) == 0 {
            self.put_pixel8(buffer, 0, line, 0xAA);
            return;
        }
        let h = if (lcdc&0b0000_0100)!=0 { 16 } else { 8 };

        // Loop through sprites
        'oamloop: for _i in 0..40 {
            // Sprite position
            let py = (cpu.readMem8(offset) as isize)-16;

            if py<0 {
                continue 'oamloop;
            }
            // Sprite doesn't intersect the line
            if (py>line as isize) || ((py + (h-1)) < line as isize) {
                continue 'oamloop;
            }

            let pattern_number = cpu.readMem8(offset+2);
            let flags = cpu.readMem8(offset+3);
            let palette = cpu.mem.lcd.get_sprite_palette(((flags&0b0001_0000)>>4) as u16);
            let tile = self.get_tile_by_id(cpu, pattern_number, true, palette);
            let _xflip = flags&0b0010_0000 != 0;
            let _yflip = flags&0b0100_0000 != 0;
            let px = (cpu.readMem8(offset+1) as isize)-8;

            let y = line-py as usize;

            for x in 0..=7 {
                let c = tile[x+y*8];
                if c!=0xFF {
                    self.put_pixel8(buffer, x+px as usize, line, c);
                }

                if _i == 0 {
                    print!("{:02X}x{:02X} {:02X} ", x, y, c);
                }
            }

            if _i == 0 {
                println!("");
            }
            offset+=4;

        }
    }

    pub fn gen_BG_map(&mut self, cpu: &mut Cpu<'a>, buffer: PixelBuffer) {
        let mut x = 0;
        let mut y = 0;

        for offset in 0x9800..=0x9BFF {
            let id = cpu.readMem8(offset);
            let palette = cpu.mem.lcd.get_bw_palette();
            let tile = self.get_tile_by_id(cpu, id, false, palette);
            self.display_tile(buffer, x, y, tile);
            x+=8;
            if x>=255 {
                x = 0;
                y += 8;
            }
        }
    }

    pub fn display_BG_map(&mut self, cpu: &mut Cpu<'a> ) {
        self.gen_BG_map(cpu, PixelBuffer::BG);
        self.display_scroll_window(cpu, PixelBuffer::BG);
        self.bg_window.update_with_buffer(&mut self.buffer_bg, self.width, self.height)
            .unwrap();
    }


    pub fn update_screen(&mut self, cpu: &mut Cpu<'a> ) {
        let y = cpu.mem.lcd.get_cur_y() as usize;
        self.gen_BG_map_line(  cpu, PixelBuffer::Render, y);
        self.gen_OBJ_map_line( cpu, PixelBuffer::Render, y);
    }

    pub fn render_screen(&mut self) {
        self.render_window.update_with_buffer(&mut self.buffer_render, self.width, self.height).unwrap();
    }

    pub fn display_scroll_window(&mut self, cpu: &mut Cpu<'a>, buf: PixelBuffer) {
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
