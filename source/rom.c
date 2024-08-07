#include "common.h"
#include <stdio.h>
#include <stdlib.h>

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
    rom->program_rom_size = 1024 * 16 * rom->rom[4];
    rom->character_rom_size = 1024 * 8 * rom->rom[5];
    rom->mirroring = rom->rom[6] & 0x01;
    rom->program_rom = rom->rom + 16 + ((rom->rom[6] & 0x04) ? 512 : 0);
    rom->character_rom = rom->program_rom + rom->program_rom_size;

    return rom;
}
