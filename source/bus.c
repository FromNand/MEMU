#include "common.h"

#define between(start, address, end) (start <= address && address <= end)

ROM *rom;
unsigned char internal_ram[0x800];

extern unsigned int cpu_cycle;
ROM *load_rom(char *file_name);
void tick_ppu(unsigned int cycle);
void init_ppu(void);
void init_apu(void);
void write_ppu_control(unsigned char value);
void write_ppu_mask(unsigned char value);
unsigned char read_ppu_status(void);
void write_oam_address(unsigned char value);
unsigned char read_oam_data(void);
void write_oam_data(unsigned char value);
void write_ppu_scroll(unsigned char value);
void write_ppu_address(unsigned char value);
unsigned char read_ppu_data(void);
void write_ppu_data(unsigned char value);
unsigned char read_joypad(void);
void write_joypad(unsigned char value);
void write_square1(unsigned short address, unsigned char value);
void write_square2(unsigned short address, unsigned char value);
void write_triangle(unsigned short address, unsigned char value);
void write_noise(unsigned short address, unsigned char value);

extern void (*init_bank)(void);
extern unsigned char (*read_bank1)(unsigned short address);
extern unsigned char (*read_bank2)(unsigned short address);
extern void (*write_bank)(unsigned short address, unsigned char value);

void tick(unsigned int cycle) {
    cpu_cycle += cycle;
    tick_ppu(cycle * 3);
}

void init_bus(char *file_name) {
    rom = load_rom(file_name);
    init_bank();
    init_ppu();
    init_apu();
}

unsigned char bus_read8(unsigned short address) {
    if(between(0x0000, address, 0x1fff)) {
        return internal_ram[address & 0x7ff];
    } else if(address == 0x2002) {
        return read_ppu_status();
    } else if(address == 0x2004) {
        return read_oam_data();
    } else if(address == 0x2007) {
        return read_ppu_data();
    } else if(between(0x2008, address, 0x3fff)) {
        return bus_read8(address & 0x2007);
    } else if(address == 0x4016) {
        return read_joypad();
    } else if(between(0x8000, address, 0xbfff)) {
        return read_bank1(address);
    } else if(between(0xc000, address, 0xffff)) {
        return read_bank2(address);
    } else if(address == 0x4017) {
        return 0;
    } else {
        error("Unsupported bus read 0x%04X\n", address);
    }
}

void bus_write8(unsigned short address, unsigned char value) {
    if(between(0x0000, address, 0x1fff)) {
        internal_ram[address & 0x7ff] = value;
    } else if(address == 0x2000) {
        write_ppu_control(value);
    } else if(address == 0x2001) {
        write_ppu_mask(value);
    } else if(address == 0x2003) {
        write_oam_address(value);
    } else if(address == 0x2004) {
        write_oam_data(value);
    } else if(address == 0x2005) {
        write_ppu_scroll(value);
    } else if(address == 0x2006) {
        write_ppu_address(value);
    } else if(address == 0x2007) {
        write_ppu_data(value);
    } else if(between(0x2008, address, 0x3fff)) {
        bus_write8(address & 0x2007, value);
    } else if(between(0x4000, address, 0x4003)) {
        write_square1(address, value);
    } else if(between(0x4004, address, 0x4007)) {
        write_square2(address, value);
    } else if(between(0x4008, address, 0x400b)) {
        write_triangle(address, value);
    } else if(between(0x400c, address, 0x400f)) {
        write_noise(address, value);
    } else if(address == 0x4014) {
        tick((cpu_cycle % 2 == 0) ? 1 : 2);
        for(int i = 0; i < 256; i++) {
            write_oam_data(bus_read8((value << 8) + i));
            tick(2);
        }
    } else if(address == 0x4016) {
        write_joypad(value);
    } else if(between(0x8000, address, 0xffff)) {
        write_bank(address, value);
    } else if(address == 0x4010 || address == 0x4011 || address == 0x4015 || address == 0x4017) {

    } else {
        error("Unsupported bus write 0x%04X\n", address);
    }
}
