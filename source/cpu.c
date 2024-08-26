#include "common.h"
#include <gtk-3.0/gtk/gtk.h>
#include <stdbool.h>

// #define NESTEST

unsigned int cpu_cycle;
extern unsigned int ppu_cycle, scanline;

typedef enum {
    IMP, ACC, IMM, ZPG, ZPX, ZPY, ABS, ABX, ABY, IND, INX, INY, REL
} Addressing_Mode;

typedef enum {
    None, Page, Branch
} Cycle_Mode;

typedef struct {
    unsigned char opcode;
    char *mnemonic;
    void (*function)(void);
    Addressing_Mode addressing_mode;
    unsigned short length;
    unsigned int cycle;
    Cycle_Mode cycle_mode;
} Instruction;

Instruction instruction[227];
Instruction *instruction_table[256];

typedef struct {
    bool c;
    bool z;
    bool i;
    bool d;
    bool v;
    bool n;
} Flag;

typedef struct {
    unsigned char a, x, y, s;
    unsigned short pc;
    Flag p;
    unsigned short address;
    unsigned int extra_cycle;
    Cycle_Mode cycle_mode;
} CPU;

CPU cpu;

void tick(unsigned int cycle);
void init_bus(char *file_name);
unsigned char bus_read8(unsigned short address);
void bus_write8(unsigned short address, unsigned char value);

void set_flag(unsigned char value) {
    cpu.p.c = (value & 0x01) != 0;
    cpu.p.z = (value & 0x02) != 0;
    cpu.p.i = (value & 0x04) != 0;
    cpu.p.d = (value & 0x08) != 0;
    cpu.p.v = (value & 0x40) != 0;
    cpu.p.n = (value & 0x80) != 0;
}

unsigned char get_flag(void) {
    unsigned char value = 0x20;
    if(cpu.p.c) value |= 0x01;
    if(cpu.p.z) value |= 0x02;
    if(cpu.p.i) value |= 0x04;
    if(cpu.p.d) value |= 0x08;
    if(cpu.p.v) value |= 0x40;
    if(cpu.p.n) value |= 0x80;
    return value;
}

unsigned char read8(unsigned short address) {
    return bus_read8(address);
}

unsigned short read16(unsigned short address) {
    return read8(address) + (read8(address + 1) << 8);
}

unsigned short bug_read16(unsigned short address) {
    if((address & 0xff) == 0xff) {
        return read8(address) + (read8(address & 0xff00) << 8);
    } else {
        return read16(address);
    }
}

void write8(unsigned short address, unsigned char value) {
    bus_write8(address, value);
}

void push8(unsigned char value) {
    write8(0x100 + cpu.s--, value);
}

void push16(unsigned short value) {
    push8(value >> 8);
    push8(value);
}

unsigned char pop8(void) {
    return read8(0x100 + ++cpu.s);
}

unsigned short pop16(void) {
    unsigned char value = pop8();
    return value + (pop8() << 8);
}

unsigned short get_address(Addressing_Mode addressing_mode) {
    unsigned short address = cpu.pc + 1;
    unsigned char argument8 = read8(address);
    unsigned short argument16 = read16(address);
    switch(addressing_mode) {
        case IMP:
        case ACC:
        case IMM:
            break;
        case ZPG:
            address = argument8;
            break;
        case ZPX:
            address = (unsigned char)(argument8 + cpu.x);
            break;
        case ZPY:
            address = (unsigned char)(argument8 + cpu.y);
            break;
        case ABS:
            address = argument16;
            break;
        case ABX:
            address = argument16 + cpu.x;
            if(cpu.cycle_mode == Page && (argument16 & 0xff00) != (address & 0xff00)) {
                cpu.extra_cycle += 1;
            }
            break;
        case ABY:
            address = argument16 + cpu.y;
            if(cpu.cycle_mode == Page && (argument16 & 0xff00) != (address & 0xff00)) {
                cpu.extra_cycle += 1;
            }
            break;
        case IND:
            address = bug_read16(argument16);
            break;
        case INX:
            address = bug_read16((unsigned char)(argument8 + cpu.x));
            break;
        case INY:
            unsigned short address2 = bug_read16(argument8);
            address = address2 + cpu.y;
            if(cpu.cycle_mode == Page && (address2 & 0xff00) != (address & 0xff00)) {
                cpu.extra_cycle += 1;
            }
            break;
        case REL:
            address = (char)argument8;
            break;
        default:
            error("Unknown addressing mode\n");
    }
    return address;
}

void _log(Instruction *i) {
    static int count;
    static FILE *fp;
    if(count++ == 0) {
        fp = fopen("./rom/memu.log", "wb");
        if(fp == NULL) {
            error("Cannot open ./rom/memu.log\n");
        }
        tick(7);
    } else if(count == 8992) {
        fclose(fp);
        error("Log finished\n");
    }

    fprintf(fp, "%04X  ", cpu.pc);
    unsigned char data[3];
    for(int index = 0; index < i->length; index++) {
        data[index] = read8(cpu.pc + index);
        fprintf(fp, "%02X ", data[index]);
    }
    fprintf(fp, "%*s", 3 * (3 - i->length), "");

    char s[256];
    unsigned char value = read8(cpu.address);
    switch(i->addressing_mode) {
        case IMP:
            s[0] = '\0';
            break;
        case ACC:
            sprintf(s, "A");
            break;
        case IMM:
            sprintf(s, "#$%02X", value);
            break;
        case ZPG:
            sprintf(s, "$%02X = %02X", cpu.address, value);
            break;
        case ZPX:
            sprintf(s, "$%02X,X @ %02X = %02X", data[1], cpu.address, value);
            break;
        case ZPY:
            sprintf(s, "$%02X,Y @ %02X = %02X", data[1], cpu.address, value);
            break;
        case ABS:
            if(i->mnemonic[0] == 'J') {
                sprintf(s, "$%04X", cpu.address);
            } else {
                sprintf(s, "$%04X = %02X", cpu.address, value);
            }
            break;
        case ABX:
            sprintf(s, "$%02X%02X,X @ %04X = %02X", data[2], data[1], cpu.address, value);
            break;
        case ABY:
            sprintf(s, "$%02X%02X,Y @ %04X = %02X", data[2], data[1], cpu.address, value);
            break;
        case IND:
            sprintf(s, "($%02X%02X) = %04X", data[2], data[1], cpu.address);
            break;
        case INX:
            sprintf(s, "($%02X,X) @ %02X = %04X = %02X", data[1], (unsigned char)(data[1] + cpu.x), cpu.address, value);
            break;
        case INY:
            sprintf(s, "($%02X),Y = %04X @ %04X = %02X", data[1], bug_read16(data[1]), cpu.address, value);
            break;
        case REL:
            sprintf(s, "$%04X", (unsigned short)(cpu.pc + 2 + cpu.address));
            break;
        default:
            error("Unknown addressing mode\n");
    }
    fprintf(fp, "%4s %-28s", i->mnemonic, s);
    fprintf(fp, "A:%02X X:%02X Y:%02X P:%02X SP:%02X PPU:%3d,%3d CYC:%d\n", cpu.a, cpu.x, cpu.y, get_flag(), cpu.s, scanline, ppu_cycle, cpu_cycle);
}

void init_nes(char *file_name) {
    init_bus(file_name);
    cpu.a = cpu.x = cpu.y = 0;
    cpu.s = 0xfd;
    cpu.pc = read16(0xfffc);
    set_flag(0x04);
    for(int i = 0; i < sizeof(instruction) / sizeof(Instruction); i++) {
        instruction_table[instruction[i].opcode] = instruction + i;
    }
}

gboolean run_nes(gpointer data) {
    Instruction *i = instruction_table[read8(cpu.pc)];
    if(i == NULL) {
        error("Invalid opcode 0x%02X\n", read8(cpu.pc));
    }
    cpu.extra_cycle = 0;
    cpu.cycle_mode = i->cycle_mode;
    cpu.address = get_address(i->addressing_mode);
#ifdef NESTEST
    _log(i);
#endif
    i->function();
    cpu.pc += i->length;
    tick(i->cycle + cpu.extra_cycle);
    return G_SOURCE_CONTINUE;
}

void update_zn(unsigned char value) {
    cpu.p.z = value == 0;
    cpu.p.n = (value & 0x80) != 0;
}

void adc(void) {
    unsigned char a = cpu.a;
    unsigned char m = read8(cpu.address);
    unsigned short r = a + m + cpu.p.c;
    cpu.a = r;
    cpu.p.c = r > 0xff;
    cpu.p.v = ((a ^ r) & (m ^ r) & 0x80) != 0;
    update_zn(cpu.a);
}

void and(void) {
    cpu.a &= read8(cpu.address);
    update_zn(cpu.a);
}

void asl_acc(void) {
    cpu.p.c = (cpu.a & 0x80) != 0;
    cpu.a <<= 1;
    update_zn(cpu.a);
}

void asl(void) {
    unsigned char m = read8(cpu.address);
    cpu.p.c = (m & 0x80) != 0;
    write8(cpu.address, m << 1);
    update_zn(m << 1);
}

void branch(bool condition) {
    if(condition) {
        cpu.extra_cycle += 1;
        if(((cpu.pc + 2) & 0xff00) != ((cpu.pc + 2 + cpu.address) & 0xff00)) {
            cpu.extra_cycle += 1;
        }
        cpu.pc += cpu.address;
    }
}

void bcc(void) {
    branch(cpu.p.c == false);
}

void bcs(void) {
    branch(cpu.p.c == true);
}

void beq(void) {
    branch(cpu.p.z == true);
}

void bit(void) {
    unsigned char m = read8(cpu.address);
    cpu.p.z = (cpu.a & m) == 0;
    cpu.p.v = (m & 0x40) != 0;
    cpu.p.n = (m & 0x80) != 0;
}

void bmi(void) {
    branch(cpu.p.n == true);
}

void bne(void) {
    branch(cpu.p.z == false);
}

void bpl(void) {
    branch(cpu.p.n == false);
}

void _brk(void) {
    push16(cpu.pc + 2);
    push8(get_flag() | 0x10);
    cpu.pc = read16(0xfffe) - 1;
    cpu.p.i = true;
}

void bvc(void) {
    branch(cpu.p.v == false);
}

void bvs(void) {
    branch(cpu.p.v == true);
}

void clc(void) {
    cpu.p.c = false;
}

void cld(void) {
    cpu.p.d = false;
}

void cli(void) {
    cpu.p.i = false;
}

void clv(void) {
    cpu.p.v = false;
}

void compare(unsigned char r) {
    unsigned char m = read8(cpu.address);
    cpu.p.c = r >= m;
    update_zn(r - m);
}

void cmp(void) {
    compare(cpu.a);
}

void cpx(void) {
    compare(cpu.x);
}

void cpy(void) {
    compare(cpu.y);
}

void dec(void) {
    unsigned char m = read8(cpu.address);
    write8(cpu.address, m - 1);
    update_zn(m - 1);
}

void dex(void) {
    cpu.x -= 1;
    update_zn(cpu.x);
}

void dey(void) {
    cpu.y -= 1;
    update_zn(cpu.y);
}

void eor(void) {
    cpu.a ^= read8(cpu.address);
    update_zn(cpu.a);
}

void inc(void) {
    unsigned char m = read8(cpu.address);
    write8(cpu.address, m + 1);
    update_zn(m + 1);
}

void inx(void) {
    cpu.x += 1;
    update_zn(cpu.x);
}

void iny(void) {
    cpu.y += 1;
    update_zn(cpu.y);
}

void jmp(void) {
    cpu.pc = cpu.address - 3;
}

void jsr(void) {
    push16(cpu.pc + 2);
    cpu.pc = cpu.address - 3;
}

void lda(void) {
    cpu.a = read8(cpu.address);
    update_zn(cpu.a);
}

void ldx(void) {
    cpu.x = read8(cpu.address);
    update_zn(cpu.x);
}

void ldy(void) {
    cpu.y = read8(cpu.address);
    update_zn(cpu.y);
}

void lsr_acc(void) {
    cpu.p.c = (cpu.a & 0x01) != 0;
    cpu.a >>= 1;
    update_zn(cpu.a);
}

void lsr(void) {
    unsigned char m = read8(cpu.address);
    cpu.p.c = (m & 0x01) != 0;
    write8(cpu.address, m >> 1);
    update_zn(m >> 1);
}

void nop(void) {
    return;
}

void ora(void) {
    cpu.a |= read8(cpu.address);
    update_zn(cpu.a);
}

void pha(void) {
    push8(cpu.a);
}

void php(void) {
    push8(get_flag() | 0x10);
}

void pla(void) {
    cpu.a = pop8();
    update_zn(cpu.a);
}

void plp(void) {
    set_flag(pop8());
}

void rol_acc(void) {
    unsigned char c = (cpu.a & 0x80) != 0;
    cpu.a = (cpu.a << 1) + cpu.p.c;
    cpu.p.c = c;
    update_zn(cpu.a);
}

void rol(void) {
    unsigned char m = read8(cpu.address);
    unsigned char c = (m & 0x80) != 0;
    write8(cpu.address, (m << 1) + cpu.p.c);
    update_zn((m << 1) + cpu.p.c);
    cpu.p.c = c;
}

void ror_acc(void) {
    unsigned char c = (cpu.a & 0x01) != 0;
    cpu.a = (cpu.a >> 1) + (cpu.p.c << 7);
    cpu.p.c = c;
    update_zn(cpu.a);
}

void ror(void) {
    unsigned char m = read8(cpu.address);
    unsigned char c = (m & 0x01) != 0;
    write8(cpu.address, (m >> 1) + (cpu.p.c << 7));
    update_zn((m >> 1) + (cpu.p.c << 7));
    cpu.p.c = c;
}

void rti(void) {
    set_flag(pop8());
    cpu.pc = pop16() - 1;
}

void rts(void) {
    cpu.pc = pop16();
}

void sbc(void) {
    unsigned char a = cpu.a;
    unsigned char m = read8(cpu.address);
    unsigned short r = a - m - !cpu.p.c;
    cpu.a = r;
    cpu.p.c = r <= 0xff;
    cpu.p.v = ((a ^ m) & (a ^ r) & 0x80) != 0;
    update_zn(cpu.a);
}

void sec(void) {
    cpu.p.c = true;
}

void sed(void) {
    cpu.p.d = true;
}

void sei(void) {
    cpu.p.i = true;
}

void sta(void) {
    write8(cpu.address, cpu.a);
}

void stx(void) {
    write8(cpu.address, cpu.x);
}

void sty(void) {
    write8(cpu.address, cpu.y);
}

void tax(void) {
    cpu.x = cpu.a;
    update_zn(cpu.x);
}

void tay(void) {
    cpu.y = cpu.a;
    update_zn(cpu.y);
}

void tsx(void) {
    cpu.x = cpu.s;
    update_zn(cpu.x);
}

void txa(void) {
    cpu.a = cpu.x;
    update_zn(cpu.a);
}

void txs(void) {
    cpu.s = cpu.x;
}

void tya(void) {
    cpu.a = cpu.y;
    update_zn(cpu.a);
}

void dcp(void) {
    dec();
    cmp();
}

void isb(void) {
    inc();
    sbc();
}

void lax(void) {
    lda();
    ldx();
}

void rla(void) {
    rol();
    and();
}

void rra(void) {
    ror();
    adc();
}

void sax(void) {
    write8(cpu.address, cpu.a & cpu.x);
}

void slo(void) {
    asl();
    ora();
}

void sre(void) {
    lsr();
    eor();
}

Instruction instruction[] = {
    {0x69, "ADC",  adc,     IMM, 2, 2, None  },
    {0x65, "ADC",  adc,     ZPG, 2, 3, None  },
    {0x75, "ADC",  adc,     ZPX, 2, 4, None  },
    {0x6d, "ADC",  adc,     ABS, 3, 4, None  },
    {0x7d, "ADC",  adc,     ABX, 3, 4, Page  },
    {0x79, "ADC",  adc,     ABY, 3, 4, Page  },
    {0x61, "ADC",  adc,     INX, 2, 6, None  },
    {0x71, "ADC",  adc,     INY, 2, 5, Page  },
    {0x29, "AND",  and,     IMM, 2, 2, None  },
    {0x25, "AND",  and,     ZPG, 2, 3, None  },
    {0x35, "AND",  and,     ZPX, 2, 4, None  },
    {0x2d, "AND",  and,     ABS, 3, 4, None  },
    {0x3d, "AND",  and,     ABX, 3, 4, Page  },
    {0x39, "AND",  and,     ABY, 3, 4, Page  },
    {0x21, "AND",  and,     INX, 2, 6, None  },
    {0x31, "AND",  and,     INY, 2, 5, Page  },
    {0x0a, "ASL",  asl_acc, ACC, 1, 2, None  },
    {0x06, "ASL",  asl,     ZPG, 2, 5, None  },
    {0x16, "ASL",  asl,     ZPX, 2, 6, None  },
    {0x0e, "ASL",  asl,     ABS, 3, 6, None  },
    {0x1e, "ASL",  asl,     ABX, 3, 7, None  },
    {0x90, "BCC",  bcc,     REL, 2, 2, Branch},
    {0xb0, "BCS",  bcs,     REL, 2, 2, Branch},
    {0xf0, "BEQ",  beq,     REL, 2, 2, Branch},
    {0x24, "BIT",  bit,     ZPG, 2, 3, None  },
    {0x2c, "BIT",  bit,     ABS, 3, 4, None  },
    {0x30, "BMI",  bmi,     REL, 2, 2, Branch},
    {0xd0, "BNE",  bne,     REL, 2, 2, Branch},
    {0x10, "BPL",  bpl,     REL, 2, 2, Branch},
    {0x00, "BRK",  _brk,    IMP, 1, 7, None  },
    {0x50, "BVC",  bvc,     REL, 2, 2, Branch},
    {0x70, "BVS",  bvs,     REL, 2, 2, Branch},
    {0x18, "CLC",  clc,     IMP, 1, 2, None  },
    {0xd8, "CLD",  cld,     IMP, 1, 2, None  },
    {0x58, "CLI",  cli,     IMP, 1, 2, None  },
    {0xb8, "CLV",  clv,     IMP, 1, 2, None  },
    {0xc9, "CMP",  cmp,     IMM, 2, 2, None  },
    {0xc5, "CMP",  cmp,     ZPG, 2, 3, None  },
    {0xd5, "CMP",  cmp,     ZPX, 2, 4, None  },
    {0xcd, "CMP",  cmp,     ABS, 3, 4, None  },
    {0xdd, "CMP",  cmp,     ABX, 3, 4, Page  },
    {0xd9, "CMP",  cmp,     ABY, 3, 4, Page  },
    {0xc1, "CMP",  cmp,     INX, 2, 6, None  },
    {0xd1, "CMP",  cmp,     INY, 2, 5, Page  },
    {0xe0, "CPX",  cpx,     IMM, 2, 2, None  },
    {0xe4, "CPX",  cpx,     ZPG, 2, 3, None  },
    {0xec, "CPX",  cpx,     ABS, 3, 4, None  },
    {0xc0, "CPY",  cpy,     IMM, 2, 2, None  },
    {0xc4, "CPY",  cpy,     ZPG, 2, 3, None  },
    {0xcc, "CPY",  cpy,     ABS, 3, 4, None  },
    {0xc6, "DEC",  dec,     ZPG, 2, 5, None  },
    {0xd6, "DEC",  dec,     ZPX, 2, 6, None  },
    {0xce, "DEC",  dec,     ABS, 3, 6, None  },
    {0xde, "DEC",  dec,     ABX, 3, 7, None  },
    {0xca, "DEX",  dex,     IMP, 1, 2, None  },
    {0x88, "DEY",  dey,     IMP, 1, 2, None  },
    {0x49, "EOR",  eor,     IMM, 2, 2, None  },
    {0x45, "EOR",  eor,     ZPG, 2, 3, None  },
    {0x55, "EOR",  eor,     ZPX, 2, 4, None  },
    {0x4d, "EOR",  eor,     ABS, 3, 4, None  },
    {0x5d, "EOR",  eor,     ABX, 3, 4, Page  },
    {0x59, "EOR",  eor,     ABY, 3, 4, Page  },
    {0x41, "EOR",  eor,     INX, 2, 6, None  },
    {0x51, "EOR",  eor,     INY, 2, 5, Page  },
    {0xe6, "INC",  inc,     ZPG, 2, 5, None  },
    {0xf6, "INC",  inc,     ZPX, 2, 6, None  },
    {0xee, "INC",  inc,     ABS, 3, 6, None  },
    {0xfe, "INC",  inc,     ABX, 3, 7, None  },
    {0xe8, "INX",  inx,     IMP, 1, 2, None  },
    {0xc8, "INY",  iny,     IMP, 1, 2, None  },
    {0x4c, "JMP",  jmp,     ABS, 3, 3, None  },
    {0x6c, "JMP",  jmp,     IND, 3, 5, None  },
    {0x20, "JSR",  jsr,     ABS, 3, 6, None  },
    {0xa9, "LDA",  lda,     IMM, 2, 2, None  },
    {0xa5, "LDA",  lda,     ZPG, 2, 3, None  },
    {0xb5, "LDA",  lda,     ZPX, 2, 4, None  },
    {0xad, "LDA",  lda,     ABS, 3, 4, None  },
    {0xbd, "LDA",  lda,     ABX, 3, 4, Page  },
    {0xb9, "LDA",  lda,     ABY, 3, 4, Page  },
    {0xa1, "LDA",  lda,     INX, 2, 6, None  },
    {0xb1, "LDA",  lda,     INY, 2, 5, Page  },
    {0xa2, "LDX",  ldx,     IMM, 2, 2, None  },
    {0xa6, "LDX",  ldx,     ZPG, 2, 3, None  },
    {0xb6, "LDX",  ldx,     ZPY, 2, 4, None  },
    {0xae, "LDX",  ldx,     ABS, 3, 4, None  },
    {0xbe, "LDX",  ldx,     ABY, 3, 4, Page  },
    {0xa0, "LDY",  ldy,     IMM, 2, 2, None  },
    {0xa4, "LDY",  ldy,     ZPG, 2, 3, None  },
    {0xb4, "LDY",  ldy,     ZPX, 2, 4, None  },
    {0xac, "LDY",  ldy,     ABS, 3, 4, None  },
    {0xbc, "LDY",  ldy,     ABX, 3, 4, Page  },
    {0x4a, "LSR",  lsr_acc, ACC, 1, 2, None  },
    {0x46, "LSR",  lsr,     ZPG, 2, 5, None  },
    {0x56, "LSR",  lsr,     ZPX, 2, 6, None  },
    {0x4e, "LSR",  lsr,     ABS, 3, 6, None  },
    {0x5e, "LSR",  lsr,     ABX, 3, 7, None  },
    {0xea, "NOP",  nop,     IMP, 1, 2, None  },
    {0x09, "ORA",  ora,     IMM, 2, 2, None  },
    {0x05, "ORA",  ora,     ZPG, 2, 3, None  },
    {0x15, "ORA",  ora,     ZPX, 2, 4, None  },
    {0x0d, "ORA",  ora,     ABS, 3, 4, None  },
    {0x1d, "ORA",  ora,     ABX, 3, 4, Page  },
    {0x19, "ORA",  ora,     ABY, 3, 4, Page  },
    {0x01, "ORA",  ora,     INX, 2, 6, None  },
    {0x11, "ORA",  ora,     INY, 2, 5, Page  },
    {0x48, "PHA",  pha,     IMP, 1, 3, None  },
    {0x08, "PHP",  php,     IMP, 1, 3, None  },
    {0x68, "PLA",  pla,     IMP, 1, 4, None  },
    {0x28, "PLP",  plp,     IMP, 1, 4, None  },
    {0x2a, "ROL",  rol_acc, ACC, 1, 2, None  },
    {0x26, "ROL",  rol,     ZPG, 2, 5, None  },
    {0x36, "ROL",  rol,     ZPX, 2, 6, None  },
    {0x2e, "ROL",  rol,     ABS, 3, 6, None  },
    {0x3e, "ROL",  rol,     ABX, 3, 7, None  },
    {0x6a, "ROR",  ror_acc, ACC, 1, 2, None  },
    {0x66, "ROR",  ror,     ZPG, 2, 5, None  },
    {0x76, "ROR",  ror,     ZPX, 2, 6, None  },
    {0x6e, "ROR",  ror,     ABS, 3, 6, None  },
    {0x7e, "ROR",  ror,     ABX, 3, 7, None  },
    {0x40, "RTI",  rti,     IMP, 1, 6, None  },
    {0x60, "RTS",  rts,     IMP, 1, 6, None  },
    {0xe9, "SBC",  sbc,     IMM, 2, 2, None  },
    {0xe5, "SBC",  sbc,     ZPG, 2, 3, None  },
    {0xf5, "SBC",  sbc,     ZPX, 2, 4, None  },
    {0xed, "SBC",  sbc,     ABS, 3, 4, None  },
    {0xfd, "SBC",  sbc,     ABX, 3, 4, Page  },
    {0xf9, "SBC",  sbc,     ABY, 3, 4, Page  },
    {0xe1, "SBC",  sbc,     INX, 2, 6, None  },
    {0xf1, "SBC",  sbc,     INY, 2, 5, Page  },
    {0x38, "SEC",  sec,     IMP, 1, 2, None  },
    {0xf8, "SED",  sed,     IMP, 1, 2, None  },
    {0x78, "SEI",  sei,     IMP, 1, 2, None  },
    {0x85, "STA",  sta,     ZPG, 2, 3, None  },
    {0x95, "STA",  sta,     ZPX, 2, 4, None  },
    {0x8d, "STA",  sta,     ABS, 3, 4, None  },
    {0x9d, "STA",  sta,     ABX, 3, 5, None  },
    {0x99, "STA",  sta,     ABY, 3, 5, None  },
    {0x81, "STA",  sta,     INX, 2, 6, None  },
    {0x91, "STA",  sta,     INY, 2, 6, None  },
    {0x86, "STX",  stx,     ZPG, 2, 3, None  },
    {0x96, "STX",  stx,     ZPY, 2, 4, None  },
    {0x8e, "STX",  stx,     ABS, 3, 4, None  },
    {0x84, "STY",  sty,     ZPG, 2, 3, None  },
    {0x94, "STY",  sty,     ZPX, 2, 4, None  },
    {0x8c, "STY",  sty,     ABS, 3, 4, None  },
    {0xaa, "TAX",  tax,     IMP, 1, 2, None  },
    {0xa8, "TAY",  tay,     IMP, 1, 2, None  },
    {0xba, "TSX",  tsx,     IMP, 1, 2, None  },
    {0x8a, "TXA",  txa,     IMP, 1, 2, None  },
    {0x9a, "TXS",  txs,     IMP, 1, 2, None  },
    {0x98, "TYA",  tya,     IMP, 1, 2, None  },
    {0xc7, "*DCP", dcp,     ZPG, 2, 5, None  },
    {0xd7, "*DCP", dcp,     ZPX, 2, 6, None  },
    {0xcf, "*DCP", dcp,     ABS, 3, 6, None  },
    {0xdf, "*DCP", dcp,     ABX, 3, 6, Page  },
    {0xdb, "*DCP", dcp,     ABY, 3, 6, Page  },
    {0xc3, "*DCP", dcp,     INX, 2, 8, None  },
    {0xd3, "*DCP", dcp,     INY, 2, 7, Page  },
    {0xe7, "*ISB", isb,     ZPG, 2, 5, None  },
    {0xf7, "*ISB", isb,     ZPX, 2, 6, None  },
    {0xef, "*ISB", isb,     ABS, 3, 6, None  },
    {0xff, "*ISB", isb,     ABX, 3, 6, Page  },
    {0xfb, "*ISB", isb,     ABY, 3, 6, Page  },
    {0xe3, "*ISB", isb,     INX, 2, 8, None  },
    {0xf3, "*ISB", isb,     INY, 2, 7, Page  },
    // {0x02, "*JAM", jam,     IMP, 1, 0, None  },
    // {0x12, "*JAM", jam,     IMP, 1, 0, None  },
    // {0x22, "*JAM", jam,     IMP, 1, 0, None  },
    // {0x32, "*JAM", jam,     IMP, 1, 0, None  },
    // {0x42, "*JAM", jam,     IMP, 1, 0, None  },
    // {0x52, "*JAM", jam,     IMP, 1, 0, None  },
    // {0x62, "*JAM", jam,     IMP, 1, 0, None  },
    // {0x72, "*JAM", jam,     IMP, 1, 0, None  },
    // {0x92, "*JAM", jam,     IMP, 1, 0, None  },
    // {0xb2, "*JAM", jam,     IMP, 1, 0, None  },
    // {0xd2, "*JAM", jam,     IMP, 1, 0, None  },
    // {0xf2, "*JAM", jam,     IMP, 1, 0, None  },
    {0xa7, "*LAX", lax,     ZPG, 2, 3, None  },
    {0xb7, "*LAX", lax,     ZPY, 2, 4, None  },
    {0xaf, "*LAX", lax,     ABS, 3, 4, None  },
    {0xbf, "*LAX", lax,     ABY, 3, 4, Page  },
    {0xa3, "*LAX", lax,     INX, 2, 6, None  },
    {0xb3, "*LAX", lax,     INY, 2, 5, Page  },
    {0x1a, "*NOP", nop,     IMP, 1, 2, None  },
    {0x3a, "*NOP", nop,     IMP, 1, 2, None  },
    {0x5a, "*NOP", nop,     IMP, 1, 2, None  },
    {0x7a, "*NOP", nop,     IMP, 1, 2, None  },
    {0xda, "*NOP", nop,     IMP, 1, 2, None  },
    {0xfa, "*NOP", nop,     IMP, 1, 2, None  },
    {0x80, "*NOP", nop,     IMM, 2, 2, None  },
    // {0x82, "*NOP", nop,     IMM, 2, 2, None  },
    // {0x89, "*NOP", nop,     IMM, 2, 2, None  },
    // {0xc2, "*NOP", nop,     IMM, 2, 2, None  },
    // {0xe2, "*NOP", nop,     IMM, 2, 2, None  },
    {0x04, "*NOP", nop,     ZPG, 2, 3, None  },
    {0x44, "*NOP", nop,     ZPG, 2, 3, None  },
    {0x64, "*NOP", nop,     ZPG, 2, 3, None  },
    {0x14, "*NOP", nop,     ZPX, 2, 4, None  },
    {0x34, "*NOP", nop,     ZPX, 2, 4, None  },
    {0x54, "*NOP", nop,     ZPX, 2, 4, None  },
    {0x74, "*NOP", nop,     ZPX, 2, 4, None  },
    {0xd4, "*NOP", nop,     ZPX, 2, 4, None  },
    {0xf4, "*NOP", nop,     ZPX, 2, 4, None  },
    {0x0c, "*NOP", nop,     ABS, 3, 4, None  },
    {0x1c, "*NOP", nop,     ABX, 3, 4, Page  },
    {0x3c, "*NOP", nop,     ABX, 3, 4, Page  },
    {0x5c, "*NOP", nop,     ABX, 3, 4, Page  },
    {0x7c, "*NOP", nop,     ABX, 3, 4, Page  },
    {0xdc, "*NOP", nop,     ABX, 3, 4, Page  },
    {0xfc, "*NOP", nop,     ABX, 3, 4, Page  },
    {0x27, "*RLA", rla,     ZPG, 2, 5, None  },
    {0x37, "*RLA", rla,     ZPX, 2, 6, None  },
    {0x2f, "*RLA", rla,     ABS, 3, 6, None  },
    {0x3f, "*RLA", rla,     ABX, 3, 6, Page  },
    {0x3b, "*RLA", rla,     ABY, 3, 6, Page  },
    {0x23, "*RLA", rla,     INX, 2, 8, None  },
    {0x33, "*RLA", rla,     INY, 2, 7, Page  },
    {0x67, "*RRA", rra,     ZPG, 2, 5, None  },
    {0x77, "*RRA", rra,     ZPX, 2, 6, None  },
    {0x6f, "*RRA", rra,     ABS, 3, 6, None  },
    {0x7f, "*RRA", rra,     ABX, 3, 6, Page  },
    {0x7b, "*RRA", rra,     ABY, 3, 6, Page  },
    {0x63, "*RRA", rra,     INX, 2, 8, None  },
    {0x73, "*RRA", rra,     INY, 2, 7, Page  },
    {0x87, "*SAX", sax,     ZPG, 2, 3, None  },
    {0x97, "*SAX", sax,     ZPY, 2, 4, None  },
    {0x8f, "*SAX", sax,     ABS, 3, 4, None  },
    {0x83, "*SAX", sax,     INX, 2, 6, None  },
    {0xeb, "*SBC", sbc,     IMM, 2, 2, None  },
    {0x07, "*SLO", slo,     ZPG, 2, 5, None  },
    {0x17, "*SLO", slo,     ZPX, 2, 6, None  },
    {0x0f, "*SLO", slo,     ABS, 3, 6, None  },
    {0x1f, "*SLO", slo,     ABX, 3, 6, Page  },
    {0x1b, "*SLO", slo,     ABY, 3, 6, Page  },
    {0x03, "*SLO", slo,     INX, 2, 8, None  },
    {0x13, "*SLO", slo,     INY, 2, 7, Page  },
    {0x47, "*SRE", sre,     ZPG, 2, 5, None  },
    {0x57, "*SRE", sre,     ZPX, 2, 6, None  },
    {0x4f, "*SRE", sre,     ABS, 3, 6, None  },
    {0x5f, "*SRE", sre,     ABX, 3, 6, Page  },
    {0x5b, "*SRE", sre,     ABY, 3, 6, Page  },
    {0x43, "*SRE", sre,     INX, 2, 8, None  },
    {0x53, "*SRE", sre,     INY, 2, 7, Page  },
};

void nmi(void) {
    push16(cpu.pc);
    push8(get_flag());
    cpu.pc = read16(0xfffa);
    cpu.p.i = true;
    tick(2);
}

bool parallel_mode;
int button_index;
unsigned char button_status;

unsigned char read_joypad(void) {
    if(button_index > 7) {
        return 1;
    } else {
        unsigned char value = (button_status >> button_index) & 0x01;
        if(parallel_mode == false) {
            button_index += 1;
        }
        return value;
    }
}

void write_joypad(unsigned char value) {
    parallel_mode = (value & 0x01) != 0;
    if(parallel_mode == true) {
        button_index = 0;
    }
}
