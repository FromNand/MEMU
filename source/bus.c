#include "common.h"

#define between(start, address, end) (start <= address && address <= end)

unsigned char internal_ram[0x800];
ROM *rom;
unsigned int cpu_cycle;

ROM *load_rom(char *file_name);

void init_bus(char *file_name) {
    rom = load_rom(file_name);
}

unsigned char bus_read8(unsigned short address) {
    if(between(0x0000, address, 0x1fff)) {
        return internal_ram[address & 0x07ff];
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
        internal_ram[address & 0x07ff] = value;
    } else if(between(0x8000, address, 0xffff)) {
        if(rom->program_rom_size == 0x4000) {
            rom->program_rom[address & 0x3fff] = value;
        } else {
            rom->program_rom[address - 0x8000] = value;
        }
    } else {
        error("Unsupported bus write 0x%04X\n", address);
    }
}

unsigned int get_cpu_cycle(void) {
    return cpu_cycle;
}

void cpu_tick(unsigned int cycle) {
    cpu_cycle += cycle;
}
