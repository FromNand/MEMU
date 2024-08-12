#include "common.h"

#define between(start, address, end) (start <= address && address <= end)

unsigned int cpu_cycle, ppu_cycle;
ROM *rom;
unsigned char internal_ram[0x800];

extern unsigned char oam_data[256];
ROM *load_rom(char *file_name);
void write_oam_data(unsigned char value);

void tick(unsigned int cycle) {
    cpu_cycle += cycle;
    ppu_cycle += cycle * 3;
}

unsigned int get_cpu_cycle(void) {
    return cpu_cycle;
}

unsigned int get_ppu_cycle(void) {
    return ppu_cycle;
}

void init_bus(char *file_name) {
    rom = load_rom(file_name);
}

unsigned char bus_read8(unsigned short address) {
    if(between(0x0000, address, 0x1fff)) {
        return internal_ram[address & 0x7ff];
    } else if(between(0x2008, address, 0x3fff)) {
        return bus_read8(address & 0x2007);
    } else if(between(0x8000, address, 0xffff)) {
        if(rom->program_rom_size == 0x4000) {
            return rom->program_rom[address & 0x3fff];
        } else {
            return rom->program_rom[address - 0x8000];
        }
    } else {
        error("Unsupported bus read 0x%04X\n", address);
    }
}

void bus_write8(unsigned short address, unsigned char value) {
    if(between(0x0000, address, 0x1fff)) {
        internal_ram[address & 0x7ff] = value;
    } else if(between(0x2008, address, 0x3fff)) {
        bus_write8(address & 0x2007, value);
    } else if(address == 0x4014) {
        for(int i = 0; i < 256; i++) {
            write_oam_data(bus_read8((value << 8) + i));
        }
        tick(513);
    } else {
        error("Unsupported bus write 0x%04X\n", address);
    }
}
