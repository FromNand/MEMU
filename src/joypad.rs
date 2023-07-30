pub struct Joypad {
    strobe: bool,
    index: u8,
    pub status: u8,
}

impl Joypad {
    pub fn new() -> Self {
        Joypad {
            strobe: false,
            index: 0,
            status: 0,
        }
    }

    pub fn read(&mut self) -> u8 {
        if self.index > 7 {
            return 1;
        }
        let value = (self.status & (1 << self.index)) >> self.index;
        if !self.strobe && self.index <= 7{
            self.index += 1;
        }
        value
    }

    pub fn write(&mut self, data: u8) {
        self.strobe = data & 0x01 != 0;
        if self.strobe {
            self.index = 0;
        }
    }
}
