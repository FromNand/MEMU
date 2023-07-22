use crate::bus::Bus;
use lazy_static::lazy_static;
use std::collections::HashMap;

enum AddrMode {
    IMP,
    ACC,
    IMM,
    ZPG,
    ZPX,
    ZPY,
    ABS,
    ABX,
    ABY,
    IND,
    INX,
    INY,
    REL,
}

enum CycleMode {
    None,
    Page,
    Branch,
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
    fn new(
        opcode: u8,
        mnemonic: &'static str,
        addr_mode: AddrMode,
        function: fn(&mut CPU),
        length: u16,
        cycle: usize,
        cycle_mode: CycleMode,
    ) -> Self {
        Instruction {
            opcode,
            mnemonic,
            addr_mode,
            function,
            length,
            cycle,
            cycle_mode,
        }
    }
}

#[rustfmt::skip]
lazy_static! {
    static ref INST_LIST: Vec<Instruction> = vec![
        Instruction::new(0x69, "ADC", AddrMode::IMM, CPU::adc, 2, 2, CycleMode::None),
        Instruction::new(0x65, "ADC", AddrMode::ZPG, CPU::adc, 2, 3, CycleMode::None),
        Instruction::new(0x75, "ADC", AddrMode::ZPX, CPU::adc, 2, 4, CycleMode::None),
        Instruction::new(0x6d, "ADC", AddrMode::ABS, CPU::adc, 3, 4, CycleMode::None),
        Instruction::new(0x7d, "ADC", AddrMode::ABX, CPU::adc, 3, 4, CycleMode::Page),
        Instruction::new(0x79, "ADC", AddrMode::ABY, CPU::adc, 3, 4, CycleMode::Page),
        Instruction::new(0x61, "ADC", AddrMode::INX, CPU::adc, 2, 6, CycleMode::None),
        Instruction::new(0x71, "ADC", AddrMode::INY, CPU::adc, 2, 5, CycleMode::Page),
        Instruction::new(0x29, "AND", AddrMode::IMM, CPU::and, 2, 2, CycleMode::None),
        Instruction::new(0x25, "AND", AddrMode::ZPG, CPU::and, 2, 3, CycleMode::None),
        Instruction::new(0x35, "AND", AddrMode::ZPX, CPU::and, 2, 4, CycleMode::None),
        Instruction::new(0x2d, "AND", AddrMode::ABS, CPU::and, 3, 4, CycleMode::None),
        Instruction::new(0x3d, "AND", AddrMode::ABX, CPU::and, 3, 4, CycleMode::Page),
        Instruction::new(0x39, "AND", AddrMode::ABY, CPU::and, 3, 4, CycleMode::Page),
        Instruction::new(0x21, "AND", AddrMode::INX, CPU::and, 2, 6, CycleMode::None),
        Instruction::new(0x31, "AND", AddrMode::INY, CPU::and, 2, 5, CycleMode::Page),
        Instruction::new(0x0a, "ASL", AddrMode::ACC, CPU::asl_acc, 1, 2, CycleMode::None),
        Instruction::new(0x06, "ASL", AddrMode::ZPG, CPU::asl, 2, 5, CycleMode::None),
        Instruction::new(0x16, "ASL", AddrMode::ZPX, CPU::asl, 2, 6, CycleMode::None),
        Instruction::new(0x0e, "ASL", AddrMode::ABS, CPU::asl, 3, 6, CycleMode::None),
        Instruction::new(0x1e, "ASL", AddrMode::ABX, CPU::asl, 3, 7, CycleMode::None),
        Instruction::new(0x90, "BCC", AddrMode::REL, CPU::bcc, 2, 2, CycleMode::Branch),
        Instruction::new(0xb0, "BCS", AddrMode::REL, CPU::bcs, 2, 2, CycleMode::Branch),
        Instruction::new(0xf0, "BEQ", AddrMode::REL, CPU::beq, 2, 2, CycleMode::Branch),
        Instruction::new(0x24, "BIT", AddrMode::ZPG, CPU::bit, 2, 3, CycleMode::None),
        Instruction::new(0x2c, "BIT", AddrMode::ABS, CPU::bit, 3, 4, CycleMode::None),
        Instruction::new(0x30, "BMI", AddrMode::REL, CPU::bmi, 2, 2, CycleMode::Branch),
        Instruction::new(0xd0, "BNE", AddrMode::REL, CPU::bne, 2, 2, CycleMode::Branch),
        Instruction::new(0x10, "BPL", AddrMode::REL, CPU::bpl, 2, 2, CycleMode::Branch),
        Instruction::new(0x00, "BRK", AddrMode::IMP, CPU::brk, 1, 7, CycleMode::None),
        Instruction::new(0x50, "BVC", AddrMode::REL, CPU::bvc, 2, 2, CycleMode::Branch),
        Instruction::new(0x70, "BVS", AddrMode::REL, CPU::bvs, 2, 2, CycleMode::Branch),
        Instruction::new(0x18, "CLC", AddrMode::IMP, CPU::clc, 1, 2, CycleMode::None),
        Instruction::new(0xd8, "CLD", AddrMode::IMP, CPU::cld, 1, 2, CycleMode::None),
        Instruction::new(0x58, "CLI", AddrMode::IMP, CPU::cli, 1, 2, CycleMode::None),
        Instruction::new(0xb8, "CLV", AddrMode::IMP, CPU::clv, 1, 2, CycleMode::None),
        Instruction::new(0xc9, "CMP", AddrMode::IMM, CPU::cmp, 2, 2, CycleMode::None),
        Instruction::new(0xc5, "CMP", AddrMode::ZPG, CPU::cmp, 2, 3, CycleMode::None),
        Instruction::new(0xd5, "CMP", AddrMode::ZPX, CPU::cmp, 2, 4, CycleMode::None),
        Instruction::new(0xcd, "CMP", AddrMode::ABS, CPU::cmp, 3, 4, CycleMode::None),
        Instruction::new(0xdd, "CMP", AddrMode::ABX, CPU::cmp, 3, 4, CycleMode::Page),
        Instruction::new(0xd9, "CMP", AddrMode::ABY, CPU::cmp, 3, 4, CycleMode::Page),
        Instruction::new(0xc1, "CMP", AddrMode::INX, CPU::cmp, 2, 6, CycleMode::None),
        Instruction::new(0xd1, "CMP", AddrMode::INY, CPU::cmp, 2, 5, CycleMode::Page),
        Instruction::new(0xe0, "CPX", AddrMode::IMM, CPU::cpx, 2, 2, CycleMode::None),
        Instruction::new(0xe4, "CPX", AddrMode::ZPG, CPU::cpx, 2, 3, CycleMode::None),
        Instruction::new(0xec, "CPX", AddrMode::ABS, CPU::cpx, 3, 4, CycleMode::None),
        Instruction::new(0xc0, "CPY", AddrMode::IMM, CPU::cpy, 2, 2, CycleMode::None),
        Instruction::new(0xc4, "CPY", AddrMode::ZPG, CPU::cpy, 2, 3, CycleMode::None),
        Instruction::new(0xcc, "CPY", AddrMode::ABS, CPU::cpy, 3, 4, CycleMode::None),
        Instruction::new(0xc6, "DEC", AddrMode::ZPG, CPU::dec, 2, 5, CycleMode::None),
        Instruction::new(0xd6, "DEC", AddrMode::ZPX, CPU::dec, 2, 6, CycleMode::None),
        Instruction::new(0xce, "DEC", AddrMode::ABS, CPU::dec, 3, 6, CycleMode::None),
        Instruction::new(0xde, "DEC", AddrMode::ABX, CPU::dec, 3, 7, CycleMode::None),
        Instruction::new(0xca, "DEX", AddrMode::IMP, CPU::dex, 1, 2, CycleMode::None),
        Instruction::new(0x88, "DEY", AddrMode::IMP, CPU::dey, 1, 2, CycleMode::None),
        Instruction::new(0x49, "EOR", AddrMode::IMM, CPU::eor, 2, 2, CycleMode::None),
        Instruction::new(0x45, "EOR", AddrMode::ZPG, CPU::eor, 2, 3, CycleMode::None),
        Instruction::new(0x55, "EOR", AddrMode::ZPX, CPU::eor, 2, 4, CycleMode::None),
        Instruction::new(0x4d, "EOR", AddrMode::ABS, CPU::eor, 3, 4, CycleMode::None),
        Instruction::new(0x5d, "EOR", AddrMode::ABX, CPU::eor, 3, 4, CycleMode::None),
        Instruction::new(0x59, "EOR", AddrMode::ABY, CPU::eor, 3, 4, CycleMode::None),
        Instruction::new(0x41, "EOR", AddrMode::INX, CPU::eor, 2, 6, CycleMode::None),
        Instruction::new(0x51, "EOR", AddrMode::INY, CPU::eor, 2, 5, CycleMode::None),
        Instruction::new(0xe6, "INC", AddrMode::ZPG, CPU::inc, 2, 5, CycleMode::None),
        Instruction::new(0xf6, "INC", AddrMode::ZPX, CPU::inc, 2, 6, CycleMode::None),
        Instruction::new(0xee, "INC", AddrMode::ABS, CPU::inc, 3, 6, CycleMode::None),
        Instruction::new(0xfe, "INC", AddrMode::ABX, CPU::inc, 3, 7, CycleMode::None),
        Instruction::new(0xe8, "INX", AddrMode::IMP, CPU::inx, 1, 2, CycleMode::None),
        Instruction::new(0xc8, "INY", AddrMode::IMP, CPU::iny, 1, 2, CycleMode::None),
        Instruction::new(0x4c, "JMP", AddrMode::ABS, CPU::jmp, 3, 3, CycleMode::None),
        Instruction::new(0x6c, "JMP", AddrMode::IND, CPU::jmp, 3, 5, CycleMode::None),
        Instruction::new(0x20, "JSR", AddrMode::ABS, CPU::jsr, 3, 6, CycleMode::None),
        Instruction::new(0xa9, "LDA", AddrMode::IMM, CPU::lda, 2, 2, CycleMode::None),
        Instruction::new(0xa5, "LDA", AddrMode::ZPG, CPU::lda, 2, 3, CycleMode::None),
        Instruction::new(0xb5, "LDA", AddrMode::ZPX, CPU::lda, 2, 4, CycleMode::None),
        Instruction::new(0xad, "LDA", AddrMode::ABS, CPU::lda, 3, 4, CycleMode::None),
        Instruction::new(0xbd, "LDA", AddrMode::ABX, CPU::lda, 3, 4, CycleMode::Page),
        Instruction::new(0xb9, "LDA", AddrMode::ABY, CPU::lda, 3, 4, CycleMode::Page),
        Instruction::new(0xa1, "LDA", AddrMode::INX, CPU::lda, 2, 6, CycleMode::None),
        Instruction::new(0xb1, "LDA", AddrMode::INY, CPU::lda, 2, 5, CycleMode::Page),
        Instruction::new(0xa2, "LDX", AddrMode::IMM, CPU::ldx, 2, 2, CycleMode::None),
        Instruction::new(0xa6, "LDX", AddrMode::ZPG, CPU::ldx, 2, 3, CycleMode::None),
        Instruction::new(0xb6, "LDX", AddrMode::ZPY, CPU::ldx, 2, 4, CycleMode::None),
        Instruction::new(0xae, "LDX", AddrMode::ABS, CPU::ldx, 3, 4, CycleMode::None),
        Instruction::new(0xbe, "LDX", AddrMode::ABY, CPU::ldx, 3, 4, CycleMode::Page),
        Instruction::new(0xa0, "LDY", AddrMode::IMM, CPU::ldy, 2, 2, CycleMode::None),
        Instruction::new(0xa4, "LDY", AddrMode::ZPG, CPU::ldy, 2, 3, CycleMode::None),
        Instruction::new(0xb4, "LDY", AddrMode::ZPX, CPU::ldy, 2, 4, CycleMode::None),
        Instruction::new(0xac, "LDY", AddrMode::ABS, CPU::ldy, 3, 4, CycleMode::None),
        Instruction::new(0xbc, "LDY", AddrMode::ABX, CPU::ldy, 3, 4, CycleMode::Page),
        Instruction::new(0x4a, "LSR", AddrMode::ACC, CPU::lsr_acc, 1, 2, CycleMode::None),
        Instruction::new(0x46, "LSR", AddrMode::ZPG, CPU::lsr, 2, 5, CycleMode::None),
        Instruction::new(0x56, "LSR", AddrMode::ZPX, CPU::lsr, 2, 6, CycleMode::None),
        Instruction::new(0x4e, "LSR", AddrMode::ABS, CPU::lsr, 3, 6, CycleMode::None),
        Instruction::new(0x5e, "LSR", AddrMode::ABX, CPU::lsr, 3, 7, CycleMode::None),
        Instruction::new(0xea, "NOP", AddrMode::IMP, CPU::nop, 1, 2, CycleMode::None),
        Instruction::new(0x09, "ORA", AddrMode::IMM, CPU::ora, 2, 2, CycleMode::None),
        Instruction::new(0x05, "ORA", AddrMode::ZPG, CPU::ora, 2, 3, CycleMode::None),
        Instruction::new(0x15, "ORA", AddrMode::ZPX, CPU::ora, 2, 4, CycleMode::None),
        Instruction::new(0x0d, "ORA", AddrMode::ABS, CPU::ora, 3, 4, CycleMode::None),
        Instruction::new(0x1d, "ORA", AddrMode::ABX, CPU::ora, 3, 4, CycleMode::Page),
        Instruction::new(0x19, "ORA", AddrMode::ABY, CPU::ora, 3, 4, CycleMode::Page),
        Instruction::new(0x01, "ORA", AddrMode::INX, CPU::ora, 2, 6, CycleMode::None),
        Instruction::new(0x11, "ORA", AddrMode::INY, CPU::ora, 2, 5, CycleMode::Page),
        Instruction::new(0x48, "PHA", AddrMode::IMP, CPU::pha, 1, 3, CycleMode::None),
        Instruction::new(0x08, "PHP", AddrMode::IMP, CPU::php, 1, 3, CycleMode::None),
        Instruction::new(0x68, "PLA", AddrMode::IMP, CPU::pla, 1, 4, CycleMode::None),
        Instruction::new(0x28, "PLP", AddrMode::IMP, CPU::plp, 1, 4, CycleMode::None),
        Instruction::new(0x2a, "ROL", AddrMode::ACC, CPU::rol_acc, 1, 2, CycleMode::None),
        Instruction::new(0x26, "ROL", AddrMode::ZPG, CPU::rol, 2, 5, CycleMode::None),
        Instruction::new(0x36, "ROL", AddrMode::ZPX, CPU::rol, 2, 6, CycleMode::None),
        Instruction::new(0x2e, "ROL", AddrMode::ABS, CPU::rol, 3, 6, CycleMode::None),
        Instruction::new(0x3e, "ROL", AddrMode::ABX, CPU::rol, 3, 7, CycleMode::None),
        Instruction::new(0x6a, "ROR", AddrMode::ACC, CPU::ror_acc, 1, 2, CycleMode::None),
        Instruction::new(0x66, "ROR", AddrMode::ZPG, CPU::ror, 2, 5, CycleMode::None),
        Instruction::new(0x76, "ROR", AddrMode::ZPX, CPU::ror, 2, 6, CycleMode::None),
        Instruction::new(0x6e, "ROR", AddrMode::ABS, CPU::ror, 3, 6, CycleMode::None),
        Instruction::new(0x7e, "ROR", AddrMode::ABX, CPU::ror, 3, 7, CycleMode::None),
        Instruction::new(0x40, "RTI", AddrMode::IMP, CPU::rti, 1, 6, CycleMode::None),
        Instruction::new(0x60, "RTS", AddrMode::IMP, CPU::rts, 1, 6, CycleMode::None),
        Instruction::new(0xe9, "SBC", AddrMode::IMM, CPU::sbc, 2, 2, CycleMode::None),
        Instruction::new(0xe5, "SBC", AddrMode::ZPG, CPU::sbc, 2, 3, CycleMode::None),
        Instruction::new(0xf5, "SBC", AddrMode::ZPX, CPU::sbc, 2, 4, CycleMode::None),
        Instruction::new(0xed, "SBC", AddrMode::ABS, CPU::sbc, 3, 4, CycleMode::None),
        Instruction::new(0xfd, "SBC", AddrMode::ABX, CPU::sbc, 3, 4, CycleMode::Page),
        Instruction::new(0xf9, "SBC", AddrMode::ABY, CPU::sbc, 3, 4, CycleMode::Page),
        Instruction::new(0xe1, "SBC", AddrMode::INX, CPU::sbc, 2, 6, CycleMode::None),
        Instruction::new(0xf1, "SBC", AddrMode::INY, CPU::sbc, 2, 5, CycleMode::Page),
        Instruction::new(0x38, "SEC", AddrMode::IMP, CPU::sec, 1, 2, CycleMode::None),
        Instruction::new(0xf8, "SED", AddrMode::IMP, CPU::sed, 1, 2, CycleMode::None),
        Instruction::new(0x78, "SEI", AddrMode::IMP, CPU::sei, 1, 2, CycleMode::None),
        Instruction::new(0x85, "STA", AddrMode::ZPG, CPU::sta, 2, 3, CycleMode::None),
        Instruction::new(0x95, "STA", AddrMode::ZPX, CPU::sta, 2, 4, CycleMode::None),
        Instruction::new(0x8d, "STA", AddrMode::ABS, CPU::sta, 3, 4, CycleMode::None),
        Instruction::new(0x9d, "STA", AddrMode::ABX, CPU::sta, 3, 5, CycleMode::None),
        Instruction::new(0x99, "STA", AddrMode::ABY, CPU::sta, 3, 5, CycleMode::None),
        Instruction::new(0x81, "STA", AddrMode::INX, CPU::sta, 2, 6, CycleMode::None),
        Instruction::new(0x91, "STA", AddrMode::INY, CPU::sta, 2, 6, CycleMode::None),
        Instruction::new(0x86, "STX", AddrMode::ZPG, CPU::stx, 2, 3, CycleMode::None),
        Instruction::new(0x96, "STX", AddrMode::ZPY, CPU::stx, 2, 4, CycleMode::None),
        Instruction::new(0x8e, "STX", AddrMode::ABS, CPU::stx, 3, 4, CycleMode::None),
        Instruction::new(0x84, "STY", AddrMode::ZPG, CPU::sty, 2, 3, CycleMode::None),
        Instruction::new(0x94, "STY", AddrMode::ZPX, CPU::sty, 2, 4, CycleMode::None),
        Instruction::new(0x8c, "STY", AddrMode::ABS, CPU::sty, 3, 4, CycleMode::None),
        Instruction::new(0xaa, "TAX", AddrMode::IMP, CPU::tax, 1, 2, CycleMode::None),
        Instruction::new(0xa8, "TAY", AddrMode::IMP, CPU::tay, 1, 2, CycleMode::None),
        Instruction::new(0xba, "TSX", AddrMode::IMP, CPU::tsx, 1, 2, CycleMode::None),
        Instruction::new(0x8a, "TXA", AddrMode::IMP, CPU::txa, 1, 2, CycleMode::None),
        Instruction::new(0x9a, "TXS", AddrMode::IMP, CPU::txs, 1, 2, CycleMode::None),
        Instruction::new(0x98, "TYA", AddrMode::IMP, CPU::tya, 1, 2, CycleMode::None),
        Instruction::new(0xc7, "*DCP", AddrMode::ZPG, CPU::dcp, 2, 5, CycleMode::None),
        Instruction::new(0xd7, "*DCP", AddrMode::ZPX, CPU::dcp, 2, 6, CycleMode::None),
        Instruction::new(0xcf, "*DCP", AddrMode::ABS, CPU::dcp, 3, 6, CycleMode::None),
        Instruction::new(0xdf, "*DCP", AddrMode::ABX, CPU::dcp, 3, 7, CycleMode::None),
        Instruction::new(0xdb, "*DCP", AddrMode::ABY, CPU::dcp, 3, 7, CycleMode::None),
        Instruction::new(0xc3, "*DCP", AddrMode::INX, CPU::dcp, 2, 8, CycleMode::None),
        Instruction::new(0xd3, "*DCP", AddrMode::INY, CPU::dcp, 2, 8, CycleMode::None),
        Instruction::new(0xe7, "*ISB", AddrMode::ZPG, CPU::isb, 2, 5, CycleMode::None),
        Instruction::new(0xf7, "*ISB", AddrMode::ZPX, CPU::isb, 2, 6, CycleMode::None),
        Instruction::new(0xef, "*ISB", AddrMode::ABS, CPU::isb, 3, 6, CycleMode::None),
        Instruction::new(0xff, "*ISB", AddrMode::ABX, CPU::isb, 3, 7, CycleMode::None),
        Instruction::new(0xfb, "*ISB", AddrMode::ABY, CPU::isb, 3, 7, CycleMode::None),
        Instruction::new(0xe3, "*ISB", AddrMode::INX, CPU::isb, 2, 8, CycleMode::None),
        Instruction::new(0xf3, "*ISB", AddrMode::INY, CPU::isb, 2, 8, CycleMode::None),
        Instruction::new(0xa7, "*LAX", AddrMode::ZPG, CPU::lax, 2, 3, CycleMode::None),
        Instruction::new(0xb7, "*LAX", AddrMode::ZPY, CPU::lax, 2, 4, CycleMode::None),
        Instruction::new(0xaf, "*LAX", AddrMode::ABS, CPU::lax, 3, 4, CycleMode::None),
        Instruction::new(0xbf, "*LAX", AddrMode::ABY, CPU::lax, 3, 4, CycleMode::Page),
        Instruction::new(0xa3, "*LAX", AddrMode::INX, CPU::lax, 2, 6, CycleMode::None),
        Instruction::new(0xb3, "*LAX", AddrMode::INY, CPU::lax, 2, 5, CycleMode::Page),
        Instruction::new(0x1a, "*NOP", AddrMode::IMP, CPU::nop, 1, 2, CycleMode::None),
        Instruction::new(0x3a, "*NOP", AddrMode::IMP, CPU::nop, 1, 2, CycleMode::None),
        Instruction::new(0x5a, "*NOP", AddrMode::IMP, CPU::nop, 1, 2, CycleMode::None),
        Instruction::new(0x7a, "*NOP", AddrMode::IMP, CPU::nop, 1, 2, CycleMode::None),
        Instruction::new(0xda, "*NOP", AddrMode::IMP, CPU::nop, 1, 2, CycleMode::None),
        Instruction::new(0xfa, "*NOP", AddrMode::IMP, CPU::nop, 1, 2, CycleMode::None),
        Instruction::new(0x27, "*RLA", AddrMode::ZPG, CPU::rla, 2, 5, CycleMode::None),
        Instruction::new(0x37, "*RLA", AddrMode::ZPX, CPU::rla, 2, 6, CycleMode::None),
        Instruction::new(0x2f, "*RLA", AddrMode::ABS, CPU::rla, 3, 6, CycleMode::None),
        Instruction::new(0x3f, "*RLA", AddrMode::ABX, CPU::rla, 3, 7, CycleMode::None),
        Instruction::new(0x3b, "*RLA", AddrMode::ABY, CPU::rla, 3, 7, CycleMode::None),
        Instruction::new(0x23, "*RLA", AddrMode::INX, CPU::rla, 2, 8, CycleMode::None),
        Instruction::new(0x33, "*RLA", AddrMode::INY, CPU::rla, 2, 8, CycleMode::None),
        Instruction::new(0x67, "*RRA", AddrMode::ZPG, CPU::rra, 2, 5, CycleMode::None),
        Instruction::new(0x77, "*RRA", AddrMode::ZPX, CPU::rra, 2, 6, CycleMode::None),
        Instruction::new(0x6f, "*RRA", AddrMode::ABS, CPU::rra, 3, 6, CycleMode::None),
        Instruction::new(0x7f, "*RRA", AddrMode::ABX, CPU::rra, 3, 7, CycleMode::None),
        Instruction::new(0x7b, "*RRA", AddrMode::ABY, CPU::rra, 3, 7, CycleMode::None),
        Instruction::new(0x63, "*RRA", AddrMode::INX, CPU::rra, 2, 8, CycleMode::None),
        Instruction::new(0x73, "*RRA", AddrMode::INY, CPU::rra, 2, 8, CycleMode::None),
        Instruction::new(0x87, "*SAX", AddrMode::ZPG, CPU::sax, 2, 3, CycleMode::None),
        Instruction::new(0x97, "*SAX", AddrMode::ZPY, CPU::sax, 2, 4, CycleMode::None),
        Instruction::new(0x8f, "*SAX", AddrMode::ABS, CPU::sax, 3, 4, CycleMode::None),
        Instruction::new(0x83, "*SAX", AddrMode::INX, CPU::sax, 2, 6, CycleMode::None),
        Instruction::new(0xeb, "*SBC", AddrMode::IMM, CPU::sbc, 2, 2, CycleMode::None),
        Instruction::new(0x07, "*SLO", AddrMode::ZPG, CPU::slo, 2, 5, CycleMode::None),
        Instruction::new(0x17, "*SLO", AddrMode::ZPX, CPU::slo, 2, 6, CycleMode::None),
        Instruction::new(0x0f, "*SLO", AddrMode::ABS, CPU::slo, 3, 6, CycleMode::None),
        Instruction::new(0x1f, "*SLO", AddrMode::ABX, CPU::slo, 3, 7, CycleMode::None),
        Instruction::new(0x1b, "*SLO", AddrMode::ABY, CPU::slo, 3, 7, CycleMode::None),
        Instruction::new(0x03, "*SLO", AddrMode::INX, CPU::slo, 2, 8, CycleMode::None),
        Instruction::new(0x13, "*SLO", AddrMode::INY, CPU::slo, 2, 8, CycleMode::None),
        Instruction::new(0x47, "*SRE", AddrMode::ZPG, CPU::sre, 2, 5, CycleMode::None),
        Instruction::new(0x57, "*SRE", AddrMode::ZPX, CPU::sre, 2, 6, CycleMode::None),
        Instruction::new(0x4f, "*SRE", AddrMode::ABS, CPU::sre, 3, 6, CycleMode::None),
        Instruction::new(0x5f, "*SRE", AddrMode::ABX, CPU::sre, 3, 7, CycleMode::None),
        Instruction::new(0x5b, "*SRE", AddrMode::ABY, CPU::sre, 3, 7, CycleMode::None),
        Instruction::new(0x43, "*SRE", AddrMode::INX, CPU::sre, 2, 8, CycleMode::None),
        Instruction::new(0x53, "*SRE", AddrMode::INY, CPU::sre, 2, 8, CycleMode::None),
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
        .expect(&format!("Invalid opcode 0x{:02x}", opcode))
}

struct Flags {
    c: bool,
    z: bool,
    i: bool,
    d: bool,
    b: bool,
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
        self.b = (data & 0x10) != 0;
        //self.r = (data & 0x20) != 0;
        self.v = (data & 0x40) != 0;
        self.n = (data & 0x80) != 0;
    }

    fn get(&self) -> u8 {
        let mut data: u8 = 0;
        if self.c {
            data |= 0x01
        };
        if self.z {
            data |= 0x02
        };
        if self.i {
            data |= 0x04
        };
        if self.d {
            data |= 0x08
        };
        if self.b {
            data |= 0x10
        };
        if self.r {
            data |= 0x20
        };
        if self.v {
            data |= 0x40
        };
        if self.n {
            data |= 0x80
        };
        data
    }
}

pub struct CPU {
    a: u8,
    x: u8,
    y: u8,
    s: u8,
    p: Flags,
    pc: u16,
    bus: Bus,
    addr: u16,
    extra_cycle: usize,
}

impl CPU {
    pub fn new() -> Self {
        let mut cpu = CPU {
            a: 0,
            x: 0,
            y: 0,
            s: 0xfd,
            p: Flags {
                c: false,
                z: false,
                i: true,
                d: false,
                b: false,
                r: true,
                v: false,
                n: false,
            },
            pc: 0,
            bus: Bus::new(),
            addr: 0,
            extra_cycle: 0,
        };
        cpu.pc = cpu.read16(0xfffc);
        cpu
    }

    fn read8(&self, addr: u16) -> u8 {
        self.bus.read8(addr)
    }

    fn read16(&self, addr: u16) -> u16 {
        let low = self.read8(addr) as u16;
        let high = self.read8(addr.wrapping_add(1)) as u16;
        low + (high << 8)
    }

    fn write8(&mut self, addr: u16, data: u8) {
        self.bus.write8(addr, data);
    }

    fn write16(&mut self, addr: u16, data: u16) {
        self.write8(addr, data as u8);
        self.write8(addr.wrapping_add(1), (data >> 8) as u8);
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

    fn get_addr(&self, mode: &AddrMode) -> u16 {
        let addr = self.pc.wrapping_add(1);
        match mode {
            AddrMode::IMP => 0,
            AddrMode::ACC => 0,
            AddrMode::IMM => addr,
            AddrMode::ZPG => self.read8(addr) as u16,
            AddrMode::ZPX => self.read8(addr).wrapping_add(self.x) as u16,
            AddrMode::ZPY => self.read8(addr).wrapping_add(self.y) as u16,
            AddrMode::ABS => self.read16(addr),
            AddrMode::ABX => self.read16(addr).wrapping_add(self.x as u16),
            AddrMode::ABY => self.read16(addr).wrapping_add(self.y as u16),
            AddrMode::IND => self.read16(self.read16(addr)),
            AddrMode::INX => self.read16(self.read8(addr).wrapping_add(self.x) as u16),
            AddrMode::INY => self
                .read16(self.read8(addr) as u16)
                .wrapping_add(self.y as u16),
            AddrMode::REL => self.read8(addr) as i8 as u16,
        }
    }

    fn update_zn(&mut self, data: u8) {
        self.p.z = data == 0;
        self.p.n = (data & 0x80) != 0;
    }

    pub fn run<F: Fn(&mut CPU)>(&mut self, callback: F) {
        callback(self);
        loop {
            if self.read8(self.pc) == 0x00 {
                return;
            }
            println!(
                "pc = 0x{:04x}, opcode = 0x{:02x}",
                self.pc,
                self.read8(self.pc)
            );
            let inst = get_inst(self.read8(self.pc));
            self.addr = self.get_addr(&inst.addr_mode);
            self.extra_cycle = 0;
            (inst.function)(self);
            self.pc = self.pc.wrapping_add(inst.length);
        }
    }

    fn adc(&mut self) {
        let a = self.a as u16;
        let b = self.read8(self.addr) as u16;
        let r = a.wrapping_add(b).wrapping_add(self.p.c as u16);
        self.a = r as u8;
        self.p.c = r > 0xff;
        self.p.v = (a ^ r) & (b ^ r) & 0x80 != 0;
        self.update_zn(self.a);
    }

    fn and(&mut self) {
        self.a &= self.read8(self.addr);
        self.update_zn(self.a);
    }

    fn asl_acc(&mut self) {
        self.p.c = self.a & 0x80 != 0;
        self.a <<= 1;
        self.update_zn(self.a);
    }

    fn asl(&mut self) {
        let mut data = self.read8(self.addr);
        self.p.c = data & 0x80 != 0;
        data <<= 1;
        self.write8(self.addr, data);
        self.update_zn(data);
    }

    fn branch(&mut self, cond: bool) {
        if cond {
            self.pc = self.pc.wrapping_add(self.addr);
        }
    }

    fn bcc(&mut self) {
        self.branch(self.p.c == false);
    }

    fn bcs(&mut self) {
        self.branch(self.p.c == true);
    }

    fn beq(&mut self) {
        self.branch(self.p.z == true);
    }

    fn bit(&mut self) {
        let data = self.read8(self.addr);
        self.p.z = self.a & data == 0;
        self.p.v = data & 0x40 != 0;
        self.p.n = data & 0x80 != 0;
    }

    fn bmi(&mut self) {
        self.branch(self.p.n == true);
    }

    fn bne(&mut self) {
        self.branch(self.p.z == false);
    }

    fn bpl(&mut self) {
        self.branch(self.p.n == false);
    }

    fn brk(&mut self) {
        if self.p.i {
            return;
        }
        self.p.b = true;
        self.push16(self.pc.wrapping_add(2));
        self.push8(self.p.get());
        self.p.i = true;
        self.pc = self.read16(0xfffe).wrapping_sub(1);
    }

    fn bvc(&mut self) {
        self.branch(self.p.v == false);
    }

    fn bvs(&mut self) {
        self.branch(self.p.v == true);
    }

    fn clc(&mut self) {
        self.p.c = false;
    }

    fn cld(&mut self) {
        self.p.d = false;
    }

    fn cli(&mut self) {
        self.p.i = false;
    }

    fn clv(&mut self) {
        self.p.v = false;
    }

    fn compare(&mut self, register: u8) {
        let data = self.read8(self.addr);
        self.p.c = register >= data;
        self.update_zn(register.wrapping_sub(data));
    }

    fn cmp(&mut self) {
        self.compare(self.a);
    }

    fn cpx(&mut self) {
        self.compare(self.x);
    }

    fn cpy(&mut self) {
        self.compare(self.y);
    }

    fn dec(&mut self) {
        let data = self.read8(self.addr).wrapping_sub(1);
        self.write8(self.addr, data);
        self.update_zn(data);
    }

    fn dex(&mut self) {
        self.x = self.x.wrapping_sub(1);
        self.update_zn(self.x);
    }

    fn dey(&mut self) {
        self.y = self.y.wrapping_sub(1);
        self.update_zn(self.y);
    }

    fn eor(&mut self) {
        self.a ^= self.read8(self.addr);
        self.update_zn(self.a);
    }

    fn inc(&mut self) {
        let data = self.read8(self.addr).wrapping_add(1);
        self.write8(self.addr, data);
        self.update_zn(data);
    }

    fn inx(&mut self) {
        self.x = self.x.wrapping_add(1);
        self.update_zn(self.x);
    }

    fn iny(&mut self) {
        self.y = self.y.wrapping_add(1);
        self.update_zn(self.y);
    }

    fn jmp(&mut self) {
        self.pc = self.addr.wrapping_sub(3);
    }

    fn jsr(&mut self) {
        self.push16(self.pc.wrapping_add(2));
        self.pc = self.addr.wrapping_sub(3);
    }

    fn lda(&mut self) {
        self.a = self.read8(self.addr);
        self.update_zn(self.a);
    }

    fn ldx(&mut self) {
        self.x = self.read8(self.addr);
        self.update_zn(self.x);
    }

    fn ldy(&mut self) {
        self.y = self.read8(self.addr);
        self.update_zn(self.y);
    }

    fn lsr_acc(&mut self) {
        self.p.c = self.a & 0x01 != 0;
        self.a >>= 1;
        self.update_zn(self.a);
    }

    fn lsr(&mut self) {
        let mut data = self.read8(self.addr);
        self.p.c = data & 0x01 != 0;
        data >>= 1;
        self.write8(self.addr, data);
        self.update_zn(data);
    }

    fn nop(&mut self) {}

    fn ora(&mut self) {
        self.a |= self.read8(self.addr);
        self.update_zn(self.a);
    }

    fn pha(&mut self) {
        self.push8(self.a);
    }

    fn php(&mut self) {
        self.push8(self.p.get() | 0x10);
    }

    fn pla(&mut self) {
        self.a = self.pop8();
        self.update_zn(self.a);
    }

    fn plp(&mut self) {
        let data = self.pop8() & !0x10;
        self.p.set(data);
    }

    fn rol_acc(&mut self) {
        let carry = self.p.c as u8;
        self.p.c = self.a & 0x80 != 0;
        self.a = (self.a << 1) + carry;
        self.update_zn(self.a);
    }

    fn rol(&mut self) {
        let mut data = self.read8(self.addr);
        let carry = self.p.c as u8;
        self.p.c = data & 0x80 != 0;
        data = (data << 1) + carry;
        self.write8(self.addr, data);
        self.update_zn(data);
    }

    fn ror_acc(&mut self) {
        let carry = self.p.c as u8;
        self.p.c = self.a & 0x01 != 0;
        self.a = (self.a >> 1) + (carry << 7);
        self.update_zn(self.a);
    }

    fn ror(&mut self) {
        let mut data = self.read8(self.addr);
        let carry = self.p.c as u8;
        self.p.c = data & 0x01 != 0;
        data = (data >> 1) + (carry << 7);
        self.write8(self.addr, data);
        self.update_zn(data);
    }

    fn rti(&mut self) {
        let data = self.pop8() & !0x10;
        self.p.set(data);
        self.pc = self.pop16();
    }

    fn rts(&mut self) {
        self.pc = self.pop16().wrapping_add(1);
    }

    fn sbc(&mut self) {
        let a = self.a as u16;
        let b = self.read8(self.addr) as u16;
        let r = a.wrapping_sub(b).wrapping_sub(1 - self.p.c as u16);
        self.a = r as u8;
        self.p.c = r <= 0xff;
        self.p.v = (a ^ b) & (a ^ r) & 0x80 != 0;
        self.update_zn(self.a);
    }

    fn sec(&mut self) {
        self.p.c = true;
    }

    fn sed(&mut self) {
        self.p.d = true;
    }

    fn sei(&mut self) {
        self.p.i = true;
    }

    fn sta(&mut self) {
        self.write8(self.addr, self.a);
    }

    fn stx(&mut self) {
        self.write8(self.addr, self.x);
    }

    fn sty(&mut self) {
        self.write8(self.addr, self.y);
    }

    fn tax(&mut self) {
        self.x = self.a;
        self.update_zn(self.x);
    }

    fn tay(&mut self) {
        self.y = self.a;
        self.update_zn(self.y);
    }

    fn tsx(&mut self) {
        self.x = self.s;
        self.update_zn(self.x);
    }

    fn txa(&mut self) {
        self.a = self.x;
        self.update_zn(self.a);
    }

    fn txs(&mut self) {
        self.s = self.x;
    }

    fn tya(&mut self) {
        self.a = self.y;
        self.update_zn(self.a);
    }

    fn dcp(&mut self) {
        self.dec();
        self.cmp();
    }

    fn isb(&mut self) {
        self.inc();
        self.sbc();
    }

    fn lax(&mut self) {
        self.lda();
        self.ldx();
    }

    fn rla(&mut self) {
        self.rol();
        self.and();
    }

    fn rra(&mut self) {
        self.ror();
        self.adc();
    }

    fn sax(&mut self) {
        self.write8(self.addr, self.a & self.x);
    }

    fn slo(&mut self) {
        self.asl();
        self.ora();
    }

    fn sre(&mut self) {
        self.lsr();
        self.eor();
    }
}

#[cfg(test)]

mod test {
    use super::*;

    #[test]
    fn test_adc() {
        let mut cpu = CPU::new();
        cpu.run(|cpu| {
            cpu.a = 0xab;
            cpu.p.c = true;
            cpu.write8(0x0000, 0x65);
            cpu.write8(0x0001, 0x12);
            cpu.write8(0x0012, 0x78);
        });
        assert_eq!(cpu.a, 0x24);
        assert_eq!(cpu.p.get(), 0x25);
    }

    #[test]
    fn test_and() {
        let mut cpu = CPU::new();
        cpu.run(|cpu| {
            cpu.a = 0xa5;
            cpu.x = 0xa2;
            cpu.write8(0x0000, 0x35);
            cpu.write8(0x0001, 0xab);
            cpu.write8(0x4d, 0x91);
        });
        assert_eq!(cpu.a, 0x81);
        assert_eq!(cpu.p.get(), 0xa4);
    }

    #[test]
    fn test_asl_acc() {
        let mut cpu = CPU::new();
        cpu.run(|cpu| {
            cpu.a = 0xaa;
            cpu.write8(0x0000, 0x0a);
        });
        assert_eq!(cpu.a, 0x54);
        assert_eq!(cpu.p.get(), 0x25);
    }

    #[test]
    fn test_asl() {
        let mut cpu = CPU::new();
        cpu.run(|cpu| {
            cpu.write8(0x0000, 0x0e);
            cpu.write16(0x0001, 0x0200);
            cpu.write8(0x0200, 0x38);
        });
        assert_eq!(cpu.read8(0x0200), 0x70);
        assert_eq!(cpu.p.get(), 0x24);
    }

    #[test]
    fn test_bcc() {
        let mut cpu = CPU::new();
        cpu.run(|cpu| {
            cpu.a = 0x84;
            cpu.write8(0x0000, 0x90);
            cpu.write8(0x0001, 0x25);
            cpu.write8(0x0027, 0x69);
            cpu.write8(0x0028, 0x85);
        });
        assert_eq!(cpu.a, 0x09);
        assert_eq!(cpu.p.get(), 0x65);
    }

    #[test]
    fn test_bcs() {
        let mut cpu = CPU::new();
        cpu.run(|cpu| {
            cpu.a = 0xa5;
            cpu.p.c = true;
            cpu.pc = 0x0200;
            cpu.write8(0x0200, 0xb0);
            cpu.write8(0x0201, 0x82);
            cpu.write8(0x0184, 0x29);
            cpu.write8(0x0185, 0x84);
        });
        assert_eq!(cpu.a, 0x84);
        assert_eq!(cpu.p.get(), 0xa5);
    }

    #[test]
    fn test_beq() {
        let mut cpu = CPU::new();
        cpu.run(|cpu| {
            cpu.x = 0x77;
            cpu.p.z = true;
            cpu.write8(0x0000, 0xf0);
            cpu.write8(0x0001, 0x65);
            cpu.write8(0x0067, 0x1e);
            cpu.write16(0x0068, 0x0456);
            cpu.write8(0x04cd, 0x56);
        });
        assert_eq!(cpu.read8(0x04cd), 0xac);
        assert_eq!(cpu.p.get(), 0xa4);
    }

    #[test]
    fn test_bit() {
        let mut cpu = CPU::new();
        cpu.run(|cpu| {
            cpu.a = 0x5a;
            cpu.write8(0x0000, 0x24);
            cpu.write8(0x0001, 0x35);
            cpu.write8(0x0035, 0xe5);
        });
        assert_eq!(cpu.a, 0x5a);
        assert_eq!(cpu.read8(0x0035), 0xe5);
        assert_eq!(cpu.p.get(), 0xe4);
    }
}
