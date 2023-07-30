// pub struct Joypad {
//     strobe: bool,
//     index: u8,
//     pub status: u8,
// }

// impl Joypad {
//     pub fn new() -> Self {
//         Joypad {
//             strobe: false,
//             index: 0,
//             status: 0,
//         }
//     }

    // pub fn read(&mut self) -> u8 {
    //     if self.index > 7 {
    //         return 1;
    //     }
    //     let value = (self.status & (1 << self.index)) >> self.index;
    //     if !self.strobe && self.index <= 7{
    //         self.index += 1;
    //     }
    //     value
    // }

    // pub fn write(&mut self, data: u8) {
    //     if data & 0x01 != 0 {
    //         self.strobe = true;
    //         self.index = 0;
    //     }
    // }
// }

use bitflags::bitflags;

bitflags! {
    // https://wiki.nesdev.com/w/index.php/Controller_reading_code
    #[derive(Clone, Copy)]
    pub struct JoypadButton: u8 {
        const RIGHT             = 0b10000000;
        const LEFT              = 0b01000000;
        const DOWN              = 0b00100000;
        const UP                = 0b00010000;
        const START             = 0b00001000;
        const SELECT            = 0b00000100;
        const BUTTON_B          = 0b00000010;
        const BUTTON_A          = 0b00000001;
    }
}

pub struct Joypad {
    strobe: bool,
    button_index: u8,
    button_status: JoypadButton,
}

impl Joypad {
    pub fn new() -> Self {
        Joypad {
            strobe: false,
            button_index: 0,
            button_status: JoypadButton::from_bits_truncate(0),
        }
    }

    pub fn write(&mut self, data: u8) {
        self.strobe = data & 1 == 1;
        if self.strobe {
            self.button_index = 0
        }
    }

    pub fn read(&mut self) -> u8 {
        if self.button_index > 7 {
            return 1;
        }
        let response = (self.button_status.bits() & (1 << self.button_index)) >> self.button_index;
        if !self.strobe && self.button_index <= 7 {
            self.button_index += 1;
        }
        response
    }

    pub fn set_button_pressed_status(&mut self, button: JoypadButton, value: bool) {
        self.button_status.set(button, value)
    }
}
