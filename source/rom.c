#include "common.h"
#include <stdio.h>
#include <stdlib.h>

extern ROM *rom;

void (*init_bank)(void);
unsigned char (*read_bank1)(unsigned short address);
unsigned char (*read_bank2)(unsigned short address);
void (*write_bank)(unsigned short address, unsigned char value);

unsigned char *low_bank;
unsigned char *high_bank;

// マッパー0

void mapper0_init_bank(void) {
    low_bank = high_bank = rom->program_rom;
    if(rom->program_rom_size != 0x4000) {
        high_bank = rom->program_rom + 0x4000;
    }
}

unsigned char mapper0_read_low_bank(unsigned short address) {
    return low_bank[address - 0x8000];
}

unsigned char mapper0_read_high_bank(unsigned short address) {
    return high_bank[address - 0xc000];
}

void mapper0_write_bank(unsigned short address, unsigned char value) {

}

// マッパー2
// 0x8000-0xbfff 16KBの切り替え可能なPRG-ROMバンク
// 0xc000-0xffff 16KBの最後のPRG-ROMバンク
// 書き込みの下位3ビットでバンク選択を行う

void mapper2_init_bank(void) {
    low_bank = rom->program_rom;
    high_bank = rom->program_rom + 0x4000 * 7;
}

unsigned char mapper2_read_low_bank(unsigned short address) {
    return low_bank[address - 0x8000];
}

unsigned char mapper2_read_high_bank(unsigned short address) {
    return high_bank[address - 0xc000];
}

void mapper2_write_bank(unsigned short address, unsigned char value) {
    value &= 0x0f;
    int bank_max = rom->program_rom_size / 0x4000;
    if(bank_max <= value) {
        value = bank_max - 1;
    }
    low_bank = rom->program_rom + 0x4000 * value;
}

ROM *load_rom(char *file_name) {
    FILE *fp = fopen(file_name, "rb");
    if(fp == NULL) {
        error("Cannot open %s\n", file_name);
    }

    fseek(fp, 0, SEEK_END);
    int file_size = ftell(fp);
    fseek(fp, 0, SEEK_SET);

    ROM *rom = malloc(sizeof(ROM));
    rom->rom = malloc(file_size);
    fread(rom->rom, 1, file_size, fp);
    fclose(fp);

    if(rom->rom[0] != 'N' || rom->rom[1] != 'E' || rom->rom[2] != 'S' || rom->rom[3] != 0x1a) {
        error("Cannot find iNES signature\n");
    }
    rom->program_rom = rom->rom + 16 + (((rom->rom[6] & 0x04) != 0) ? 512 : 0);
    rom->program_rom_size = 1024 * 16 * rom->rom[4];
    rom->character_rom = rom->program_rom + rom->program_rom_size;
    rom->character_rom_size = 1024 * 8 * rom->rom[5];
    rom->has_character_ram = rom->character_rom_size == 0;
    rom->mirroring = (rom->rom[6] & 0x01) + ((rom->rom[6] & 0x08) >> 2);
    rom->mapper = (rom->rom[6] >> 4) + (rom->rom[7] & 0xf0);

    if(rom->has_character_ram) {
        static unsigned char character_ram[1024 * 8];
        rom->character_rom = character_ram;
    }

    if(rom->mapper == 0) {
        init_bank = mapper0_init_bank;
        read_bank1 = mapper0_read_low_bank;
        read_bank2 = mapper0_read_high_bank;
        write_bank = mapper0_write_bank;
    } else if(rom->mapper == 2) {
        init_bank = mapper2_init_bank;
        read_bank1 = mapper2_read_low_bank;
        read_bank2 = mapper2_read_high_bank;
        write_bank = mapper2_write_bank;
    } else {
        error("Unsupported mapper %d\n", rom->mapper);
    }

    return rom;
}
