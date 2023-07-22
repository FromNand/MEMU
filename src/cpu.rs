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
        let low = self.bus.read8(addr) as u16;
        let high = self.bus.read8(addr.wrapping_add(1)) as u16;
        low + (high << 8)
    }

    fn write8(&mut self, addr: u16, data: u8) {
        self.bus.write8(addr, data);
    }

    fn write16(&mut self, addr: u16, data: u16) {
        self.bus.write8(addr, data as u8);
        self.bus.write8(addr.wrapping_add(1), (data >> 8) as u8);
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
            cpu.write8(0x0000, 0x69);
            cpu.write8(0x0001, 0x78);
        });
        assert_eq!(cpu.a, 0x24);
        assert_eq!(cpu.p.get(), 0x25);
    }
}
