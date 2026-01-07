use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[derive(PartialEq, EnumIter)]
pub enum Reg8 {
    A = 7,
    B = 0, 
    C = 1, 
    D = 2,
    E = 3, 
    H = 4, 
    L = 5,  
}

pub enum Reg16 {
    BC, 
    DE, 
    HL = 6
}

impl Reg8 {
    pub fn from(reg: u8) -> Reg8 {
        Reg8::iter().get(reg as usize)
            .expect("Unknown Register!")
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

    pub fn hl(&self) -> u16 {
        ((self.h as u16) << 8) | self.l as u16
    }

    pub fn set_hl(&mut self, value: u16) {
        self.h = (value >> 8) as u8;
        self.l = (value & 0xF0) as u8;
    }

    pub fn af(&self) -> u16 {
        ((self.a as u16) << 8) | self.flag_register.flags as u16
    }

    pub fn set_af(&mut self, value: u16) {
        self.a = (value >> 8) as u8;
        self.flag_register.flags = (value & 0xF0) as u8;
    }

    pub fn bc(&self) -> u16 {
        ((self.b as u16) << 8) | self.c as u16
    }

    pub fn set_bc(&mut self, value: u16) {
        self.b = (value >> 8) as u8;
        self.c = (value & 0xF0) as u8;
    }

    pub fn de(&self) -> u16 {
        ((self.d as u16) << 8) | self.e as u16
    }

    pub fn set_de(&mut self, value: u16) {
        self.d = (value >> 8) as u8;
        self.e = (value & 0xF0) as u8;
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