use crate::Cartridge;
use crate::ppu::PPU;
use crate::joypad::Joypad;

pub struct Bus<'call> {
    work_ram: [u8; 0x0800],
    prog_rom: Vec<u8>,
    pub ppu: PPU,
    joypad1: Joypad,
    callback: Box<dyn FnMut(&PPU, &mut Joypad) + 'call>,
}

impl<'a> Bus<'a> {
    pub fn new<'call, F: FnMut(&PPU, &mut Joypad) + 'call>(cart: Cartridge, callback: F) -> Bus<'call> {
        Bus {
            work_ram: [0; 0x0800],
            prog_rom: cart.prog_rom,
            ppu: PPU::new(cart.mirroring, cart.char_rom),
            joypad1: Joypad::new(),
            callback: Box::from(callback),
        }
    }

    pub fn read8(&mut self, mut addr: u16) -> u8 {
        match addr {
            0x0000..=0x1fff => self.work_ram[(addr & 0x07ff) as usize],
            0x2002 => self.ppu.read_ppustts(),
            0x2004 => self.ppu.read_oam_data(),
            0x2007 => self.ppu.read(),
            0x2008..=0x3fff => self.read8(addr & 0x2007),
            0x4016 => self.joypad1.read(),
            0x2000 | 0x2001 | 0x2003 | 0x2005 | 0x2006 | 0x4000 | 0x4010 | 0x4011 | 0x4014 | 0x4015 | 0x4017 => 0,
            0x8000..=0xffff => {
                if self.prog_rom.len() == 0x4000 {
                    addr &= 0x3fff;
                } else {
                    addr &= 0x7fff;
                }
                self.prog_rom[addr as usize]
            }
            _ => todo!("Read from 0x{:04X} in CPU", addr),
        }
    }

    pub fn write8(&mut self, addr: u16, data: u8) {
        match addr {
            0x0000..=0x1fff => self.work_ram[(addr & 0x07ff) as usize] = data,
            0x2000 => self.ppu.write_to_ppuctrl(data),
            0x2001 => self.ppu.write_to_ppumask(data),
            0x2003 => self.ppu.write_to_oam_addr(data),
            0x2004 => self.ppu.write_to_oam_data(data),
            0x2005 => self.ppu.write_to_ppuscrl(data),
            0x2006 => self.ppu.write_to_ppuaddr(data),
            0x2007 => self.ppu.write(data),
            0x2008..=0x3fff => self.write8(addr & 0x2007, data),
            0x4014 => {
                let mut buffer: [u8; 256] = [0; 256];
                for i in 0..=255 {
                    buffer[i] = self.read8(((data as u16) << 8) + i as u16);
                }
                self.ppu.write_to_oam_dma(buffer);
            }
            0x4016 => self.joypad1.write(data),
            0x2005 | 0x4000..=0x4013 | 0x4015..=0x4017 => {},
            _ => todo!("Write to 0x{:04X} in CPU", addr),
        }
    }

    pub fn tick(&mut self, cycle: usize) {
        let old_nmi = self.ppu.nmi;
        self.ppu.tick(cycle * 3);
        if !old_nmi && self.ppu.nmi {
            (self.callback)(&self.ppu, &mut self.joypad1);
        }
    }
}
