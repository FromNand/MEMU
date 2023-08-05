use crate::Bus;
use lazy_static::lazy_static;
use std::collections::HashMap;

enum AddrMode {
    IMP, ACC, IMM, ZPG, ZPX, ZPY, ABS, ABX, ABY, IND, INX, INY, REL,
}

#[derive(Clone, PartialEq)]
enum CycleMode {
    None, Page, Branch,
}

struct Instruction {
    opcode: u8,
    mnemonic: &'static str,
    addr_mode: AddrMode,
    function: fn(&mut CPU),
    length: u16,
    cycle: usize,
    cycle_mode: CycleMode,
}

impl Instruction {
    fn new(opcode: u8, mnemonic: &'static str, addr_mode: AddrMode, function: fn(&mut CPU), length: u16,
    cycle: usize, cycle_mode: CycleMode, ) -> Self {
        Instruction { opcode, mnemonic, addr_mode, function, length, cycle, cycle_mode, }
    }
}

lazy_static! {
    static ref INST_LIST: Vec<Instruction> = vec![
        Instruction::new(0x69, "ADC", AddrMode::IMM, adc, 2, 2, CycleMode::None),
        Instruction::new(0x65, "ADC", AddrMode::ZPG, adc, 2, 3, CycleMode::None),
        Instruction::new(0x75, "ADC", AddrMode::ZPX, adc, 2, 4, CycleMode::None),
        Instruction::new(0x6d, "ADC", AddrMode::ABS, adc, 3, 4, CycleMode::None),
        Instruction::new(0x7d, "ADC", AddrMode::ABX, adc, 3, 4, CycleMode::Page),
        Instruction::new(0x79, "ADC", AddrMode::ABY, adc, 3, 4, CycleMode::Page),
        Instruction::new(0x61, "ADC", AddrMode::INX, adc, 2, 6, CycleMode::None),
        Instruction::new(0x71, "ADC", AddrMode::INY, adc, 2, 5, CycleMode::Page),
        Instruction::new(0x29, "AND", AddrMode::IMM, and, 2, 2, CycleMode::None),
        Instruction::new(0x25, "AND", AddrMode::ZPG, and, 2, 3, CycleMode::None),
        Instruction::new(0x35, "AND", AddrMode::ZPX, and, 2, 4, CycleMode::None),
        Instruction::new(0x2d, "AND", AddrMode::ABS, and, 3, 4, CycleMode::None),
        Instruction::new(0x3d, "AND", AddrMode::ABX, and, 3, 4, CycleMode::Page),
        Instruction::new(0x39, "AND", AddrMode::ABY, and, 3, 4, CycleMode::Page),
        Instruction::new(0x21, "AND", AddrMode::INX, and, 2, 6, CycleMode::None),
        Instruction::new(0x31, "AND", AddrMode::INY, and, 2, 5, CycleMode::Page),
        Instruction::new(0x0a, "ASL", AddrMode::ACC, asl_acc, 1, 2, CycleMode::None),
        Instruction::new(0x06, "ASL", AddrMode::ZPG, asl, 2, 5, CycleMode::None),
        Instruction::new(0x16, "ASL", AddrMode::ZPX, asl, 2, 6, CycleMode::None),
        Instruction::new(0x0e, "ASL", AddrMode::ABS, asl, 3, 6, CycleMode::None),
        Instruction::new(0x1e, "ASL", AddrMode::ABX, asl, 3, 7, CycleMode::None),
        Instruction::new(0x90, "BCC", AddrMode::REL, bcc, 2, 2, CycleMode::Branch),
        Instruction::new(0xb0, "BCS", AddrMode::REL, bcs, 2, 2, CycleMode::Branch),
        Instruction::new(0xf0, "BEQ", AddrMode::REL, beq, 2, 2, CycleMode::Branch),
        Instruction::new(0x24, "BIT", AddrMode::ZPG, bit, 2, 3, CycleMode::None),
        Instruction::new(0x2c, "BIT", AddrMode::ABS, bit, 3, 4, CycleMode::None),
        Instruction::new(0x30, "BMI", AddrMode::REL, bmi, 2, 2, CycleMode::Branch),
        Instruction::new(0xd0, "BNE", AddrMode::REL, bne, 2, 2, CycleMode::Branch),
        Instruction::new(0x10, "BPL", AddrMode::REL, bpl, 2, 2, CycleMode::Branch),
        Instruction::new(0x00, "BRK", AddrMode::IMP, brk, 1, 7, CycleMode::None),
        Instruction::new(0x50, "BVC", AddrMode::REL, bvc, 2, 2, CycleMode::Branch),
        Instruction::new(0x70, "BVS", AddrMode::REL, bvs, 2, 2, CycleMode::Branch),
        Instruction::new(0x18, "CLC", AddrMode::IMP, clc, 1, 2, CycleMode::None),
        Instruction::new(0xd8, "CLD", AddrMode::IMP, cld, 1, 2, CycleMode::None),
        Instruction::new(0x58, "CLI", AddrMode::IMP, cli, 1, 2, CycleMode::None),
        Instruction::new(0xb8, "CLV", AddrMode::IMP, clv, 1, 2, CycleMode::None),
        Instruction::new(0xc9, "CMP", AddrMode::IMM, cmp, 2, 2, CycleMode::None),
        Instruction::new(0xc5, "CMP", AddrMode::ZPG, cmp, 2, 3, CycleMode::None),
        Instruction::new(0xd5, "CMP", AddrMode::ZPX, cmp, 2, 4, CycleMode::None),
        Instruction::new(0xcd, "CMP", AddrMode::ABS, cmp, 3, 4, CycleMode::None),
        Instruction::new(0xdd, "CMP", AddrMode::ABX, cmp, 3, 4, CycleMode::Page),
        Instruction::new(0xd9, "CMP", AddrMode::ABY, cmp, 3, 4, CycleMode::Page),
        Instruction::new(0xc1, "CMP", AddrMode::INX, cmp, 2, 6, CycleMode::None),
        Instruction::new(0xd1, "CMP", AddrMode::INY, cmp, 2, 5, CycleMode::Page),
        Instruction::new(0xe0, "CPX", AddrMode::IMM, cpx, 2, 2, CycleMode::None),
        Instruction::new(0xe4, "CPX", AddrMode::ZPG, cpx, 2, 3, CycleMode::None),
        Instruction::new(0xec, "CPX", AddrMode::ABS, cpx, 3, 4, CycleMode::None),
        Instruction::new(0xc0, "CPY", AddrMode::IMM, cpy, 2, 2, CycleMode::None),
        Instruction::new(0xc4, "CPY", AddrMode::ZPG, cpy, 2, 3, CycleMode::None),
        Instruction::new(0xcc, "CPY", AddrMode::ABS, cpy, 3, 4, CycleMode::None),
        Instruction::new(0xc6, "DEC", AddrMode::ZPG, dec, 2, 5, CycleMode::None),
        Instruction::new(0xd6, "DEC", AddrMode::ZPX, dec, 2, 6, CycleMode::None),
        Instruction::new(0xce, "DEC", AddrMode::ABS, dec, 3, 6, CycleMode::None),
        Instruction::new(0xde, "DEC", AddrMode::ABX, dec, 3, 7, CycleMode::None),
        Instruction::new(0xca, "DEX", AddrMode::IMP, dex, 1, 2, CycleMode::None),
        Instruction::new(0x88, "DEY", AddrMode::IMP, dey, 1, 2, CycleMode::None),
        Instruction::new(0x49, "EOR", AddrMode::IMM, eor, 2, 2, CycleMode::None),
        Instruction::new(0x45, "EOR", AddrMode::ZPG, eor, 2, 3, CycleMode::None),
        Instruction::new(0x55, "EOR", AddrMode::ZPX, eor, 2, 4, CycleMode::None),
        Instruction::new(0x4d, "EOR", AddrMode::ABS, eor, 3, 4, CycleMode::None),
        Instruction::new(0x5d, "EOR", AddrMode::ABX, eor, 3, 4, CycleMode::Page),
        Instruction::new(0x59, "EOR", AddrMode::ABY, eor, 3, 4, CycleMode::Page),
        Instruction::new(0x41, "EOR", AddrMode::INX, eor, 2, 6, CycleMode::None),
        Instruction::new(0x51, "EOR", AddrMode::INY, eor, 2, 5, CycleMode::Page),
        Instruction::new(0xe6, "INC", AddrMode::ZPG, inc, 2, 5, CycleMode::None),
        Instruction::new(0xf6, "INC", AddrMode::ZPX, inc, 2, 6, CycleMode::None),
        Instruction::new(0xee, "INC", AddrMode::ABS, inc, 3, 6, CycleMode::None),
        Instruction::new(0xfe, "INC", AddrMode::ABX, inc, 3, 7, CycleMode::None),
        Instruction::new(0xe8, "INX", AddrMode::IMP, inx, 1, 2, CycleMode::None),
        Instruction::new(0xc8, "INY", AddrMode::IMP, iny, 1, 2, CycleMode::None),
        Instruction::new(0x4c, "JMP", AddrMode::ABS, jmp, 3, 3, CycleMode::None),
        Instruction::new(0x6c, "JMP", AddrMode::IND, jmp, 3, 5, CycleMode::None),
        Instruction::new(0x20, "JSR", AddrMode::ABS, jsr, 3, 6, CycleMode::None),
        Instruction::new(0xa9, "LDA", AddrMode::IMM, lda, 2, 2, CycleMode::None),
        Instruction::new(0xa5, "LDA", AddrMode::ZPG, lda, 2, 3, CycleMode::None),
        Instruction::new(0xb5, "LDA", AddrMode::ZPX, lda, 2, 4, CycleMode::None),
        Instruction::new(0xad, "LDA", AddrMode::ABS, lda, 3, 4, CycleMode::None),
        Instruction::new(0xbd, "LDA", AddrMode::ABX, lda, 3, 4, CycleMode::Page),
        Instruction::new(0xb9, "LDA", AddrMode::ABY, lda, 3, 4, CycleMode::Page),
        Instruction::new(0xa1, "LDA", AddrMode::INX, lda, 2, 6, CycleMode::None),
        Instruction::new(0xb1, "LDA", AddrMode::INY, lda, 2, 5, CycleMode::Page),
        Instruction::new(0xa2, "LDX", AddrMode::IMM, ldx, 2, 2, CycleMode::None),
        Instruction::new(0xa6, "LDX", AddrMode::ZPG, ldx, 2, 3, CycleMode::None),
        Instruction::new(0xb6, "LDX", AddrMode::ZPY, ldx, 2, 4, CycleMode::None),
        Instruction::new(0xae, "LDX", AddrMode::ABS, ldx, 3, 4, CycleMode::None),
        Instruction::new(0xbe, "LDX", AddrMode::ABY, ldx, 3, 4, CycleMode::Page),
        Instruction::new(0xa0, "LDY", AddrMode::IMM, ldy, 2, 2, CycleMode::None),
        Instruction::new(0xa4, "LDY", AddrMode::ZPG, ldy, 2, 3, CycleMode::None),
        Instruction::new(0xb4, "LDY", AddrMode::ZPX, ldy, 2, 4, CycleMode::None),
        Instruction::new(0xac, "LDY", AddrMode::ABS, ldy, 3, 4, CycleMode::None),
        Instruction::new(0xbc, "LDY", AddrMode::ABX, ldy, 3, 4, CycleMode::Page),
        Instruction::new(0x4a, "LSR", AddrMode::ACC, lsr_acc, 1, 2, CycleMode::None),
        Instruction::new(0x46, "LSR", AddrMode::ZPG, lsr, 2, 5, CycleMode::None),
        Instruction::new(0x56, "LSR", AddrMode::ZPX, lsr, 2, 6, CycleMode::None),
        Instruction::new(0x4e, "LSR", AddrMode::ABS, lsr, 3, 6, CycleMode::None),
        Instruction::new(0x5e, "LSR", AddrMode::ABX, lsr, 3, 7, CycleMode::None),
        Instruction::new(0xea, "NOP", AddrMode::IMP, nop, 1, 2, CycleMode::None),
        Instruction::new(0x09, "ORA", AddrMode::IMM, ora, 2, 2, CycleMode::None),
        Instruction::new(0x05, "ORA", AddrMode::ZPG, ora, 2, 3, CycleMode::None),
        Instruction::new(0x15, "ORA", AddrMode::ZPX, ora, 2, 4, CycleMode::None),
        Instruction::new(0x0d, "ORA", AddrMode::ABS, ora, 3, 4, CycleMode::None),
        Instruction::new(0x1d, "ORA", AddrMode::ABX, ora, 3, 4, CycleMode::Page),
        Instruction::new(0x19, "ORA", AddrMode::ABY, ora, 3, 4, CycleMode::Page),
        Instruction::new(0x01, "ORA", AddrMode::INX, ora, 2, 6, CycleMode::None),
        Instruction::new(0x11, "ORA", AddrMode::INY, ora, 2, 5, CycleMode::Page),
        Instruction::new(0x48, "PHA", AddrMode::IMP, pha, 1, 3, CycleMode::None),
        Instruction::new(0x08, "PHP", AddrMode::IMP, php, 1, 3, CycleMode::None),
        Instruction::new(0x68, "PLA", AddrMode::IMP, pla, 1, 4, CycleMode::None),
        Instruction::new(0x28, "PLP", AddrMode::IMP, plp, 1, 4, CycleMode::None),
        Instruction::new(0x2a, "ROL", AddrMode::ACC, rol_acc, 1, 2, CycleMode::None),
        Instruction::new(0x26, "ROL", AddrMode::ZPG, rol, 2, 5, CycleMode::None),
        Instruction::new(0x36, "ROL", AddrMode::ZPX, rol, 2, 6, CycleMode::None),
        Instruction::new(0x2e, "ROL", AddrMode::ABS, rol, 3, 6, CycleMode::None),
        Instruction::new(0x3e, "ROL", AddrMode::ABX, rol, 3, 7, CycleMode::None),
        Instruction::new(0x6a, "ROR", AddrMode::ACC, ror_acc, 1, 2, CycleMode::None),
        Instruction::new(0x66, "ROR", AddrMode::ZPG, ror, 2, 5, CycleMode::None),
        Instruction::new(0x76, "ROR", AddrMode::ZPX, ror, 2, 6, CycleMode::None),
        Instruction::new(0x6e, "ROR", AddrMode::ABS, ror, 3, 6, CycleMode::None),
        Instruction::new(0x7e, "ROR", AddrMode::ABX, ror, 3, 7, CycleMode::None),
        Instruction::new(0x40, "RTI", AddrMode::IMP, rti, 1, 6, CycleMode::None),
        Instruction::new(0x60, "RTS", AddrMode::IMP, rts, 1, 6, CycleMode::None),
        Instruction::new(0xe9, "SBC", AddrMode::IMM, sbc, 2, 2, CycleMode::None),
        Instruction::new(0xe5, "SBC", AddrMode::ZPG, sbc, 2, 3, CycleMode::None),
        Instruction::new(0xf5, "SBC", AddrMode::ZPX, sbc, 2, 4, CycleMode::None),
        Instruction::new(0xed, "SBC", AddrMode::ABS, sbc, 3, 4, CycleMode::None),
        Instruction::new(0xfd, "SBC", AddrMode::ABX, sbc, 3, 4, CycleMode::Page),
        Instruction::new(0xf9, "SBC", AddrMode::ABY, sbc, 3, 4, CycleMode::Page),
        Instruction::new(0xe1, "SBC", AddrMode::INX, sbc, 2, 6, CycleMode::None),
        Instruction::new(0xf1, "SBC", AddrMode::INY, sbc, 2, 5, CycleMode::Page),
        Instruction::new(0x38, "SEC", AddrMode::IMP, sec, 1, 2, CycleMode::None),
        Instruction::new(0xf8, "SED", AddrMode::IMP, sed, 1, 2, CycleMode::None),
        Instruction::new(0x78, "SEI", AddrMode::IMP, sei, 1, 2, CycleMode::None),
        Instruction::new(0x85, "STA", AddrMode::ZPG, sta, 2, 3, CycleMode::None),
        Instruction::new(0x95, "STA", AddrMode::ZPX, sta, 2, 4, CycleMode::None),
        Instruction::new(0x8d, "STA", AddrMode::ABS, sta, 3, 4, CycleMode::None),
        Instruction::new(0x9d, "STA", AddrMode::ABX, sta, 3, 5, CycleMode::None),
        Instruction::new(0x99, "STA", AddrMode::ABY, sta, 3, 5, CycleMode::None),
        Instruction::new(0x81, "STA", AddrMode::INX, sta, 2, 6, CycleMode::None),
        Instruction::new(0x91, "STA", AddrMode::INY, sta, 2, 6, CycleMode::None),
        Instruction::new(0x86, "STX", AddrMode::ZPG, stx, 2, 3, CycleMode::None),
        Instruction::new(0x96, "STX", AddrMode::ZPY, stx, 2, 4, CycleMode::None),
        Instruction::new(0x8e, "STX", AddrMode::ABS, stx, 3, 4, CycleMode::None),
        Instruction::new(0x84, "STY", AddrMode::ZPG, sty, 2, 3, CycleMode::None),
        Instruction::new(0x94, "STY", AddrMode::ZPX, sty, 2, 4, CycleMode::None),
        Instruction::new(0x8c, "STY", AddrMode::ABS, sty, 3, 4, CycleMode::None),
        Instruction::new(0xaa, "TAX", AddrMode::IMP, tax, 1, 2, CycleMode::None),
        Instruction::new(0xa8, "TAY", AddrMode::IMP, tay, 1, 2, CycleMode::None),
        Instruction::new(0xba, "TSX", AddrMode::IMP, tsx, 1, 2, CycleMode::None),
        Instruction::new(0x8a, "TXA", AddrMode::IMP, txa, 1, 2, CycleMode::None),
        Instruction::new(0x9a, "TXS", AddrMode::IMP, txs, 1, 2, CycleMode::None),
        Instruction::new(0x98, "TYA", AddrMode::IMP, tya, 1, 2, CycleMode::None),
        Instruction::new(0xc7, "*DCP", AddrMode::ZPG, dcp, 2, 5, CycleMode::None),
        Instruction::new(0xd7, "*DCP", AddrMode::ZPX, dcp, 2, 6, CycleMode::None),
        Instruction::new(0xcf, "*DCP", AddrMode::ABS, dcp, 3, 6, CycleMode::None),
        Instruction::new(0xdf, "*DCP", AddrMode::ABX, dcp, 3, 6, CycleMode::Page),
        Instruction::new(0xdb, "*DCP", AddrMode::ABY, dcp, 3, 6, CycleMode::Page),
        Instruction::new(0xc3, "*DCP", AddrMode::INX, dcp, 2, 8, CycleMode::None),
        Instruction::new(0xd3, "*DCP", AddrMode::INY, dcp, 2, 7, CycleMode::Page),
        Instruction::new(0xe7, "*ISB", AddrMode::ZPG, isb, 2, 5, CycleMode::None),
        Instruction::new(0xf7, "*ISB", AddrMode::ZPX, isb, 2, 6, CycleMode::None),
        Instruction::new(0xef, "*ISB", AddrMode::ABS, isb, 3, 6, CycleMode::None),
        Instruction::new(0xff, "*ISB", AddrMode::ABX, isb, 3, 6, CycleMode::Page),
        Instruction::new(0xfb, "*ISB", AddrMode::ABY, isb, 3, 6, CycleMode::Page),
        Instruction::new(0xe3, "*ISB", AddrMode::INX, isb, 2, 8, CycleMode::None),
        Instruction::new(0xf3, "*ISB", AddrMode::INY, isb, 2, 7, CycleMode::Page),
        Instruction::new(0x02, "*JAM", AddrMode::IMP, jam, 1, 0, CycleMode::None),
        Instruction::new(0x12, "*JAM", AddrMode::IMP, jam, 1, 0, CycleMode::None),
        Instruction::new(0x22, "*JAM", AddrMode::IMP, jam, 1, 0, CycleMode::None),
        Instruction::new(0x32, "*JAM", AddrMode::IMP, jam, 1, 0, CycleMode::None),
        Instruction::new(0x42, "*JAM", AddrMode::IMP, jam, 1, 0, CycleMode::None),
        Instruction::new(0x52, "*JAM", AddrMode::IMP, jam, 1, 0, CycleMode::None),
        Instruction::new(0x62, "*JAM", AddrMode::IMP, jam, 1, 0, CycleMode::None),
        Instruction::new(0x72, "*JAM", AddrMode::IMP, jam, 1, 0, CycleMode::None),
        Instruction::new(0x92, "*JAM", AddrMode::IMP, jam, 1, 0, CycleMode::None),
        Instruction::new(0xb2, "*JAM", AddrMode::IMP, jam, 1, 0, CycleMode::None),
        Instruction::new(0xd2, "*JAM", AddrMode::IMP, jam, 1, 0, CycleMode::None),
        Instruction::new(0xf2, "*JAM", AddrMode::IMP, jam, 1, 0, CycleMode::None),
        Instruction::new(0xa7, "*LAX", AddrMode::ZPG, lax, 2, 3, CycleMode::None),
        Instruction::new(0xb7, "*LAX", AddrMode::ZPY, lax, 2, 4, CycleMode::None),
        Instruction::new(0xaf, "*LAX", AddrMode::ABS, lax, 3, 4, CycleMode::None),
        Instruction::new(0xbf, "*LAX", AddrMode::ABY, lax, 3, 4, CycleMode::Page),
        Instruction::new(0xa3, "*LAX", AddrMode::INX, lax, 2, 6, CycleMode::None),
        Instruction::new(0xb3, "*LAX", AddrMode::INY, lax, 2, 5, CycleMode::Page),
        Instruction::new(0x1a, "*NOP", AddrMode::IMP, nop, 1, 2, CycleMode::None),
        Instruction::new(0x3a, "*NOP", AddrMode::IMP, nop, 1, 2, CycleMode::None),
        Instruction::new(0x5a, "*NOP", AddrMode::IMP, nop, 1, 2, CycleMode::None),
        Instruction::new(0x7a, "*NOP", AddrMode::IMP, nop, 1, 2, CycleMode::None),
        Instruction::new(0xda, "*NOP", AddrMode::IMP, nop, 1, 2, CycleMode::None),
        Instruction::new(0xfa, "*NOP", AddrMode::IMP, nop, 1, 2, CycleMode::None),
        Instruction::new(0x80, "*NOP", AddrMode::IMM, nop, 2, 2, CycleMode::None),
        Instruction::new(0x82, "*NOP", AddrMode::IMM, nop, 2, 2, CycleMode::None),
        Instruction::new(0x89, "*NOP", AddrMode::IMM, nop, 2, 2, CycleMode::None),
        Instruction::new(0xc2, "*NOP", AddrMode::IMM, nop, 2, 2, CycleMode::None),
        Instruction::new(0xe2, "*NOP", AddrMode::IMM, nop, 2, 2, CycleMode::None),
        Instruction::new(0x04, "*NOP", AddrMode::ZPG, nop, 2, 3, CycleMode::None),
        Instruction::new(0x44, "*NOP", AddrMode::ZPG, nop, 2, 3, CycleMode::None),
        Instruction::new(0x64, "*NOP", AddrMode::ZPG, nop, 2, 3, CycleMode::None),
        Instruction::new(0x14, "*NOP", AddrMode::ZPX, nop, 2, 4, CycleMode::None),
        Instruction::new(0x34, "*NOP", AddrMode::ZPX, nop, 2, 4, CycleMode::None),
        Instruction::new(0x54, "*NOP", AddrMode::ZPX, nop, 2, 4, CycleMode::None),
        Instruction::new(0x74, "*NOP", AddrMode::ZPX, nop, 2, 4, CycleMode::None),
        Instruction::new(0xd4, "*NOP", AddrMode::ZPX, nop, 2, 4, CycleMode::None),
        Instruction::new(0xf4, "*NOP", AddrMode::ZPX, nop, 2, 4, CycleMode::None),
        Instruction::new(0x0c, "*NOP", AddrMode::ABS, nop, 3, 4, CycleMode::None),
        Instruction::new(0x1c, "*NOP", AddrMode::ABX, nop, 3, 4, CycleMode::Page),
        Instruction::new(0x3c, "*NOP", AddrMode::ABX, nop, 3, 4, CycleMode::Page),
        Instruction::new(0x5c, "*NOP", AddrMode::ABX, nop, 3, 4, CycleMode::Page),
        Instruction::new(0x7c, "*NOP", AddrMode::ABX, nop, 3, 4, CycleMode::Page),
        Instruction::new(0xdc, "*NOP", AddrMode::ABX, nop, 3, 4, CycleMode::Page),
        Instruction::new(0xfc, "*NOP", AddrMode::ABX, nop, 3, 4, CycleMode::Page),
        Instruction::new(0x27, "*RLA", AddrMode::ZPG, rla, 2, 5, CycleMode::None),
        Instruction::new(0x37, "*RLA", AddrMode::ZPX, rla, 2, 6, CycleMode::None),
        Instruction::new(0x2f, "*RLA", AddrMode::ABS, rla, 3, 6, CycleMode::None),
        Instruction::new(0x3f, "*RLA", AddrMode::ABX, rla, 3, 6, CycleMode::Page),
        Instruction::new(0x3b, "*RLA", AddrMode::ABY, rla, 3, 6, CycleMode::Page),
        Instruction::new(0x23, "*RLA", AddrMode::INX, rla, 2, 8, CycleMode::None),
        Instruction::new(0x33, "*RLA", AddrMode::INY, rla, 2, 7, CycleMode::Page),
        Instruction::new(0x67, "*RRA", AddrMode::ZPG, rra, 2, 5, CycleMode::None),
        Instruction::new(0x77, "*RRA", AddrMode::ZPX, rra, 2, 6, CycleMode::None),
        Instruction::new(0x6f, "*RRA", AddrMode::ABS, rra, 3, 6, CycleMode::None),
        Instruction::new(0x7f, "*RRA", AddrMode::ABX, rra, 3, 6, CycleMode::Page),
        Instruction::new(0x7b, "*RRA", AddrMode::ABY, rra, 3, 6, CycleMode::Page),
        Instruction::new(0x63, "*RRA", AddrMode::INX, rra, 2, 8, CycleMode::None),
        Instruction::new(0x73, "*RRA", AddrMode::INY, rra, 2, 7, CycleMode::Page),
        Instruction::new(0x87, "*SAX", AddrMode::ZPG, sax, 2, 3, CycleMode::None),
        Instruction::new(0x97, "*SAX", AddrMode::ZPY, sax, 2, 4, CycleMode::None),
        Instruction::new(0x8f, "*SAX", AddrMode::ABS, sax, 3, 4, CycleMode::None),
        Instruction::new(0x83, "*SAX", AddrMode::INX, sax, 2, 6, CycleMode::None),
        Instruction::new(0xeb, "*SBC", AddrMode::IMM, sbc, 2, 2, CycleMode::None),
        Instruction::new(0x07, "*SLO", AddrMode::ZPG, slo, 2, 5, CycleMode::None),
        Instruction::new(0x17, "*SLO", AddrMode::ZPX, slo, 2, 6, CycleMode::None),
        Instruction::new(0x0f, "*SLO", AddrMode::ABS, slo, 3, 6, CycleMode::None),
        Instruction::new(0x1f, "*SLO", AddrMode::ABX, slo, 3, 6, CycleMode::Page),
        Instruction::new(0x1b, "*SLO", AddrMode::ABY, slo, 3, 6, CycleMode::Page),
        Instruction::new(0x03, "*SLO", AddrMode::INX, slo, 2, 8, CycleMode::None),
        Instruction::new(0x13, "*SLO", AddrMode::INY, slo, 2, 7, CycleMode::Page),
        Instruction::new(0x47, "*SRE", AddrMode::ZPG, sre, 2, 5, CycleMode::None),
        Instruction::new(0x57, "*SRE", AddrMode::ZPX, sre, 2, 6, CycleMode::None),
        Instruction::new(0x4f, "*SRE", AddrMode::ABS, sre, 3, 6, CycleMode::None),
        Instruction::new(0x5f, "*SRE", AddrMode::ABX, sre, 3, 6, CycleMode::Page),
        Instruction::new(0x5b, "*SRE", AddrMode::ABY, sre, 3, 6, CycleMode::Page),
        Instruction::new(0x43, "*SRE", AddrMode::INX, sre, 2, 8, CycleMode::None),
        Instruction::new(0x53, "*SRE", AddrMode::INY, sre, 2, 7, CycleMode::Page),
    ];
    static ref INST_MAP: HashMap<u8, &'static Instruction> = {
        let mut map = HashMap::new();
        for inst in &*INST_LIST {
            map.insert(inst.opcode, inst);
        }
        map
    };
}

fn get_inst(opcode: u8) -> &'static Instruction {
    *INST_MAP
        .get(&opcode)
        .expect(&format!("Invalid opcode 0x{:02X}", opcode))
}

struct Flags {
    c: bool,
    z: bool,
    i: bool,
    d: bool,
    r: bool,
    v: bool,
    n: bool,
}

impl Flags {
    fn set(&mut self, data: u8) {
        self.c = (data & 0x01) != 0;
        self.z = (data & 0x02) != 0;
        self.i = (data & 0x04) != 0;
        self.d = (data & 0x08) != 0;
        self.v = (data & 0x40) != 0;
        self.n = (data & 0x80) != 0;
    }

    fn get(&self) -> u8 {
        let mut data: u8 = 0;
        if self.c { data |= 0x01 };
        if self.z { data |= 0x02 };
        if self.i { data |= 0x04 };
        if self.d { data |= 0x08 };
        if self.r { data |= 0x20 };
        if self.v { data |= 0x40 };
        if self.n { data |= 0x80 };
        data
    }
}

pub struct CPU<'a> {
    a: u8,
    x: u8,
    y: u8,
    s: u8,
    p: Flags,
    pub pc: u16,
    bus: Bus<'a>,
    addr: u16,
    cycle_mode: CycleMode,
    extra_cycle: usize,
}

impl<'a> CPU<'a> {
    pub fn new<'b>(bus: Bus<'b>) -> CPU<'b> {
        let mut cpu: CPU<'b> = CPU {
            a: 0,
            x: 0,
            y: 0,
            s: 0xfd,
            p: Flags { c: false, z: false, i: true, d: false, r: true, v: false, n: false, },
            pc: 0,
            bus,
            addr: 0,
            cycle_mode: CycleMode::None,
            extra_cycle: 0,
        };
        cpu.pc = cpu.read16(0xfffc);
        cpu
    }

    pub fn read8(&mut self, addr: u16) -> u8 {
        self.bus.read8(addr)
    }

    fn read16(&mut self, addr: u16) -> u16 {
        let low = self.read8(addr) as u16;
        let high = self.read8(addr.wrapping_add(1)) as u16;
        low + (high << 8)
    }

    fn bug_read16(&mut self, addr: u16) -> u16 {
        if addr & 0x00ff == 0x00ff {
            self.read8(addr) as u16 + ((self.read8(addr & 0xff00) as u16) << 8)
        } else {
            self.read16(addr)
        }
    }

    pub fn write8(&mut self, addr: u16, data: u8) {
        self.bus.write8(addr, data);
    }

    fn push8(&mut self, data: u8) {
        self.write8(self.s as u16 + 0x0100, data);
        self.s = self.s.wrapping_sub(1);
    }

    fn push16(&mut self, data: u16) {
        self.push8((data >> 8) as u8);
        self.push8(data as u8);
    }

    fn pop8(&mut self) -> u8 {
        self.s = self.s.wrapping_add(1);
        self.read8(self.s as u16 + 0x0100)
    }

    fn pop16(&mut self) -> u16 {
        let low = self.pop8();
        let high = self.pop8();
        low as u16 + ((high as u16) << 8)
    }

    fn get_addr(&mut self, mode: &AddrMode) -> u16 {
        let addr = self.pc.wrapping_add(1);
        match mode {
            AddrMode::IMP => 0,
            AddrMode::ACC => 0,
            AddrMode::IMM => addr,
            AddrMode::ZPG => self.read8(addr) as u16,
            AddrMode::ZPX => self.read8(addr).wrapping_add(self.x) as u16,
            AddrMode::ZPY => self.read8(addr).wrapping_add(self.y) as u16,
            AddrMode::ABS => self.read16(addr),
            AddrMode::ABX => {
                let addr = self.read16(addr);
                let addr2 = addr.wrapping_add(self.x as u16);
                if self.cycle_mode == CycleMode::Page && addr & 0xff00 != addr2 & 0xff00 {
                    self.extra_cycle += 1;
                }
                addr2
            }
            AddrMode::ABY => {
                let addr = self.read16(addr);
                let addr2 = addr.wrapping_add(self.y as u16);
                if self.cycle_mode == CycleMode::Page && addr & 0xff00 != addr2 & 0xff00 {
                    self.extra_cycle += 1;
                }
                addr2
            }
            AddrMode::IND => {
                let addr = self.read16(addr);
                self.bug_read16(addr)
            }
            AddrMode::INX => {
                let addr = self.read8(addr);
                self.bug_read16(addr.wrapping_add(self.x) as u16)
            }
            AddrMode::INY => {
                let addr = self.read8(addr);
                let addr2 = self.bug_read16(addr as u16);
                let addr3 = addr2.wrapping_add(self.y as u16);
                if self.cycle_mode == CycleMode::Page && addr2 & 0xff00 != addr3 & 0xff00 {
                    self.extra_cycle += 1;
                }
                addr3
            }
            AddrMode::REL => self.read8(addr) as i8 as u16,
        }
    }

    fn update_zn(&mut self, data: u8) {
        self.p.z = data == 0;
        self.p.n = (data & 0x80) != 0;
    }

    fn nmi(&mut self) {
        self.push16(self.pc);
        self.push8(self.p.get());
        self.p.i = true;
        self.bus.tick(2);
        self.pc = self.read16(0xfffa);
    }

    fn apu_irq(&mut self) {
        if self.p.i {
            return;
        }
        self.push16(self.pc);
        self.push8(self.p.get());
        self.p.i = true;
        self.pc = self.read16(0xfffe);
    }

    pub fn run<F: FnMut(&mut CPU)>(&mut self, mut callback: F) {
        loop {
            if self.bus.ppu.clear_nmi {
                self.bus.ppu.nmi = false;
                self.bus.ppu.clear_nmi = false;
            }
            if self.bus.ppu.nmi {
                self.bus.ppu.nmi = false;
                self.nmi();
            }
            if self.bus.poll_apu_irq() {
                self.apu_irq();
            }
            let inst = get_inst(self.read8(self.pc));
            let b1 = self.read8(self.pc + 1);
            let b2 = self.read8(self.pc + 2);
            let b3 = self.read16(0xfffe);
            self.cycle_mode = inst.cycle_mode.clone();
            self.extra_cycle = 0;
            self.addr = self.get_addr(&inst.addr_mode);
            callback(self);
            (inst.function)(self);
            self.pc = self.pc.wrapping_add(inst.length);
            self.bus.tick(inst.cycle + self.extra_cycle);
        }
    }

    pub fn log(&mut self) {
        let inst = get_inst(self.read8(self.pc));
        let mut data: Vec<u8> = Vec::new();
        for b in 0..inst.length {
            data.push(self.read8(self.pc.wrapping_add(b)));
        }
        let pc = format!("{:04X}", self.pc);
        let mut binary: Vec<String> = Vec::new();
        for b in 0..inst.length as usize {
            binary.push(format!("{:02X}", data[b]));
        }
        let mut disasm: Vec<String> = Vec::new();
        if inst.mnemonic.starts_with("*") == false {
            disasm.push(String::from(""));
        }
        disasm.push(String::from(inst.mnemonic));
        let value = self.read8(self.addr);
        disasm.push(String::from(match &inst.addr_mode {
            AddrMode::IMP => format!(""),
            AddrMode::ACC => format!("A"),
            AddrMode::IMM => format!("#${:02X}", value),
            AddrMode::ZPG => format!("${:02X} = {:02X}", self.addr, value),
            AddrMode::ZPX => format!("${:02X},X @ {:02X} = {:02X}", data[1], self.addr, value),
            AddrMode::ZPY => format!("${:02X},Y @ {:02X} = {:02X}", data[1], self.addr, value),
            AddrMode::ABS => if inst.mnemonic.starts_with("J") { format!("${:04X}", self.addr) } else { format!("${:04X} = {:02X}", self.addr, value) },
            AddrMode::ABX => format!("${:02X}{:02X},X @ {:04X} = {:02X}", data[2], data[1], self.addr, value),
            AddrMode::ABY => format!("${:02X}{:02X},Y @ {:04X} = {:02X}", data[2], data[1], self.addr, value),
            AddrMode::IND => format!("(${:02X}{:02X}) = {:04X}", data[2], data[1], self.addr),
            AddrMode::INX => format!("(${:02X},X) @ {:02X} = {:04X} = {:02X}", data[1], data[1].wrapping_add(self.x), self.addr, value),
            AddrMode::INY => format!("(${:02X}),Y = {:04X} @ {:04X} = {:02X}", data[1], self.bug_read16(data[1] as u16), self.addr, value),
            AddrMode::REL => format!("${:04X}", self.pc.wrapping_add(2).wrapping_add(self.addr)),
        }));
        let status = format!("A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X}", self.a, self.x, self.y, self.p.get(), self.s);
        println!("{}", format!("{:6}{:9}{:33}{}", pc, binary.join(" "), disasm.join(" "), status));
    }
}

fn adc(cpu: &mut CPU) {
    let a = cpu.a as u16;
    let b = cpu.read8(cpu.addr) as u16;
    let r = a.wrapping_add(b).wrapping_add(cpu.p.c as u16);
    cpu.a = r as u8;
    cpu.p.c = r > 0xff;
    cpu.p.v = (a ^ r) & (b ^ r) & 0x80 != 0;
    cpu.update_zn(cpu.a);
}

fn and(cpu: &mut CPU) {
    cpu.a &= cpu.read8(cpu.addr);
    cpu.update_zn(cpu.a);
}

fn asl_acc(cpu: &mut CPU) {
    cpu.p.c = cpu.a & 0x80 != 0;
    cpu.a <<= 1;
    cpu.update_zn(cpu.a);
}

fn asl(cpu: &mut CPU) {
    let mut data = cpu.read8(cpu.addr);
    cpu.p.c = data & 0x80 != 0;
    data <<= 1;
    cpu.write8(cpu.addr, data);
    cpu.update_zn(data);
}

fn branch(cpu: &mut CPU, cond: bool) {
    if cond {
        cpu.extra_cycle += 1;
        if cpu.pc.wrapping_add(2) & 0xff00 != (cpu.pc.wrapping_add(2).wrapping_add(cpu.addr) & 0xff00) {
            cpu.extra_cycle += 1;
        }
        cpu.pc = cpu.pc.wrapping_add(cpu.addr);
    }
}

fn bcc(cpu: &mut CPU) {
    branch(cpu, cpu.p.c == false);
}

fn bcs(cpu: &mut CPU) {
    branch(cpu, cpu.p.c == true);
}

fn beq(cpu: &mut CPU) {
    branch(cpu, cpu.p.z == true);
}

fn bit(cpu: &mut CPU) {
    let data = cpu.read8(cpu.addr);
    cpu.p.z = cpu.a & data == 0;
    cpu.p.v = data & 0x40 != 0;
    cpu.p.n = data & 0x80 != 0;
}

fn bmi(cpu: &mut CPU) {
    branch(cpu, cpu.p.n == true);
}

fn bne(cpu: &mut CPU) {
    branch(cpu, cpu.p.z == false);
}

fn bpl(cpu: &mut CPU) {
    branch(cpu, cpu.p.n == false);
}

fn brk(cpu: &mut CPU) {
    // if cpu.p.i {
    //     return;
    // }
    cpu.push16(cpu.pc.wrapping_add(2));
    cpu.push8(cpu.p.get() | 0x10);
    cpu.p.i = true;
    cpu.pc = cpu.read16(0xfffe).wrapping_sub(1);
}

fn bvc(cpu: &mut CPU) {
    branch(cpu, cpu.p.v == false);
}

fn bvs(cpu: &mut CPU) {
    branch(cpu, cpu.p.v == true);
}

fn clc(cpu: &mut CPU) {
    cpu.p.c = false;
}

fn cld(cpu: &mut CPU) {
    cpu.p.d = false;
}

fn cli(cpu: &mut CPU) {
    cpu.p.i = false;
}

fn clv(cpu: &mut CPU) {
    cpu.p.v = false;
}

fn compare(cpu: &mut CPU, register: u8) {
    let data = cpu.read8(cpu.addr);
    cpu.p.c = register >= data;
    cpu.update_zn(register.wrapping_sub(data));
}

fn cmp(cpu: &mut CPU) {
    compare(cpu, cpu.a);
}

fn cpx(cpu: &mut CPU) {
    compare(cpu, cpu.x);
}

fn cpy(cpu: &mut CPU) {
    compare(cpu, cpu.y);
}

fn dec(cpu: &mut CPU) {
    let data = cpu.read8(cpu.addr).wrapping_sub(1);
    cpu.write8(cpu.addr, data);
    cpu.update_zn(data);
}

fn dex(cpu: &mut CPU) {
    cpu.x = cpu.x.wrapping_sub(1);
    cpu.update_zn(cpu.x);
}

fn dey(cpu: &mut CPU) {
    cpu.y = cpu.y.wrapping_sub(1);
    cpu.update_zn(cpu.y);
}

fn eor(cpu: &mut CPU) {
    cpu.a ^= cpu.read8(cpu.addr);
    cpu.update_zn(cpu.a);
}

fn inc(cpu: &mut CPU) {
    let data = cpu.read8(cpu.addr).wrapping_add(1);
    cpu.write8(cpu.addr, data);
    cpu.update_zn(data);
}

fn inx(cpu: &mut CPU) {
    cpu.x = cpu.x.wrapping_add(1);
    cpu.update_zn(cpu.x);
}

fn iny(cpu: &mut CPU) {
    cpu.y = cpu.y.wrapping_add(1);
    cpu.update_zn(cpu.y);
}

fn jmp(cpu: &mut CPU) {
    cpu.pc = cpu.addr.wrapping_sub(3);
}

fn jsr(cpu: &mut CPU) {
    cpu.push16(cpu.pc.wrapping_add(2));
    cpu.pc = cpu.addr.wrapping_sub(3);
}

fn lda(cpu: &mut CPU) {
    cpu.a = cpu.read8(cpu.addr);
    cpu.update_zn(cpu.a);
}

fn ldx(cpu: &mut CPU) {
    cpu.x = cpu.read8(cpu.addr);
    cpu.update_zn(cpu.x);
}

fn ldy(cpu: &mut CPU) {
    cpu.y = cpu.read8(cpu.addr);
    cpu.update_zn(cpu.y);
}

fn lsr_acc(cpu: &mut CPU) {
    cpu.p.c = cpu.a & 0x01 != 0;
    cpu.a >>= 1;
    cpu.update_zn(cpu.a);
}

fn lsr(cpu: &mut CPU) {
    let mut data = cpu.read8(cpu.addr);
    cpu.p.c = data & 0x01 != 0;
    data >>= 1;
    cpu.write8(cpu.addr, data);
    cpu.update_zn(data);
}

fn nop(cpu: &mut CPU) {}

fn ora(cpu: &mut CPU) {
    cpu.a |= cpu.read8(cpu.addr);
    cpu.update_zn(cpu.a);
}

fn pha(cpu: &mut CPU) {
    cpu.push8(cpu.a);
}

fn php(cpu: &mut CPU) {
    cpu.push8(cpu.p.get() | 0x10);
}

fn pla(cpu: &mut CPU) {
    cpu.a = cpu.pop8();
    cpu.update_zn(cpu.a);
}

fn plp(cpu: &mut CPU) {
    let data = cpu.pop8();
    cpu.p.set(data);
}

fn rol_acc(cpu: &mut CPU) {
    let carry = cpu.p.c as u8;
    cpu.p.c = cpu.a & 0x80 != 0;
    cpu.a = (cpu.a << 1) + carry;
    cpu.update_zn(cpu.a);
}

fn rol(cpu: &mut CPU) {
    let mut data = cpu.read8(cpu.addr);
    let carry = cpu.p.c as u8;
    cpu.p.c = data & 0x80 != 0;
    data = (data << 1) + carry;
    cpu.write8(cpu.addr, data);
    cpu.update_zn(data);
}

fn ror_acc(cpu: &mut CPU) {
    let carry = cpu.p.c as u8;
    cpu.p.c = cpu.a & 0x01 != 0;
    cpu.a = (cpu.a >> 1) + (carry << 7);
    cpu.update_zn(cpu.a);
}

fn ror(cpu: &mut CPU) {
    let mut data = cpu.read8(cpu.addr);
    let carry = cpu.p.c as u8;
    cpu.p.c = data & 0x01 != 0;
    data = (data >> 1) + (carry << 7);
    cpu.write8(cpu.addr, data);
    cpu.update_zn(data);
}

fn rti(cpu: &mut CPU) {
    let data = cpu.pop8();
    cpu.p.set(data);
    cpu.pc = cpu.pop16().wrapping_sub(1);
}

fn rts(cpu: &mut CPU) {
    cpu.pc = cpu.pop16();
}

fn sbc(cpu: &mut CPU) {
    let a = cpu.a as u16;
    let b = cpu.read8(cpu.addr) as u16;
    let r = a.wrapping_sub(b).wrapping_sub(!cpu.p.c as u16);
    cpu.a = r as u8;
    cpu.p.c = r <= 0xff;
    cpu.p.v = (a ^ b) & (a ^ r) & 0x80 != 0;
    cpu.update_zn(cpu.a);
}

fn sec(cpu: &mut CPU) {
    cpu.p.c = true;
}

fn sed(cpu: &mut CPU) {
    cpu.p.d = true;
}

fn sei(cpu: &mut CPU) {
    cpu.p.i = true;
}

fn sta(cpu: &mut CPU) {
    cpu.write8(cpu.addr, cpu.a);
}

fn stx(cpu: &mut CPU) {
    cpu.write8(cpu.addr, cpu.x);
}

fn sty(cpu: &mut CPU) {
    cpu.write8(cpu.addr, cpu.y);
}

fn tax(cpu: &mut CPU) {
    cpu.x = cpu.a;
    cpu.update_zn(cpu.x);
}

fn tay(cpu: &mut CPU) {
    cpu.y = cpu.a;
    cpu.update_zn(cpu.y);
}

fn tsx(cpu: &mut CPU) {
    cpu.x = cpu.s;
    cpu.update_zn(cpu.x);
}

fn txa(cpu: &mut CPU) {
    cpu.a = cpu.x;
    cpu.update_zn(cpu.a);
}

fn txs(cpu: &mut CPU) {
    cpu.s = cpu.x;
}

fn tya(cpu: &mut CPU) {
    cpu.a = cpu.y;
    cpu.update_zn(cpu.a);
}

fn dcp(cpu: &mut CPU) {
    dec(cpu);
    cmp(cpu);
}

fn isb(cpu: &mut CPU) {
    inc(cpu);
    sbc(cpu);
}

fn jam(cpu: &mut CPU) {
    panic!("jam() is called");
}

fn lax(cpu: &mut CPU) {
    lda(cpu);
    ldx(cpu);
}

fn rla(cpu: &mut CPU) {
    rol(cpu);
    and(cpu);
}

fn rra(cpu: &mut CPU) {
    ror(cpu);
    adc(cpu);
}

fn sax(cpu: &mut CPU) {
    cpu.write8(cpu.addr, cpu.a & cpu.x);
}

fn slo(cpu: &mut CPU) {
    asl(cpu);
    ora(cpu);
}

fn sre(cpu: &mut CPU) {
    lsr(cpu);
    eor(cpu);
}
