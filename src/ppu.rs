pub enum Mirroring {
    HORIZONTAL,
    VERTICAL,
    FOUR_SCREEN,
}

pub struct PPU {
    pub cycle: usize,
    pub scanline: usize,
    pub nmi: bool,
    internal_data: u8,
    ppuctrl: PPUCtrl,
    ppumask: PPUMask,
    ppustts: PPUStts,
    oam_addr: u8,
    ppuaddr: PPUAddr,
    mirroring: Mirroring,
    pub char_rom: Vec<u8>,
    pub name_table: [u8; 0x0800],
    pub palette: [u8; 0x0020],
    pub oam_data: [u8; 0x0100],
}

impl PPU {
    pub fn new(mirroring: Mirroring, char_rom: Vec<u8>) -> Self {
        PPU {
            cycle: 0,
            scanline: 0,
            nmi: false,
            internal_data: 0,
            ppuctrl: PPUCtrl { name_table_addr: 0, increment: false, sprite_addr: false, background_addr: false, sprite_size: false, slave: false, enable_nmi: false },
            ppumask: PPUMask { gray_scale: false, show_left_back: false, show_left_sprite: false, show_back: false, show_sprite: false, emphasize_red: false, emphasize_green: false, emphasize_blue: false },
            ppustts: PPUStts { open_bus: 0x00, sprite_overflow: false, sprite0_hit: false, in_vblank: false },
            oam_addr: 0x00,
            ppuaddr: PPUAddr { addr: 0x0000, access_low: false },
            mirroring,
            char_rom,
            name_table: [0; 0x0800],
            palette: [0; 0x0020],
            oam_data: [0; 0x0100],
        }
    }

    pub fn background_addr(&self) -> u16 {
        if self.ppuctrl.background_addr { 0x1000 } else { 0x0000 }
    }

    pub fn sprite_addr(&self) -> u16 {
        if self.ppuctrl.sprite_addr { 0x1000 } else { 0x0000 }
    }

    pub fn write_to_ppuctrl(&mut self, data: u8) {
        let old_enable_nmi = self.ppuctrl.enable_nmi;
        self.ppuctrl.set(data);
        if !old_enable_nmi && self.ppuctrl.enable_nmi && self.ppustts.in_vblank {
            self.nmi = true;
        }
    }

    pub fn write_to_ppumask(&mut self, data: u8) {
        self.ppumask.set(data);
    }

    pub fn read_ppustts(&mut self) -> u8 {
        let mut data = self.ppustts.open_bus;
        if self.ppustts.sprite_overflow { data |= 0x20 };
        if self.ppustts.sprite0_hit { data |= 0x40 };
        if self.ppustts.in_vblank { data |= 0x80 };
        self.ppustts.in_vblank = false;
        self.ppuaddr.reset();
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
                self.internal_data = self.char_rom[addr as usize];
                data
            }
            0x2000..=0x3eff => {
                let data = self.internal_data;
                let mirror_addr = addr & 0x0fff;
                let mirror_addr = match (&self.mirroring, mirror_addr / 0x0400) {
                    (Mirroring::HORIZONTAL, 1) | (Mirroring::HORIZONTAL, 2) => mirror_addr - 0x0400,
                    (Mirroring::HORIZONTAL, 3) | (Mirroring::VERTICAL, 2) | (Mirroring::VERTICAL, 3) => mirror_addr - 0x0800,
                    _ => mirror_addr,
                };
                self.internal_data = self.name_table[mirror_addr as usize];
                data
            }
            0x3f00..=0x3fff => {
                let mirror_addr = addr & 0x001f;
                let mirror_addr = match mirror_addr {
                    0x0010 => 0x0000,
                    0x0014 => 0x0004,
                    0x0018 => 0x0008,
                    0x001c => 0x000c,
                    _ => mirror_addr,
                };
                self.palette[mirror_addr as usize]
            }
            _ => panic!("Read from 0x{:04X} in PPU", addr),
        }
    }

    pub fn write(&mut self, data: u8) {
        let addr = self.ppuaddr.addr & 0x3fff;
        self.increment_ppuaddr();
        match addr {
            0x0000..=0x1fff => {},
            0x2000..=0x3eff => {
                let mirror_addr = addr & 0x0fff;
                let mirror_addr = match (&self.mirroring, mirror_addr / 0x0400) {
                    (Mirroring::HORIZONTAL, 1) | (Mirroring::HORIZONTAL, 2) => mirror_addr - 0x0400,
                    (Mirroring::HORIZONTAL, 3) | (Mirroring::VERTICAL, 2) | (Mirroring::VERTICAL, 3) => mirror_addr - 0x0800,
                    _ => mirror_addr,
                };
                self.name_table[mirror_addr as usize] = data;
            }
            0x3f00..=0x3fff => {
                let mirror_addr = addr & 0x001f;
                let mirror_addr = match mirror_addr {
                    0x0010 => 0x0000,
                    0x0014 => 0x0004,
                    0x0018 => 0x0008,
                    0x001c => 0x000c,
                    _ => mirror_addr,
                };
                self.palette[mirror_addr as usize] = data;
            }
            _ => panic!("Write to  0x{:04X} in PPU", addr),
        }
    }

    pub fn tick(&mut self, cycle: usize) -> bool {
        self.cycle += cycle;
        if self.cycle >= 341 {
            self.cycle -= 341;
            self.scanline += 1;
            // check sprite0 hit
            if self.scanline == 241 {
                // clear sprite0 hit
                self.ppustts.in_vblank = true;
                if self.ppuctrl.enable_nmi {
                    self.nmi = true;
                }
            }
            if self.scanline >= 262 {
                self.scanline = 0;
                self.ppustts.in_vblank = false;
                self.nmi = false;
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
}

struct PPUStts {
    open_bus: u8,
    sprite_overflow: bool,
    sprite0_hit: bool,
    in_vblank: bool,
}

struct PPUAddr {
    addr: u16,
    access_low: bool,
}

impl PPUAddr {
    fn reset(&mut self) {
        self.addr = 0x0000;
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
