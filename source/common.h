#ifndef _COMMON_H
#define _COMMON_H

#include <stdbool.h>

#define SCREEN_BLOCK_WIDTH (256)
#define SCREEN_BLOCK_HEIGHT (240)
#define BYTE_PER_PIXEL (4)
#define BLOCK_PIXEL_SIZE (3)
#define SCREEN_PIXEL_WIDTH (BLOCK_PIXEL_SIZE * SCREEN_BLOCK_WIDTH)
#define SCREEN_PIXEL_HEIGHT (BLOCK_PIXEL_SIZE * SCREEN_BLOCK_HEIGHT)

typedef struct {
    unsigned char *rom;
    unsigned char *program_rom;
    unsigned int program_rom_size;
    unsigned char *character_rom;
    unsigned int character_rom_size;
    bool has_character_ram;
    unsigned int mirroring;
    unsigned int mapper;
} ROM;

void error(char *message, ...);

#endif
