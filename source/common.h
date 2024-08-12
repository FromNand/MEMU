#ifndef _COMMON_H
#define _COMMON_H

#define SCREEN_WIDTH (256)
#define SCREEN_HEIGHT (240)
#define BYTE_PER_PIXEL (4)

typedef struct {
    unsigned char *rom;
    unsigned char *program_rom;
    unsigned int program_rom_size;
    unsigned char *character_rom;
    unsigned int character_rom_size;
    unsigned int mirroring;
    unsigned int mapper;
} ROM;

void error(char *message, ...);

#endif
