use crate::{nes::ppu, nes::main::MAPPER};
use ppu::PPU;
use crate::nes::mapper::Mapper;

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

struct Rect {
    x1: usize,
    y1: usize,
    x2: usize,
    y2: usize,
}

impl Rect {
    fn new(x1: usize, y1: usize, x2: usize, y2: usize) -> Self {
        Rect { x1, y1, x2, y2 }
    }
}

pub struct Frame {
    pub data: Vec<u8>,
}

impl Frame {
    pub fn new() -> Self {
        Frame { data: vec![0; 3 * 256 * 240] }
    }

    fn background_palette(&self, ppu: &PPU, name_table: &[u8], tile_x: usize, tile_y: usize) -> [u8; 4] {
        let index = tile_x / 4 + tile_y / 4 * 8;
        let byte = name_table[index];
        let palette_index = match (tile_x % 4 / 2, tile_y % 4 / 2) {
            (0,0) => byte & 0b11,
            (1,0) => (byte >> 2) & 0b11,
            (0,1) => (byte >> 4) & 0b11,
            (1,1) => (byte >> 6) & 0b11,
            _ => panic!("can't be here"),
        };
        let start: usize = 1 + palette_index as usize * 4;
        let p = ppu.read_palette_table(tile_y * 8);
        [p[0] & 0x3f, p[start] & 0x3f, p[start + 1] & 0x3f, p[start + 2] & 0x3f]
    }

    fn draw_pixel(&mut self, x: usize, y: usize, rgb: (u8, u8, u8)) {
        let base = 3 * (256 * y + x);
        if base + 2 < self.data.len() {
            self.data[base] = rgb.0;
            self.data[base + 1] = rgb.1;
            self.data[base + 2] = rgb.2;
        }
    }

    fn render_name_table(&mut self, ppu: &PPU, name_table: &[u8], view_port: Rect, shift_x: isize, shift_y: isize) {
        let bank = ppu.background_addr();
        let attribute_table = &name_table[0x03c0..0x0400];
        for i in 0x0000..0x03c0 {
            let tile_column = i % 32;
            let tile_row = i / 32;
            let tile_idx = name_table[i] as u16;
            let start = bank + 16 * tile_idx;
            let mut tile: [u8; 16] = [0; 16];
            for i in 0..=15 {
                tile[i] = MAPPER.lock().unwrap().read_char_rom(start + i as u16);
            }
            let palette = self.background_palette(ppu, attribute_table, tile_column, tile_row);
            for y in 0..=7 {
                let mut upper = tile[y];
                let mut lower = tile[y + 8];
                for x in (0..=7).rev() {
                    let value = (1 & lower) << 1 | (1 & upper);
                    upper = upper >> 1;
                    lower = lower >> 1;
                    let rgb = match value {
                        0 => SYSTEM_PALLETE[palette[0] as usize],
                        1 => SYSTEM_PALLETE[palette[1] as usize],
                        2 => SYSTEM_PALLETE[palette[2] as usize],
                        3 => SYSTEM_PALLETE[palette[3] as usize],
                        _ => panic!("can't be here"),
                    };
                    let pixel_x = tile_column * 8 + x;
                    let pixel_y = tile_row * 8 + y;
                    if pixel_x >= view_port.x1 && pixel_x < view_port.x2 && pixel_y >= view_port.y1 && pixel_y < view_port.y2 {
                        self.draw_pixel((shift_x + pixel_x as isize) as usize, (shift_y + pixel_y as isize) as usize, rgb);
                    }
                }
            }
        }
    }

    fn sprite_palette(&self, ppu: &PPU, tile_y: usize, palette_idx: u8) -> [u8; 4] {
        let start = 0x11 + (palette_idx * 4) as usize;
        let p = ppu.read_palette_table(tile_y);
        [0, p[start] & 0x3f, p[start + 1] & 0x3f, p[start + 2] & 0x3f]
    }

    pub fn render(&mut self, ppu: &PPU) {
        //if ppu.show_back() {
            let scroll_x = ppu.get_scroll_x() as usize;
            let scroll_y = ppu.get_scroll_y() as usize;
            let (main_nametable, second_nametable) = match (&MAPPER.lock().unwrap().mirroring(), ppu.nametable_addr()) {
                (ppu::Mirroring::HORIZONTAL, 0x2000) | (ppu::Mirroring::HORIZONTAL, 0x2400) => {
                    (&ppu.name_table[0..0x400], &ppu.name_table[0x400..0x800])
                }
                (ppu::Mirroring::HORIZONTAL, 0x2800) | (ppu::Mirroring::HORIZONTAL, 0x2c00) => {
                    (&ppu.name_table[0x400..0x800], &ppu.name_table[0..0x400])
                }
                (ppu::Mirroring::VERTICAL, 0x2000) | (ppu::Mirroring::VERTICAL, 0x2800) => {
                    (&ppu.name_table[0..0x400], &ppu.name_table[0x400..0x800])
                }
                (ppu::Mirroring::VERTICAL, 0x2400) | (ppu::Mirroring::VERTICAL, 0x2c00) => {
                    (&ppu.name_table[0x400..0x800], &ppu.name_table[0x0..0x400])
                }
                (_, _) => panic!("not supported mirroring type"),
            };
            let screen_w = 256;
            let screen_h = 240;
                    // 左上
            self.render_name_table(
                ppu,
                main_nametable,
                Rect::new(scroll_x, scroll_y, screen_w, screen_h),
                -(scroll_x as isize),
                -(scroll_y as isize),
            );

            // 右下
            self.render_name_table(
                ppu,
                second_nametable,
                Rect::new(0, 0, scroll_x, scroll_y),
                (screen_w - scroll_x) as isize,
                (screen_h - scroll_y) as isize,
            );

            // 左下
            self.render_name_table(
                ppu,
                main_nametable,
                Rect::new(scroll_x, 0, screen_w, scroll_y),
                -(scroll_x as isize),
                (screen_h - scroll_y) as isize,
            );

            // 右上
            self.render_name_table(
                ppu,
                second_nametable,
                Rect::new(0, scroll_y, scroll_x, screen_h),
                (screen_w - scroll_x) as isize,
                -(scroll_y as isize),
            );
        //}

        //if ppu.show_sprite() {
            for i in (0..ppu.oam_data.len()).step_by(4) {
                let tile_index = ppu.oam_data[i + 1] as u16;
                let tile_x = ppu.oam_data[i + 3] as usize;
                let tile_y = ppu.oam_data[i] as usize;
                let tile_attr = ppu.oam_data[i + 2];
                let sprite_palette = self.sprite_palette(ppu, tile_y, tile_attr & 0x03);
                let bank: u16 = ppu.sprite_addr();
                let start = bank + 16 * tile_index;
                let mut tile: [u8; 16] = [0; 16];
                for i in 0..=15 {
                    tile[i] = MAPPER.lock().unwrap().read_char_rom(start + i as u16);
                }
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
        //}
    }
}
