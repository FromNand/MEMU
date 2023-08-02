use crate::Cartridge;
use crate::ppu::PPU;
use crate::apu::APU;
use crate::joypad::Joypad;
use crate::MAPPER;

pub struct Bus<'call> {
    work_ram: [u8; 0x0800],
    //prog_rom: Vec<u8>,
    pub ppu: PPU,
    apu: APU,
    joypad1: Joypad,
    callback: Box<dyn FnMut(&PPU, &mut Joypad) + 'call>,
}

impl<'a> Bus<'a> {
    pub fn new<'call, F: FnMut(&PPU, &mut Joypad) + 'call>(sdl_context: &sdl2::Sdl, cart: Cartridge, callback: F) -> Bus<'call> {
        Bus {
            work_ram: [0; 0x0800],
            //prog_rom: cart.prog_rom,
            ppu: PPU::new(cart.mirroring, cart.is_char_ram, cart.char_rom),
            apu: APU::new(sdl_context),
            joypad1: Joypad::new(),
            callback: Box::from(callback),
        }
    }

    pub fn read8(&mut self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x1fff => self.work_ram[(addr & 0x07ff) as usize],
            0x2002 => self.ppu.read_ppustts(),
            0x2004 => self.ppu.read_oam_data(),
            0x2007 => self.ppu.read(),
            0x2008..=0x3fff => self.read8(addr & 0x2007),
            0x4015 => self.apu.read_status(),
            0x4016 => self.joypad1.read(),
            0x4017 => 0,
            0x8000..=0xffff => {
                MAPPER.lock().unwrap().read_prog_rom(addr)
                // if self.prog_rom.len() == 0x4000 {
                //     addr &= 0x3fff;
                // } else {
                //     addr &= 0x7fff;
                // }
                // self.prog_rom[addr as usize]
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
            0x4000..=0x4003 | 0x4004..=0x4007 | 0x4008 | 0x400a | 0x400b | 0x400c | 0x400e | 0x400f => self.apu.write(addr, data),
            0x4014 => {
                let mut buffer: [u8; 256] = [0; 256];
                for i in 0..=255 {
                    buffer[i] = self.read8(((data as u16) << 8) + i as u16);
                }
                self.ppu.write_to_oam_dma(buffer);
                for _ in 0..513 {
                    // fixme
                    self.ppu.tick(3);
                }
            }
            0x4015 => self.apu.write_status(data),
            0x4016 => self.joypad1.write(data),
            0x4017 => self.apu.write_frame_counter(data),
            0x8000..=0xffff => {
                MAPPER.lock().unwrap().write(addr, data);
            }
            0x4010 | 0x4011 => {},
            _ => todo!("Write to 0x{:04X} in CPU", addr),
        }
    }

    pub fn poll_apu_irq(&self) -> bool {
        self.apu.irq()
    }

    pub fn tick(&mut self, cycle: usize) {
        let old_nmi = self.ppu.nmi;
        self.ppu.tick(cycle * 3);
        self.apu.tick(cycle);
        if !old_nmi && self.ppu.nmi {
            (self.callback)(&self.ppu, &mut self.joypad1);
        }
    }
}
