mod bus;
mod cartridge;
mod cpu;
mod ppu;
mod joypad;

use bus::Bus;
use cartridge::Cartridge;
use cpu::CPU;
use ppu::PPU;
use std::collections::HashMap;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;

static SYSTEM_PALLETE: [(u8,u8,u8); 64] = [
    (0x80, 0x80, 0x80), (0x00, 0x3D, 0xA6), (0x00, 0x12, 0xB0), (0x44, 0x00, 0x96), (0xA1, 0x00, 0x5E),
    (0xC7, 0x00, 0x28), (0xBA, 0x06, 0x00), (0x8C, 0x17, 0x00), (0x5C, 0x2F, 0x00), (0x10, 0x45, 0x00),
    (0x05, 0x4A, 0x00), (0x00, 0x47, 0x2E), (0x00, 0x41, 0x66), (0x00, 0x00, 0x00), (0x05, 0x05, 0x05),
    (0x05, 0x05, 0x05), (0xC7, 0xC7, 0xC7), (0x00, 0x77, 0xFF), (0x21, 0x55, 0xFF), (0x82, 0x37, 0xFA),
    (0xEB, 0x2F, 0xB5), (0xFF, 0x29, 0x50), (0xFF, 0x22, 0x00), (0xD6, 0x32, 0x00), (0xC4, 0x62, 0x00),
    (0x35, 0x80, 0x00), (0x05, 0x8F, 0x00), (0x00, 0x8A, 0x55), (0x00, 0x99, 0xCC), (0x21, 0x21, 0x21),
    (0x09, 0x09, 0x09), (0x09, 0x09, 0x09), (0xFF, 0xFF, 0xFF), (0x0F, 0xD7, 0xFF), (0x69, 0xA2, 0xFF),
    (0xD4, 0x80, 0xFF), (0xFF, 0x45, 0xF3), (0xFF, 0x61, 0x8B), (0xFF, 0x88, 0x33), (0xFF, 0x9C, 0x12),
    (0xFA, 0xBC, 0x20), (0x9F, 0xE3, 0x0E), (0x2B, 0xF0, 0x35), (0x0C, 0xF0, 0xA4), (0x05, 0xFB, 0xFF),
    (0x5E, 0x5E, 0x5E), (0x0D, 0x0D, 0x0D), (0x0D, 0x0D, 0x0D), (0xFF, 0xFF, 0xFF), (0xA6, 0xFC, 0xFF),
    (0xB3, 0xEC, 0xFF), (0xDA, 0xAB, 0xEB), (0xFF, 0xA8, 0xF9), (0xFF, 0xAB, 0xB3), (0xFF, 0xD2, 0xB0),
    (0xFF, 0xEF, 0xA6), (0xFF, 0xF7, 0x9C), (0xD7, 0xE8, 0x95), (0xA6, 0xED, 0xAF), (0xA2, 0xF2, 0xDA),
    (0x99, 0xFF, 0xFC), (0xDD, 0xDD, 0xDD), (0x11, 0x11, 0x11), (0x11, 0x11, 0x11)
];

pub struct Frame {
    pub data: Vec<u8>,
}

impl Frame {
    fn new() -> Self {
        Frame { data: vec![0; 3 * 256 * 240] }
    }

    fn background_palette(&self, ppu: &PPU, tile_x: usize, tile_y: usize) -> [u8; 4] {
        let index = tile_x / 4 + tile_y / 4 * 8;
        let byte = ppu.name_table[0x03c0 + index];
        let palette_index = match (tile_x % 4 / 2, tile_y % 4 / 2) {
            (0,0) => byte & 0b11,
            (1,0) => (byte >> 2) & 0b11,
            (0,1) => (byte >> 4) & 0b11,
            (1,1) => (byte >> 6) & 0b11,
            _ => panic!("can't be here"),
        };
        let start: usize = 1 + palette_index as usize * 4;
        [ppu.palette[0], ppu.palette[start], ppu.palette[start + 1], ppu.palette[start + 2]]
    }

    fn draw_pixel(&mut self, x: usize, y: usize, rgb: (u8, u8, u8)) {
        let base = 3 * (256 * y + x);
        if base + 2 < self.data.len() {
            self.data[base] = rgb.0;
            self.data[base + 1] = rgb.1;
            self.data[base + 2] = rgb.2;
        }
    }

    fn render(&mut self, ppu: &PPU) {
        let addr = ppu.background_addr();
        for i in 0x0000..0x03c0 {
            let tile_index = ppu.name_table[i] as u16;
            let tile_x = i % 32;
            let tile_y = i / 32;
            let tile = &ppu.char_rom[(addr + 16 * tile_index) as usize ..= (addr + 16 * tile_index + 15) as usize];
            let palette = self.background_palette(ppu, tile_x, tile_y);
            for y in 0..=7 {
                let mut low = tile[y];
                let mut high = tile[y + 8];
                for x in 0..=7 {
                    let value = ((low & 0x80) >> 7) | ((high & 0x80) >> 6);
                    low <<= 1;
                    high <<= 1;
                    let rgb = match value {
                        0 => SYSTEM_PALLETE[ppu.palette[0] as usize],
                        1 => SYSTEM_PALLETE[palette[1] as usize],
                        2 => SYSTEM_PALLETE[palette[2] as usize],
                        3 => SYSTEM_PALLETE[palette[3] as usize],
                        _ => panic!("can't be here"),
                    };
                    self.draw_pixel(8 * tile_x + x, 8 * tile_y + y, rgb);
                }
            }
        }

        for i in (0..ppu.oam_data.len()).step_by(4) {
            let tile_index = ppu.oam_data[i + 1] as u16;
            let tile_x = ppu.oam_data[i + 3] as usize;
            let tile_y = ppu.oam_data[i] as usize;
            let tile_attr = ppu.oam_data[i + 2];
            let palette_index = (0x11 + 4 * (tile_attr & 0x03)) as usize;
            let sprite_palette: [usize; 4] = [0, ppu.palette[palette_index] as usize, ppu.palette[palette_index + 1] as usize, ppu.palette[palette_index + 2] as usize];
            let bank: u16 = ppu.sprite_addr();
            let tile = &ppu.char_rom[(bank + 16 * tile_index) as usize ..= (bank + 16 * tile_index + 15) as usize];
            for y in 0..=7 {
                let mut low = tile[y];
                let mut high = tile[y + 8];
                for x in 0..=7 {
                    let value = ((low & 0x80) >> 7) | ((high & 0x80) >> 6);
                    low <<= 1;
                    high <<= 1;
                    let rgb = match value {
                        0 => continue,
                        1 => SYSTEM_PALLETE[sprite_palette[1] as usize],
                        2 => SYSTEM_PALLETE[sprite_palette[2] as usize],
                        3 => SYSTEM_PALLETE[sprite_palette[3] as usize],
                        _ => panic!("can't be here"),
                    };
                    match (tile_attr & 0x40 != 0, tile_attr & 0x80 != 0) {
                        (false, false) => self.draw_pixel(tile_x + x, tile_y + y, rgb),
                        (true, false) => self.draw_pixel(tile_x + 7 - x, tile_y + y, rgb),
                        (false, true) => self.draw_pixel(tile_x + x, tile_y + 7 - y, rgb),
                        (true, true) => self.draw_pixel(tile_x + 7 - x, tile_y + 7 - y, rgb),
                    }
                }
            }
        }
    }
}

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let window = sdl_context.video().unwrap().window("MEMU", 256 * 3, 240 * 3).position_centered().build().unwrap();
    let mut canvas = window.into_canvas().present_vsync().build().unwrap();
    canvas.set_scale(3.0, 3.0).unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();
    let creator = canvas.texture_creator();
    let mut texture = creator.create_texture_target(PixelFormatEnum::RGB24, 256, 240).unwrap();

    let mut key_map = HashMap::new();
    key_map.insert(Keycode::Down, joypad::JoypadButton::DOWN);
    key_map.insert(Keycode::Up, joypad::JoypadButton::UP);
    key_map.insert(Keycode::Right, joypad::JoypadButton::RIGHT);
    key_map.insert(Keycode::Left, joypad::JoypadButton::LEFT);
    key_map.insert(Keycode::Space, joypad::JoypadButton::SELECT);
    key_map.insert(Keycode::Return, joypad::JoypadButton::START);
    key_map.insert(Keycode::A, joypad::JoypadButton::BUTTON_A);
    key_map.insert(Keycode::S, joypad::JoypadButton::BUTTON_B);

    let cart = Cartridge::new(&std::fs::read("cartridge/alter.nes").unwrap());
    let mut frame = Frame::new();
    let bus = Bus::new(cart, move |ppu, joypad| {
        println!("*** GameLoop ***");
        frame.render(ppu);
        texture.update(None, &frame.data, 256 * 3).unwrap();
        canvas.copy(&texture, None, None).unwrap();
        canvas.present();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => std::process::exit(0),
                Event::KeyDown { keycode, .. } => {
                    if let Some(key) = key_map.get(&keycode.unwrap_or(Keycode::Ampersand)) {
                        joypad.set_button_pressed_status(*key, true);
                    }
                }
                Event::KeyUp { keycode, .. } => {
                    if let Some(key) = key_map.get(&keycode.unwrap_or(Keycode::Ampersand)) {
                        joypad.set_button_pressed_status(*key, false);
                    }
                }
                _ => {},
            }
        }
    });
    let mut cpu = CPU::new(bus);
    cpu.run(|cpu| {
        //cpu.log();
    });
}
