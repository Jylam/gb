// Graphical; renderer
#![allow(non_snake_case)]
#![allow(unused_imports)]

extern crate minifb;
extern crate image;
extern crate webp_animation;

use std::fs;
use std::fs::File;
use std::io::Write;
use self::webp_animation::{Encoder};
use minifb::{Key, KeyRepeat, Window, WindowOptions, Scale, ScaleMode};
use std::thread::sleep;
use std::time::{SystemTime, UNIX_EPOCH};
use std::borrow::Borrow;
use std::mem::swap;
use std::marker::PhantomData;
use std::process;

use std::time::Duration;

use lr35902::Cpu;

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
    f11_pressed: bool,
    f12_pressed: bool,
    recording: bool,
    webp_encoder: webp_animation::Encoder,
    webp_timestamp: i32,

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
        let bg_window = Window::new(
            "BGMap - ESC to exit",
            256,
            256,
            WindowOptions::default(),
            )
            .unwrap_or_else(|e| {
                panic!("{}", e);
            });
        let mut render_window = Window::new(
            "Render - ESC to exit",
            160,
            144,
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
        let render = Render {
            render_window: render_window,
            bg_window: bg_window,
            tiles_window: tiles_window,
            width: 256,
            height: 256,
            buffer_render: vec![0x00; 256*256],
            buffer_bg:     vec![0x00; 256*256],
            buffer_tiles:  vec![0x00; 256*256],
            f1_pressed: false,
            f11_pressed: false,
            f12_pressed: false,
            recording: false,
            webp_encoder: Encoder::new((160, 144)).unwrap(),
            webp_timestamp: 0,
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

        // Disasm
        if self.render_window.is_key_pressed(Key::F1, KeyRepeat::No) {
            if self.f1_pressed == false {
                cpu.toggle_disasm();
            }
            self.f1_pressed = true;
        }
        if self.render_window.is_key_released(Key::F1) {
            self.f1_pressed = false;
        }

        // Screenshot
        if self.render_window.is_key_pressed(Key::F11, KeyRepeat::No) {
            if self.f11_pressed == false {
                println!("Saving image");
                let mut buffer = vec![0x00_u8; 160*144*3];
                let mut offset = 0;
                for y in 0..144 {
                    for x in 0..160 {
                        let b = self.buffer_render[x+y*256];
                        buffer[offset]   = ((b&0x00FF0000)>>16) as u8;
                        buffer[offset+1] = ((b&0x0000FF00)>>8)  as u8;
                        buffer[offset+2] = (b&0x000000FF)       as u8;
                        offset+=3;
                    }
                }
                image::save_buffer("kuk.png", buffer.as_slice(), 160, 144, image::ColorType::Rgb8).unwrap();
                self.f11_pressed = true;
            }
        }
        if self.render_window.is_key_released(Key::F11) {
            self.f11_pressed = false;
        }

        // Animation
        if self.render_window.is_key_pressed(Key::F12, KeyRepeat::No) {
            if self.f12_pressed == false {
                if self.webp_timestamp == 0 {
                    self.webp_encoder = Encoder::new((160, 144)).unwrap();
                    self.recording = true;
                } else {
                    println!("SAVING");
                    let w = self.webp_timestamp;
                    let mut webpe = Encoder::new((160, 144)).unwrap();
                    swap(&mut webpe, &mut self.webp_encoder);

                    let contents = &webpe.finalize(w).unwrap();

                    fs::write("test.webp", contents).unwrap();

                    self.webp_timestamp = 0;
                    self.recording = false;
                }
            }
            self.f12_pressed = true;
        }
        if self.render_window.is_key_released(Key::F12) {
            self.f12_pressed = false;
        }


        self.bg_window.is_key_down(Key::Escape) ||
            self.tiles_window.is_key_down(Key::Escape) ||
            self.render_window.is_key_down(Key::Escape)
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

        match c {
            /*            0x00 => {r=0x00; g=0x00; b=0x00;}, // Black
                          0x01 => {r=0x55; g=0x55; b=0x55;}, // Dark gray
                          0x02 => {r=0xAA; g=0xAA; b=0xAA;}, // Light gray
                          0x03 => {r=0xFF; g=0xFF; b=0xFF;}, // White
                          */
            0x00 => {r=0x40; g=0x50; b=0x10;}, // Black
            0x01 => {r=0x70; g=0x80; b=0x28;}, // Dark gray
            0x02 => {r=0xA0; g=0xA8; b=0x40;}, // Light gray
            0x03 => {r=0xD0; g=0xD0; b=0x58;}, // White

            // Special colors
            0x55 => {r=0xFF; g=0x00; b=0x00;}, // Red
            0xAA => {r=0x00; g=0xFF; b=0x00;}, // Green
            0xBB => {r=0x00; g=0x00; b=0xFF;}, // Blue
            _    => {r=0xFF; g=0xFF; b=0xFF;}  // Default White
        }

        self.put_pixel24(buf, x, y, r, g, b);
    }

    pub fn get_tile_by_id(&mut self, cpu: &mut Cpu<'a>, id: u8, is_sprite: bool) -> Vec<u8> {
        let addr = cpu.mem.lcd.get_tile_addr(id, is_sprite);
        self.get_tile_at_addr(cpu, addr)
    }

    pub fn get_tile_at_addr(&mut self, cpu: &mut Cpu<'a>, addr: u16) -> Vec<u8> {

        let mut ret = vec![0; 8*8];
        let mut offset = addr;
        for i in 0..8 {
            let a = cpu.readMem8(offset);
            let b = cpu.readMem8(offset+1);

            let p1 = ((a&0b10000000)>>7) | (b&0b10000000)>>6;
            let p2 = ((a&0b01000000)>>6) | (b&0b01000000)>>5;
            let p3 = ((a&0b00100000)>>5) | (b&0b00100000)>>4;
            let p4 = ((a&0b00010000)>>4) | (b&0b00010000)>>3;
            let p5 = ((a&0b00001000)>>3) | (b&0b00001000)>>2;
            let p6 = ((a&0b00000100)>>2) | (b&0b00000100)>>1;
            let p7 = ((a&0b00000010)>>1) | (b&0b00000010)>>0;
            let p8 = ((a&0b00000001)>>0) | (b&0b00000001)<<1;

            offset+=2;

            // Put 0xFF if the color is transparent
            ret[0+i*8] = p1;
            ret[1+i*8] = p2;
            ret[2+i*8] = p3;
            ret[3+i*8] = p4;
            ret[4+i*8] = p5;
            ret[5+i*8] = p6;
            ret[6+i*8] = p7;
            ret[7+i*8] = p8;
        }
        ret
    }

    pub fn display_tile(&mut self, cpu: &mut Cpu<'a>, buf: PixelBuffer, x: usize, y: usize, buft: Vec<u8>) {

        let palette = cpu.mem.lcd.get_bw_palette();

        for ty in 0..8 {
            for tx in 0..8 {
                self.put_pixel8(buf, x+tx, y+ty, palette[buft[tx+ty*8] as usize]);
            }
        }
    }

    pub fn display_tile_pattern_tables(&mut self, cpu: &mut Cpu<'a> ) {
        let mut x = 0;
        let mut y = 0;

        for j in (0x8000..0x97FF).step_by(16) {
            let tile = self.get_tile_at_addr(cpu, j);
            self.display_tile(cpu, PixelBuffer::Tiles, x, y, tile);
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

        let lcdc = cpu.mem.read8(0xFF40);
        let bgmap = if lcdc&0b0000_1000!=0 { 0x9C00 } else {0x9800};
        // X and Y offset in the 32x32 BGMAP
        let xoff = (x / 8)%32;
        let yoff = (y / 8)%32;
        // Pixel in the tile
        let xrest = (x-(xoff*8))%256;
        let yrest = (y-(yoff*8))%256;
        // Offset in the BGMAP
        let bgoff = xoff+yoff*32;
        // Tile ID
        let id = cpu.readMem8(bgmap+bgoff as u16);
        // Tile Pixels
        let tile = self.get_tile_by_id(cpu, id, false);
        // Get Pixel value
        tile[xrest+yrest*8]
    }
    pub fn get_win_pixel_at(&mut self, cpu: &mut Cpu<'a>, x: usize, y: usize) -> u8 {

        let lcdc = cpu.mem.read8(0xFF40);
        let winmap = if lcdc&0b0100_0000!=0 { 0x9C00 } else {0x9800};
        // X and Y offset in the 32x32 WIN
        let xoff = (x / 8)%32;
        let yoff = (y / 8)%32;
        // Pixel in the tile
        let xrest = (x-(xoff*8))%256;
        let yrest = (y-(yoff*8))%256;
        // Offset in the WINMAP
        let winoff = xoff+yoff*32;
        // Tile ID
        let id = cpu.readMem8(winmap+winoff as u16);
        // Tile Pixels
        let tile = self.get_tile_by_id(cpu, id, false);
        // Get Pixel value
        tile[xrest+yrest*8]
    }

    pub fn gen_WIN_map_line(&mut self, cpu: &mut Cpu<'a>, buffer: PixelBuffer, line: usize) {
        if line>144 {
            return;
        }
        let lcdc = cpu.mem.read8(0xFF40);
        if lcdc&0b0010_0000 == 0 {
            return;
        }

        let WY  = cpu.mem.lcd.get_wy() as usize;
        if WY>line {
            return;
        }

        let WX  = cpu.mem.lcd.get_wx() as usize - 7;
        let palette = cpu.mem.lcd.get_bw_palette();

        for x in 0..160 {
            if (lcdc & 0b0000_0001) == 1 {
                let c = self.get_win_pixel_at(cpu, x, line-WY);
                self.put_pixel8(buffer, x+WX, line, palette[c as usize]);
            } else {
                self.put_pixel8(buffer, x, line, 0x03);
            }
        }
    }
    pub fn gen_BG_map_line(&mut self, cpu: &mut Cpu<'a>, buffer: PixelBuffer, line: usize) {
        if line>144 {
            return;
        }
        let SCY  = cpu.mem.lcd.get_scy() as usize;
        let SCX  = cpu.mem.lcd.get_scx() as usize;
        let lcdc = cpu.mem.read8(0xFF40);

        if lcdc & 1 == 0 {
            return;
        }

        let palette = cpu.mem.lcd.get_bw_palette();


        for x in 0..160 {
            if (lcdc & 0b0000_0001) == 0x01 {
                let c = self.get_bg_pixel_at(cpu,  x + SCX, line + SCY);
                self.put_pixel8(buffer, x, line, palette[c as usize]);
            } else {
                self.put_pixel8(buffer, x, line, 0x03);
            }
        }
    }

    pub fn gen_OBJ_map_line(&mut self, cpu: &mut Cpu<'a>, buffer: PixelBuffer, line: usize) {
        if line>144 {
            return;
        }
        let mut offset: u16;
        let lcdc = cpu.mem.read8(0xFF40);

        // OBJ Disabled
        if (lcdc&0b0000_0010) == 0 {
            return;
        }
        let h = if (lcdc&0b0000_0100)!=0 { 16 } else { 8 };

        let mut count = 0;
        // Loop through sprites
        'oamloop: for i in 0..40 {
            if count==10 {
                return;
            }
            // Sprite position
            offset = 0xFE00 + (i*4);
            let py = (cpu.readMem8(offset) as isize)-16;

            // Sprite doesn't intersect the line
            if (py>line as isize) || ((py + (h-1)) < line as isize) {
                continue 'oamloop;
            }

            let flags = cpu.readMem8(offset+3);
            let _xflip = flags&0b0010_0000 != 0;
            let _yflip = flags&0b0100_0000 != 0;
            let mut tile_index = cpu.readMem8(offset+2);
            let palette = cpu.mem.lcd.get_sprite_palette(((flags&0b0001_0000)>>4) as u16);
            let px = (cpu.readMem8(offset+1) as isize)-8;

            // Flip Y
            let mut y = if _yflip {(h-1) as usize -(line-py as usize)} else {line-py as usize};

            // Double height ?
            if h==16 {
                tile_index = tile_index&0b1111_1110;
            }
            let tile;
            if y<8 {
                tile = self.get_tile_by_id(cpu, tile_index, true);
            } else {
                tile = self.get_tile_by_id(cpu, tile_index+1, true);
                y = y-8;
            }

            for x in 0..=7 {
                let ox = if _xflip {7-x} else {x};
                let c = tile[ox+y*8];
                if c!=0x00 && (x+px as usize)<160 {

                    // OBJ Priority over BG (and WIN FIXME)
                    if (flags&0b1000_0000)==0
                        || ((flags&0b1000_0000)!=0 && self.get_bg_pixel_at(cpu, x+px as usize, line)!=0x00)
                            || ((flags&0b1000_0000)!=0 && self.get_win_pixel_at(cpu, x+px as usize, line)!=0x00) {
                                self.put_pixel8(buffer, x+px as usize, line, palette[c as usize]);
                            }
                }
            }
            count+=1;
        }
    }

    pub fn gen_BG_map(&mut self, cpu: &mut Cpu<'a>, buffer: PixelBuffer) {
        let mut x = 0;
        let mut y = 0;

        for offset in 0x9800..=0x9BFF {
            let id = cpu.readMem8(offset);
            let tile = self.get_tile_by_id(cpu, id, false);
            self.display_tile(cpu, buffer, x, y, tile);
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
        let lcdc = cpu.mem.read8(0xFF40);
        if lcdc&0b1000_0000 != 0 {
            self.gen_BG_map_line(  cpu, PixelBuffer::Render, y);
            self.gen_WIN_map_line( cpu, PixelBuffer::Render, y);
            self.gen_OBJ_map_line( cpu, PixelBuffer::Render, y);
        } else {
            for y in 0..144 {
                for x in 0..160 {
                    self.put_pixel8(PixelBuffer::Render, x, y, 0x03);
                }
            }
        }
    }

    pub fn render_screen(&mut self) {
        let mut buf = vec![0x00; 160*144];
        for y in 0..144 {
            for x in 0..160 {
                buf[x+y*160] = self.buffer_render[x+y*256];
            }
        }
        self.render_window.update_with_buffer(&mut buf, 160, 144).unwrap();

        if self.recording {

            let mut rgba8 = vec![0 as u8; 160*144*4];
            let mut offset = 0;
            for y in 0..144 {
                for x in 0..160 {
                    let pixel = self.buffer_render[x+y*256];
                    rgba8[offset]   = ((pixel&0x00FF0000)>>16) as u8;
                    rgba8[offset+1] = ((pixel&0x0000FF00)>>8) as u8;
                    rgba8[offset+2] = ((pixel&0x000000FF)) as u8;
                    rgba8[offset+3] = 0xFF as u8;
                    offset+=4;

                }
            }
            self.webp_encoder.add_frame(rgba8.as_slice(), self.webp_timestamp).unwrap();
            self.webp_timestamp += 16;
        }
    }

    pub fn display_scroll_window(&mut self, cpu: &mut Cpu<'a>, buf: PixelBuffer) {
        let SCY  = cpu.mem.lcd.get_scy() as usize;
        let SCX  = cpu.mem.lcd.get_scx() as usize;
        let cury = cpu.mem.lcd.get_cur_y() as usize;

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
