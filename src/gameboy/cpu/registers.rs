use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[derive(PartialEq, EnumIter)]
pub enum Reg8 {
    B = 0, 
    C = 1, 
    D = 2,
    E = 3, 
    H = 4, 
    L = 5,  
    A = 7,
}

impl Reg8 {
    pub fn from(reg: u8) -> Reg8 {
        Reg8::iter()
            .get(reg as usize)
            .expect("Unknown Reg8 Register!")
    }
}

#[derive(PartialEq, EnumIter)]
pub enum Reg16 {
    BC = 0, 
    DE = 1, 
    HL = 2,
    AF = 3,
}

impl Reg16 {
    pub fn from(reg: u8) -> Reg16 {
        Reg16::iter()
            .get(reg as usize)
            .expect("Unknown Reg16 Register!")
    }
}

#[derive(Default)]
pub struct Registers {
    pub a: u8,
    pub flag_register: FlagsRegister,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,
}

impl Registers {
    pub fn new() -> Self {
        Registers {
            a: 0x01,
            flag_register: FlagsRegister { flags: 0xB0 },
            b: 0x00,
            c: 0x13,
            d: 0x00,
            e: 0xD8,
            h: 0x01,
            l: 0x4D,
        }
    }

    pub fn read8(&self, reg: &Reg8) -> u8 {
        match reg {
            Reg8::A  => self.a,
            Reg8::B  => self.b,
            Reg8::C  => self.c,
            Reg8::D  => self.d,
            Reg8::E  => self.e,
            Reg8::H  => self.h,
            Reg8::L  => self.l,
        }
    }

    pub fn write8(&mut self, reg: &Reg8, value: u8) {
        match reg {
            Reg8::A  => self.a = value,
            Reg8::B  => self.b = value,
            Reg8::C  => self.c = value,
            Reg8::D  => self.d = value,
            Reg8::E  => self.e = value,
            Reg8::H  => self.h = value,
            Reg8::L  => self.l = value,
        }
    }

    pub fn read16(&self, reg: &Reg16) -> u16 {
        let (high_byte, low_byte) = match reg {
            Reg16::BC => (self.b, self.c),
            Reg16::DE => (self.d, self.e),
            Reg16::HL => (self.h, self.l),
            Reg16::AF => (self.a, self.flag_register.flags)
        };

        ((high_byte as u16) << 8) | low_byte as u16
    }

    pub fn write16(&mut self, reg: &Reg16, value: u16) {
        let (high_reg, low_reg) = match reg {
            Reg16::BC => (&mut self.b, &mut self.c),
            Reg16::DE => (&mut self.d, &mut self.e),
            Reg16::HL => (&mut self.h, &mut self.l),
            Reg16::AF => (&mut self.a, &mut self.flag_register.flags)
        };

        *high_reg = (value >> 8) as u8;
        *low_reg = (value & 0xF0) as u8;
    }
}

#[allow(non_camel_case_types)]
pub enum Flags {
    ZERO = 0x80,
    SUBSTRACTION = 0x40,
    HALF_CARRY = 0x20,
    CARRY = 0x10
}

#[derive(Default)]
pub struct FlagsRegister {
    pub flags: u8
}

impl FlagsRegister {
    pub fn set_flag(&mut self, flag: Flags, value: bool) {
        let flag = flag as u8; 
        if value { self.flags |= flag } else { self.flags &= !flag }
    }

    pub fn get_flag(&mut self, flag: Flags) -> u8 {
        if self.flags & flag as u8 != 0 { 1 } else { 0 }
    }
}