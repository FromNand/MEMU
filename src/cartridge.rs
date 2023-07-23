use core::panic;

enum Mirroring {
    HORIZONTAL,
    VERTICAL,
    FOUR_SCREEN,
}

pub struct Cartridge {
    mapper: usize,
    pub prog_rom: Vec<u8>,
    char_rom: Vec<u8>,
    mirroring: Mirroring,
}

impl Cartridge {
    pub fn new(game: &Vec<u8>) -> Self {
        if &game[0..4] != [0x4e, 0x45, 0x53, 0x1a] {
            panic!("Not in iNES format");
        }
        if game[7] & 0x0c != 0 {
            panic!("Only iNES format is supported");
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
            char_rom: game[char_rom_start..(char_rom_start + char_rom_size)].to_vec(),
            mirroring,
        }
    }
}
