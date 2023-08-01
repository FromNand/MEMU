pub struct Mapper2 {
    pub prog_rom: Vec<u8>,
    bank_select: u8,
}

impl Mapper2 {
    pub fn new() -> Self {
        Mapper2 { prog_rom: vec![], bank_select: 0 }
    }

    pub fn write(&mut self, addr: u16, data: u8) {
        self.bank_select = data & 0x0f;
    }

    pub fn read_prog_rom(&self, addr: u16) -> u8 {
        let bank_len = 1024 * 16;
        let bank_max = self.prog_rom.len() / bank_len;
        match addr {
            0x8000..=0xbfff => {
                self.prog_rom[(addr - 0x8000) as usize + bank_len * self.bank_select as usize]
            }
            0xc000..=0xffff => {
                self.prog_rom[(addr - 0xc000) as usize + bank_len * (bank_max - 1)]
            }
            _ => panic!("can't be here"),
        }
    }
}
