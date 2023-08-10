use crate::nes::ppu::Mirroring;

pub struct Cartridge {
    pub mapper: usize,
    pub prog_rom: Vec<u8>,
    pub char_rom: Vec<u8>,
    pub mirroring: Mirroring,
    pub is_char_ram: bool,
}

impl Cartridge {
    pub fn new(game: &Vec<u8>) -> Self {
        if &game[0..4] != [0x4e, 0x45, 0x53, 0x1a] || game[7] & 0x0f != 0 {
            panic!("Only iNES1.0 format is supported");
        }
        let mapper = (game[6] >> 4) + (game[7] & 0xf0);
        let prog_rom_size = 1024 * 16 * game[4] as usize;
        let char_rom_size = 1024 * 8 * game[5] as usize;
        let prog_rom_start = 16 + if game[6] & 0x04 != 0 { 512 } else { 0 };
        let char_rom_start = prog_rom_start + prog_rom_size;
        let mirroring = match (game[6] & 0x08 != 0, game[6] & 0x01 != 0) {
            (false, false) => Mirroring::HORIZONTAL,
            (false, true) => Mirroring::VERTICAL,
            (true, _) => Mirroring::FOUR_SCREEN,
        };
        Cartridge {
            mapper: mapper as usize,
            prog_rom: game[prog_rom_start..(prog_rom_start + prog_rom_size)].to_vec(),
            char_rom: if char_rom_size == 0 {
                let blank_char_rom: Vec<u8> = vec![0; 1024 * 8];
                blank_char_rom
            } else {
                game[char_rom_start..(char_rom_start + char_rom_size)].to_vec()
            },
            mirroring,
            is_char_ram: char_rom_size == 0,
        }
    }

    pub fn empty() -> Self {
        Cartridge { mapper: 0, prog_rom: vec![], char_rom: vec![], mirroring: Mirroring::VERTICAL, is_char_ram: false }
    }
}
