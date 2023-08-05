use crate::{ppu::Mirroring, cartridge::Cartridge};
use std::fs::File;
use std::io::Write;

// pub fn create_mapper(cart: Cartridge) -> Mapper {
//     let mapper = match cart.mapper {
//         0 => Mapper0::new(),
//         1 => Mapper1::new(),
//         2 => Mapper2::new(),
//         _ => panic!("invalid mapper"),
//     };
//     return mapper;
// }

pub trait Mapper {
    fn write(&mut self, addr: u16, data: u8);
    fn mirroring(&self) -> Mirroring;
    fn read_prog_rom(&self, addr: u16) -> u8;
    fn read_char_rom(&self, addr: u16) -> u8;
    fn write_char_rom(&mut self, addr: u16, data: u8);
    fn read_save_ram(&self, addr: u16) -> u8;
    fn write_save_ram(&mut self, addr: u16, data: u8);
    fn backup(&self);
}

pub struct Mapper0 {
    pub cart: Cartridge,
}

impl Mapper0 {
    pub fn new() -> Self {
        Mapper0 { cart: Cartridge::empty() }
    }
}

impl Mapper for Mapper0 {
    fn write(&mut self, addr: u16, data: u8) {
        // do nothing
    }

    fn read_prog_rom(&self, mut addr: u16) -> u8 {
        if self.cart.prog_rom.len() == 0x4000 {
            addr &= 0x3fff;
        } else {
            addr &= 0x7fff;
        }
        self.cart.prog_rom[addr as usize]
    }

    fn mirroring(&self) -> Mirroring { self.cart.mirroring }
    fn read_char_rom(&self, addr: u16) -> u8 { self.cart.char_rom[addr as usize] }
    fn write_char_rom(&mut self, addr: u16, data: u8) {}
    fn read_save_ram(&self, addr: u16) -> u8 { 0 }
    fn write_save_ram(&mut self, addr: u16, data: u8) {}
    fn backup(&self) {}
}






pub struct Mapper1 {
    pub cart: Cartridge,
    save_ram: [u8; 8192],
    shift_register: u8,
    shift_count: u8,
    control: u8,
    char_bank0: u8,
    char_bank1: u8,
    prog_bank: u8,
}

impl Mapper1 {
    pub fn new() -> Self {
        let ram = &std::fs::read("save/DQ3.dat").unwrap();
        let mut array_u8: [u8; 8192] = [0; 8192];
        array_u8[..ram.len()].copy_from_slice(&ram);
        Mapper1 { cart: Cartridge::empty(), save_ram: array_u8, shift_register: 0x10, shift_count: 0, control: 0x0c, char_bank0: 0, char_bank1: 0, prog_bank: 0 }
        //Mapper1 { cart: Cartridge::empty(), save_ram: [0xff; 8192], shift_register: 0x10, shift_count: 0, control: 0x0c, char_bank0: 0, char_bank1: 0, prog_bank: 0 }
    }

    fn reset(&mut self) {
        self.shift_register = 0x10;
        self.shift_count = 0;
    }

    fn char_rom_addr(&self, addr: u16) -> usize {
        addr as usize
    }
}

impl Mapper for Mapper1 {
    fn write(&mut self, addr: u16, data: u8) {
        if (data & 0x80) != 0 {
            self.reset();
            return;
        }
        self.shift_register = (self.shift_register >> 1) + ((data & 0x01) << 4);
        self.shift_count += 1;
        if self.shift_count == 5 {
            match addr {
                0x8000..=0x9fff => self.control = self.shift_register,
                0xa000..=0xbfff => self.char_bank0 = self.shift_register,
                0xc000..=0xdfff => self.char_bank1 = self.shift_register,
                0xe000..=0xffff => self.prog_bank = self.shift_register,
                _ => panic!("can't be here"),
            }
            self.reset();
        }
    }

    fn mirroring(&self) -> Mirroring {
        match self.control & 0x03 {
            2 => Mirroring::VERTICAL,
            3 => Mirroring::HORIZONTAL,
            _ => panic!("not supported mirroring mode"),
        }
    }

    fn read_prog_rom(&self, addr: u16) -> u8 {
        let bank_len = 1024 * 16;
        let bank_max = self.cart.prog_rom.len() / bank_len;
        let mut bank = self.prog_bank & 0x0f;
        let mut first_bank = 0;
        let mut last_bank = bank_max - 1;
        if self.char_bank0 & 0x10 != 0 {
            bank = bank | 0x10;
            first_bank = 0x10;
            last_bank |= 0x10;
        } else {
            bank = bank & 0x0f;
            first_bank = 0;
            last_bank = last_bank & 0x0f;
        }
        match (self.control & 0x0c) >> 2 {
            0 | 1 => {
                bank = bank & 0x1e;
                self.cart.prog_rom[(addr - 0x8000) as usize + bank_len * bank as usize]
            }
            2 => {
                match addr {
                    0x8000..=0xbfff => self.cart.prog_rom[(addr - 0x8000) as usize + bank_len * first_bank],
                    0xc000..=0xffff => {
                        self.cart.prog_rom[(addr - 0xc000) as usize + bank_len * bank as usize]
                    }
                    _ => panic!("can't be here"),
                }
            }
            3 => {
                match addr {
                    0x8000..=0xbfff => {
                        self.cart.prog_rom[(addr - 0x8000) as usize + bank_len * bank as usize]
                    }
                    0xc000..=0xffff => self.cart.prog_rom[(addr - 0xc000) as usize + bank_len * last_bank],
                    _ => panic!("can't be here"),
                }
            }
            _ => panic!("can't be here"),
        }
    }

    fn read_char_rom(&self, addr: u16) -> u8 {
        self.cart.char_rom[self.char_rom_addr(addr)]
    }

    fn write_char_rom(&mut self, addr: u16, data: u8) {
        let addr2 = self.char_rom_addr(addr);
        self.cart.char_rom[addr2] = data;
    }

    fn read_save_ram(&self, addr: u16) -> u8 {
        self.save_ram[addr as usize - 0x6000]
    }

    fn write_save_ram(&mut self, addr: u16, data: u8) {
        self.save_ram[addr as usize - 0x6000] = data;
    }

    fn backup(&self) {
        let mut file = File::create("save/DQ3.dat").unwrap();
        file.write_all(&self.save_ram).unwrap();
        file.flush().unwrap();
    }
}





pub struct Mapper2 {
    pub cart: Cartridge,
    bank_select: u8,
}

impl Mapper2 {
    pub fn new() -> Self {
        Mapper2 { cart: Cartridge::empty(), bank_select: 0 }
    }
}

impl Mapper for Mapper2 {
    fn write(&mut self, addr: u16, data: u8) {
        self.bank_select = data & 0x0f;
    }

    fn mirroring(&self) -> Mirroring { self.cart.mirroring }

    fn read_prog_rom(&self, addr: u16) -> u8 {
        let bank_len = 1024 * 16;
        let bank_max = self.cart.prog_rom.len() / bank_len;
        match addr {
            0x8000..=0xbfff => {
                self.cart.prog_rom[(addr - 0x8000) as usize + bank_len * self.bank_select as usize]
            }
            0xc000..=0xffff => {
                self.cart.prog_rom[(addr - 0xc000) as usize + bank_len * (bank_max - 1)]
            }
            _ => panic!("can't be here"),
        }
    }

    fn read_char_rom(&self, addr: u16) -> u8 { self.cart.char_rom[addr as usize] }

    fn write_char_rom(&mut self, addr: u16, data: u8) {
        self.cart.char_rom[addr as usize] = data;
    }

    fn read_save_ram(&self, addr: u16) -> u8 {
        0
    }

    fn write_save_ram(&mut self, addr: u16, data: u8) {

    }

    fn backup(&self) {

    }
}
