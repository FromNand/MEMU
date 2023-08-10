use crate::nes::main::MAPPER;
use crate::nes::mapper::Mapper;

#[derive(Clone, Copy)]
pub enum Mirroring {
    HORIZONTAL,
    VERTICAL,
    FOUR_SCREEN,
}

pub struct PPU {
    pub cycle: usize,
    pub scanline: usize,
    scanline_palette_indexes: Vec<usize>,
    scanline_palette_tables: Vec<[u8; 0x20]>,
    pub nmi: bool,
    pub clear_nmi: bool,
    internal_data: u8,
    ppuctrl: PPUCtrl,
    ppumask: PPUMask,
    ppustts: PPUStts,
    oam_addr: u8,
    ppuscrl: PPUScrl,
    ppuaddr: PPUAddr,
    pub name_table: [u8; 0x0800],
    pub palette: [u8; 0x0020],
    pub oam_data: [u8; 0x0100],
}

impl PPU {
    pub fn new() -> Self {
        PPU {
            cycle: 0,
            scanline: 0,
            scanline_palette_indexes: vec![],
            scanline_palette_tables: vec![],
            nmi: false,
            clear_nmi: false,
            internal_data: 0,
            ppuctrl: PPUCtrl { name_table_addr: 0, increment: false, sprite_addr: false, background_addr: false, sprite_size: false, slave: false, enable_nmi: false },
            ppumask: PPUMask { gray_scale: false, show_left_back: false, show_left_sprite: false, show_back: false, show_sprite: false, emphasize_red: false, emphasize_green: false, emphasize_blue: false },
            ppustts: PPUStts { open_bus: 0x10, sprite_overflow: false, sprite0_hit: false, in_vblank: false },
            oam_addr: 0x00,
            ppuscrl: PPUScrl { scroll_x: 0, scroll_y: 0, select_y: false },
            ppuaddr: PPUAddr { addr: 0x0000, access_low: false },
            name_table: [0; 0x0800],
            palette: [0; 0x0020],
            oam_data: [0; 0x0100],
        }
    }

    pub fn show_sprite(&self) -> bool {
        self.ppumask.show_sprite
    }

    pub fn show_back(&self) -> bool {
        self.ppumask.show_back
    }

    fn mirror_palette_addr(&self, addr: u16) -> u16 {
        let mirror_addr = addr & 0x001f;
        match mirror_addr {
            0x0010 => 0x0000,
            0x0014 => 0x0004,
            0x0018 => 0x0008,
            0x001c => 0x000c,
            _ => mirror_addr,
        }
    }

    fn write_palette_table(&mut self, addr: u16, value: u8) {
        let addr = self.mirror_palette_addr(addr) as usize;
        self.palette[addr] = value;
        let scanline = self.scanline;
        let last_scanline = self.scanline_palette_indexes.last().unwrap_or(&0);
        if *last_scanline != scanline {
            self.scanline_palette_indexes.push(scanline);
            self.scanline_palette_tables
                .push(self.palette.clone());
        } else {
            self.scanline_palette_tables.pop();
            self.scanline_palette_tables
                .push(self.palette.clone());
        }
    }

    fn clear_palette_table_histories(&mut self) {
        self.scanline_palette_indexes = vec![];
        self.scanline_palette_tables = vec![];
    }

    pub fn read_palette_table(&self, scanline: usize) -> &[u8; 32] {
        if self.scanline_palette_indexes.is_empty() {
            return &self.palette;
        }
        let mut index = 0;
        for (i, s) in self.scanline_palette_indexes.iter().enumerate() {
            if *s > scanline {
                break;
            }
            index = i
        }
        let table = &self.scanline_palette_tables[index];
        table
    }

    pub fn nametable_addr(&self) -> u16 {
        match self.ppuctrl.name_table_addr {
            0 => 0x2000,
            1 => 0x2400,
            2 => 0x2800,
            3 => 0x2c00,
            _ => panic!("can't be here"),
        }
    }

    pub fn background_addr(&self) -> u16 {
        if self.ppuctrl.background_addr { 0x1000 } else { 0x0000 }
    }

    pub fn sprite_addr(&self) -> u16 {
        if self.ppuctrl.sprite_addr { 0x1000 } else { 0x0000 }
    }

    pub fn read_ppuctrl(&self) -> u8 {
        self.ppuctrl.get()
    }

    pub fn write_to_ppuctrl(&mut self, data: u8) {
        let old_enable_nmi = self.ppuctrl.enable_nmi;
        self.ppuctrl.set(data);
        if !old_enable_nmi && self.ppuctrl.enable_nmi && self.ppustts.in_vblank {
            self.nmi = true;
        }
    }

    pub fn read_ppumask(&self) -> u8 {
        self.ppumask.get()
    }

    pub fn write_to_ppumask(&mut self, data: u8) {
        self.ppumask.set(data);
    }

    pub fn read_ppustts(&mut self) -> u8 {
        let mut data = self.ppustts.open_bus;
        if self.ppustts.sprite_overflow { data |= 0x20 };
        if self.ppustts.sprite0_hit { data |= 0x40 };
        if self.ppustts.in_vblank { data |= 0x80 };
        self.clear_nmi = true;
        self.ppustts.in_vblank = false;
        self.ppuaddr.reset();
        self.ppuscrl.reset();
        data
    }

    pub fn write_to_oam_addr(&mut self, data: u8) {
        self.oam_addr = data;
    }

    pub fn read_oam_data(&self) -> u8 {
        self.oam_data[self.oam_addr as usize]
    }

    pub fn write_to_oam_data(&mut self, data: u8) {
        self.oam_data[self.oam_addr as usize] = data;
        self.oam_addr = self.oam_addr.wrapping_add(1);
    }

    pub fn get_scroll_x(&self) -> u8 {
        self.ppuscrl.scroll_x
    }

    pub fn get_scroll_y(&self) -> u8 {
        self.ppuscrl.scroll_y
    }

    pub fn write_to_ppuscrl(&mut self, data: u8) {
        self.ppuscrl.write(data);
    }

    pub fn write_to_ppuaddr(&mut self, data: u8) {
        self.ppuaddr.set(data);
    }

    fn increment_ppuaddr(&mut self) {
        self.ppuaddr.increment(if self.ppuctrl.increment { 32 } else { 1 });
    }

    pub fn write_to_oam_dma(&mut self, data: [u8; 256]) {
        self.oam_data = data;
    }

    pub fn read(&mut self) -> u8 {
        let addr = self.ppuaddr.addr & 0x3fff;
        self.increment_ppuaddr();
        match addr {
            0x0000..=0x1fff => {
                let data = self.internal_data;
                self.internal_data = MAPPER.lock().unwrap().read_char_rom(addr);
                data
            }
            0x2000..=0x3eff => {
                let data = self.internal_data;
                let mirror_addr = addr & 0x0fff;
                let mirror_addr = match (&MAPPER.lock().unwrap().mirroring(), mirror_addr / 0x0400) {
                    (Mirroring::HORIZONTAL, 1) | (Mirroring::HORIZONTAL, 2) => mirror_addr - 0x0400,
                    (Mirroring::HORIZONTAL, 3) | (Mirroring::VERTICAL, 2) | (Mirroring::VERTICAL, 3) => mirror_addr - 0x0800,
                    _ => mirror_addr,
                };
                self.internal_data = self.name_table[mirror_addr as usize];
                data
            }
            0x3f00..=0x3fff => {
                self.internal_data = self.palette[self.mirror_palette_addr(addr) as usize];
                self.internal_data
            }
            _ => panic!("Read from 0x{:04X} in PPU", addr),
        }
    }

    pub fn write(&mut self, data: u8) {
        let addr = self.ppuaddr.addr & 0x3fff;
        self.increment_ppuaddr();
        match addr {
            0x0000..=0x1fff => {
                if MAPPER.lock().unwrap().cart.is_char_ram {
                    MAPPER.lock().unwrap().write_char_rom(addr, data);
                }
            }
            0x2000..=0x3eff => {
                let mirror_addr = addr & 0x0fff;
                let mirror_addr = match (&MAPPER.lock().unwrap().mirroring(), mirror_addr / 0x0400) {
                    (Mirroring::HORIZONTAL, 1) | (Mirroring::HORIZONTAL, 2) => mirror_addr - 0x0400,
                    (Mirroring::HORIZONTAL, 3) | (Mirroring::VERTICAL, 2) | (Mirroring::VERTICAL, 3) => mirror_addr - 0x0800,
                    _ => mirror_addr,
                };
                self.name_table[mirror_addr as usize] = data;
            }
            0x3f00..=0x3fff => {
                //self.palette[self.mirror_palette_addr(addr) as usize] = data;
                self.write_palette_table(addr, data);
            }
            _ => panic!("Write to  0x{:04X} in PPU", addr),
        }
    }

    fn is_sprite0_hit(&self, cycle: usize) -> bool {
        let y = self.oam_data[0] as usize;
        let x = self.oam_data[3] as usize;
        (y == self.scanline as usize) && x <= cycle && self.ppumask.show_sprite
    }

    pub fn tick(&mut self, cycle: usize) -> bool {
        self.cycle += cycle;
        if self.cycle >= 341 {
            if self.is_sprite0_hit(self.cycle) {
                self.ppustts.sprite0_hit = true;
            }
            self.cycle -= 341;
            self.scanline += 1;
            if self.scanline == 241 {
                self.ppustts.in_vblank = true;
                self.ppustts.sprite0_hit = false;
                if self.ppuctrl.enable_nmi {
                    self.nmi = true;
                }
            }
            if self.scanline >= 262 {
                self.scanline = 0;
                self.ppustts.in_vblank = false;
                self.ppustts.sprite0_hit = false;
                self.nmi = false;
                self.clear_palette_table_histories();
                return true;
            }
        }
        return false;
    }
}

struct PPUCtrl {
    name_table_addr: u8,
    increment: bool,
    sprite_addr: bool,
    background_addr: bool,
    sprite_size: bool,
    slave: bool,
    enable_nmi: bool,
}

impl PPUCtrl {
    fn set(&mut self, data: u8) {
        self.name_table_addr = data & 0x03;
        self.increment = data & 0x04 != 0;
        self.sprite_addr = data & 0x08 != 0;
        self.background_addr = data & 0x10 != 0;
        self.sprite_size = data & 0x20 != 0;
        self.slave = data & 0x40 != 0;
        self.enable_nmi = data & 0x80 != 0;
    }

    fn get(&self) -> u8 {
        let mut value = self.name_table_addr;
        if self.increment { value |= 0x04 };
        if self.sprite_addr { value |= 0x08 };
        if self.background_addr { value |= 0x10 };
        if self.sprite_size { value |= 0x20 };
        if self.slave { value |= 0x40 };
        if self.enable_nmi { value |= 0x80 };
        value
    }
}

struct PPUMask {
    gray_scale: bool,
    show_left_back: bool,
    show_left_sprite: bool,
    show_back: bool,
    show_sprite: bool,
    emphasize_red: bool,
    emphasize_green: bool,
    emphasize_blue: bool,
}

impl PPUMask {
    fn set(&mut self, data: u8) {
        if data & 0x01 != 0 { self.gray_scale = true }
        if data & 0x02 != 0 { self.show_left_back = true }
        if data & 0x04 != 0 { self.show_left_sprite = true }
        if data & 0x08 != 0 { self.show_back = true }
        if data & 0x10 != 0 { self.show_sprite = true }
        if data & 0x20 != 0 { self.emphasize_red = true }
        if data & 0x40 != 0 { self.emphasize_green = true }
        if data & 0x80 != 0 { self.emphasize_blue = true }
    }

    fn get(&self) -> u8 {
        let mut value = 0;
        if self.gray_scale { value |= 0x01 };
        if self.show_left_back { value |= 0x02 };
        if self.show_left_sprite { value |= 0x04 };
        if self.show_back { value |= 0x08 };
        if self.show_sprite { value |= 0x10 };
        if self.emphasize_red { value |= 0x20 };
        if self.emphasize_green { value |= 0x40 };
        if self.emphasize_blue { value |= 0x80 };
        value
    }
}

struct PPUStts {
    open_bus: u8,
    sprite_overflow: bool,
    sprite0_hit: bool,
    in_vblank: bool,
}

struct PPUScrl {
    scroll_x: u8,
    scroll_y: u8,
    select_y: bool,
}

impl PPUScrl {
    fn write(&mut self, data: u8) {
        if self.select_y {
            self.scroll_y = data;
        } else {
            self.scroll_x = data;
        }
        self.select_y = !self.select_y;
    }

    fn reset(&mut self) {
        self.select_y = false;
    }
}

struct PPUAddr {
    addr: u16,
    access_low: bool,
}

impl PPUAddr {
    fn reset(&mut self) {
        self.access_low = false;
    }

    fn set(&mut self, addr: u8) {
        if self.access_low {
            self.addr = (self.addr & 0xff00) + (addr as u16);
        } else {
            self.addr = (self.addr & 0x00ff) + ((addr as u16) << 8);
        }
        self.access_low = !self.access_low;
    }

    fn increment(&mut self, data: u16) {
        self.addr = self.addr.wrapping_add(data);
    }
}
