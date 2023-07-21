pub struct Bus {
    work_ram: [u8; 0x0800],
}

impl Bus {
    pub fn new() -> Self {
        Bus {
            work_ram: [0; 0x0800],
        }
    }

    pub fn read8(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x1fff => self.work_ram[(addr & 0x07ff) as usize],
            _ => todo!("Read from 0x{:04X}", addr),
        }
    }

    pub fn write8(&mut self, addr: u16, data: u8) {
        match addr {
            0x0000..=0x1fff => {
                self.work_ram[(addr & 0x07ff) as usize] = data;
            }
            _ => todo!("Write 0x{:02X} to 0x{:04X}", data, addr),
        }
    }
}
