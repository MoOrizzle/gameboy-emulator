mod registers;

use super::mmu::Mmu;
use registers::{Flags, Reg8, Reg16, Registers};

enum Operand8 {
    Register(Reg8),
    IndirectHL,
}

const INTERRUPTS: [(u8, u16); 5] = [
    (0, 0x40),
    (1, 0x48),
    (2, 0x50),
    (3, 0x58),
    (4, 0x60),
];

pub struct Cpu {
    pub program_counter: u16,
    pub stack_pointer: u16,

    pub registers: Registers,
    
    pub pending_ime: bool,
    pub ime: bool, // Interrupt Master Enable
    pub halted: bool,
}

impl Cpu {
    pub fn new() -> Self {
        Self {
            stack_pointer: 0xFFFE,
            program_counter: 0x0100,
            registers: Registers::new(),
            pending_ime: false,
            ime: false,
            halted: false,
        }
    }

    /// Steps 
    /// 
    /// * `result` - Returns the cycles the cpu needs to execute the current opcode
    pub fn step(&mut self, mmu: &mut Mmu) -> u8 {
        
        if self.handle_interrupts(mmu) {
            if self.pending_ime {
                self.pending_ime = false;
                self.ime = true;
            }
            return 5;
        }

        if self.halted {
            let ie = mmu.read8(0xFFFF);
            let iflag = mmu.read8(0xFF0F);
            if ie & iflag != 0 {
                self.halted = false;
            }
            return 4;
        }

        let opcode = self.fetch_byte(mmu);
        let cylces = match opcode {
            // NOP aka No OPeration
            0x00 => 4,

            //RLCA 
            0x07 => {
                let dst_register = Reg8::A;
                let dst_value = self.registers.read8(&dst_register);

                let pushed_out = (dst_value & 0x80) >> 7;
                let result = dst_value.rotate_left(1);

                self.registers.write8(&dst_register, result);

                let flags = &mut self.registers.flag_register;
                flags.set_flag(Flags::ZERO, false);
                flags.set_flag(Flags::SUBSTRACTION, false);
                flags.set_flag(Flags::HALF_CARRY, false);
                flags.set_flag(Flags::CARRY, pushed_out == 1);

                4
            },

            //RRCA 
            0x0F => {
                let dst_register = Reg8::A;
                let dst_value = self.registers.read8(&dst_register);

                let pushed_out = dst_value & 1;
                let result = dst_value.rotate_right(1);

                self.registers.write8(&dst_register, result);

                let flags = &mut self.registers.flag_register;
                flags.set_flag(Flags::ZERO, false);
                flags.set_flag(Flags::SUBSTRACTION, false);
                flags.set_flag(Flags::HALF_CARRY, false);
                flags.set_flag(Flags::CARRY, pushed_out == 1);

                4
            },

            //RLA
            0x17 => {
                let dst_register = Reg8::A;
                let dst_value = self.registers.read8(&dst_register);

                let carry_flag = self.registers.flag_register.get_flag(Flags::CARRY) as u8;
                let pushed_out = (dst_value & 0x80) >> 7;
                let result = (dst_value << 1) | carry_flag;

                self.registers.write8(&dst_register, result);
                
                let flags = &mut self.registers.flag_register;
                flags.set_flag(Flags::ZERO, false);
                flags.set_flag(Flags::SUBSTRACTION, false);
                flags.set_flag(Flags::HALF_CARRY, false);
                flags.set_flag(Flags::CARRY, pushed_out == 1);

                4
            },

            //RRA
            0x1F => {
                let dst_register = Reg8::A;
                let dst_value = self.registers.read8(&dst_register);

                let carry_flag = self.registers.flag_register.get_flag(Flags::CARRY) as u8;
                let pushed_out = dst_value & 1;
                let result = (dst_value >> 1) | (carry_flag << 7);

                self.registers.write8(&dst_register, result);
                
                let flags = &mut self.registers.flag_register;
                flags.set_flag(Flags::ZERO, false);
                flags.set_flag(Flags::SUBSTRACTION, false);
                flags.set_flag(Flags::HALF_CARRY, false);
                flags.set_flag(Flags::CARRY, pushed_out == 1);

                4
            },

            //JR
            0x18 | 0x20 | 0x28 | 0x30 | 0x38 => {
                let flags = &self.registers.flag_register;
                let condition = match opcode {
                    //JR e8
                    0x18 => true,
                    //JR NZ e8
                    0x20 => flags.get_flag(Flags::ZERO),
                    //JR Z e8
                    0x28 => flags.get_flag(Flags::ZERO),
                    //JR NC e8
                    0x30 => flags.get_flag(Flags::CARRY),
                    //JR C e8
                    0x38 => flags.get_flag(Flags::CARRY),
                    
                    _ => unreachable!()
                };

                if !condition {
                    return 8;
                }
                
                let jmp_offset = self.fetch_byte(mmu) as i8;
                self.program_counter = ((self.program_counter as i16) + (jmp_offset as i16)) as u16;

                12
            },
            
            //INC r8 | HL
            0x04 | 0x0C | 0x14 | 0x1C | 0x24 | 0x2C | 0x34 | 0x3C |
            //DEC r8 | HL 
            0x05 | 0x0D | 0x15 | 0x1D | 0x25 | 0x2D | 0x35 | 0x3D => {
                let destination_num = (opcode >> 3) & 0x07;

                let destination = match destination_num {
                    6 => Operand8::IndirectHL,
                    _ => Operand8::Register(Reg8::from(destination_num))
                };

                let val = match destination {
                    Operand8::Register(ref reg) => self.registers.read8(reg),
                    Operand8::IndirectHL => mmu.read8(self.registers.read16(&Reg16::HL)),
                };

                let is_inc = (opcode & 0x01) == 0;
                let result = match is_inc {
                    true  => val.wrapping_add(1),
                    false => val.wrapping_sub(1)
                };

                match destination {
                    Operand8::Register(ref reg) => self.registers.write8(reg, result),
                    Operand8::IndirectHL => mmu.write8(self.registers.read16(&Reg16::HL), result),
                }

                let flags = &mut self.registers.flag_register;
                flags.set_flag(Flags::ZERO, result == 0);
                flags.set_flag(Flags::SUBSTRACTION, false);

                let half_carry_borrow = if is_inc { 0x0F } else { 0x00 };
                flags.set_flag(Flags::HALF_CARRY, (val & 0x0F) == half_carry_borrow);
                
                if destination_num == 6 { 12 } else { 4 }
            },

            //INC r16 | SP
            0x03 | 0x13 | 0x23 |
            //DEC r16 | SP
            0x0B | 0x1B | 0x2B => {
                let destination_num = (opcode >> 4) & 0x07;
                let reg16 = Reg16::from(destination_num);
                
                let val = self.registers.read16(&reg16);

                let is_inc = ((opcode >> 3) & 0x01) == 0;
                let result = match is_inc {
                    true  => val.wrapping_add(1),
                    false => val.wrapping_sub(1)
                };

                self.registers.write16(&reg16, result);
                
                8
            },

            //INC SP
            0x33 => {
                self.stack_pointer = self.stack_pointer.wrapping_add(1);
                8
            },

            //DEC SP
            0x3B => {
                self.stack_pointer = self.stack_pointer.wrapping_sub(1);
                8
            },

            //LD r n8
            0x06 | 0x0E | 0x16 | 0x1E | 0x26 | 0x2E | 0x3E => {
                let reg = Reg8::from((opcode >> 3) & 0x07);
                let val = self.fetch_byte(mmu);

                self.registers.write8(&reg, val);
                
                8
            },

            //LD HL n8
            0x36 => {
                let val = self.fetch_byte(mmu);
                 mmu.write8(self.registers.read16(&Reg16::HL), val);
                
                12
            }

            //HALT
            0x76 => {
                self.halted = true;
                
                4
            },

            //LD r8 HL
            0x46 | 0x4E | 0x56 | 0x5E | 0x66 | 0x6E | 0x7E => {
                let src = Reg8::from(opcode & 0x07);

                let val = self.registers.read8(&src);
                mmu.write8(self.registers.read16(&Reg16::HL), val);

                8
            },

            //LD HL r8
            0x70 | 0x71 | 0x72 | 0x73 | 0x74 | 0x75 | 0x77 => {
                let dst = Reg8::from((opcode >> 3) & 0x07);

                let val = mmu.read8(self.registers.read16(&Reg16::HL));
                self.registers.write8(&dst, val);

                8
            },

            //LD r8 r8 -> All special HL "register" should already be handled
            0x40..=0x7F => {
                let dst = Reg8::from((opcode >> 3) & 0x07);
                let src = Reg8::from(opcode & 0x07);

                let val = self.registers.read8(&src);
                self.registers.write8(&dst, val);

                4
            },

            //LD r16 n16
            0x01 | 0x11 | 0x21 => {
                let reg16 = Reg16::from((opcode >> 4) & 0x03);

                let word = self.fetch_word(mmu);
                self.registers.write16(&reg16, word);

                12
            },

            //LD SP n16
            0x31 => {
                let word = self.fetch_word(mmu);
                self.stack_pointer = word;

                12
            },

            //LD [r16] A
            0x02 | 0x12 => {
                let val = self.registers.read8(&Reg8::A);

                let reg16 = Reg16::from((opcode >> 4) & 0x03);
                let addr = self.registers.read16(&reg16);

                mmu.write8(addr, val);

                8
            },

            //LD [HL+] | [HL-] A
            0x22 | 0x32 => {
                let val = self.registers.read8(&Reg8::A);

                let addr = self.registers.read16(&Reg16::HL);

                mmu.write8(addr, val);

                let write_back = if opcode == 0x22 { addr.wrapping_add(1) } else { addr.wrapping_sub(1) };
                self.registers.write16(&Reg16::HL, write_back);

                8
            },

            //LD A [r16]
            0x0A | 0x1A => {
                let reg16 = Reg16::from((opcode >> 4) & 0x03);
                let addr = self.registers.read16(&reg16);

                let val = mmu.read8(addr);

                self.registers.write8(&Reg8::A, val);

                8
            },

            //LD A [HL+] | [HL-]
            0x2A | 0x3A => {
                let addr = self.registers.read16(&Reg16::HL);

                let val = mmu.read8(addr);

                self.registers.write8(&Reg8::A, val);

                let write_back = if opcode == 0x2A { addr.wrapping_add(1) } else { addr.wrapping_sub(1) };
                self.registers.write16(&Reg16::HL, write_back);

                8
            },

            //LDH [a8] A
            0xE0 => {
                let a8 = self.fetch_byte(mmu);
                let addr = 0xFF00 | (a8 as u16);

                let val = self.registers.read8(&Reg8::A);

                mmu.write8(addr, val);

                12
            },

            //LDH A [a8]
            0xF0 => {
                let a8 = self.fetch_byte(mmu);
                let addr = 0xFF00 | (a8 as u16);

                let val = mmu.read8(addr);

                self.registers.write8(&Reg8::A, val);

                12
            },

            //LDH [C] A
            0xE2 => {
                let reg_c_val = self.registers.read8(&Reg8::C);
                let addr = 0xFF00 | (reg_c_val as u16);

                let val = self.registers.read8(&Reg8::A);

                mmu.write8(addr, val);

                8
            },

            //LDH A [C]
            0xF2 => {
                let reg_c_val = self.registers.read8(&Reg8::C);
                let addr = 0xFF00 | (reg_c_val as u16);

                let val = mmu.read8(addr);

                self.registers.write8(&Reg8::A, val);
                
                8
            },

            //LD [a16] A
            0xEA => {
                let addr = self.fetch_word(mmu);

                let val = self.registers.read8(&Reg8::A);

                mmu.write8(addr, val);
                
                16
            },

            //LD A [a16]
            0xFA => {
                let addr = self.fetch_word(mmu);

                let val = mmu.read8(addr);

                self.registers.write8(&Reg8::A, val);

                16
            },

            //*ALU OPERATIONS*

            //ADD A r8 | [HL] | n8
            0x80..=0x87 | 0xC6 => {
                let dst_register = Reg8::A;
                let dst_value = self.registers.read8(&dst_register);
                
                let reg_num = opcode & 0x07;

                let to_add_value = match opcode {
                    0x86 => mmu.read8(self.registers.read16(&Reg16::HL)),
                    0xC6 => self.fetch_byte(mmu),
                    _ => self.registers.read8(&Reg8::from(reg_num))
                };
                
                let result = dst_value.wrapping_add(to_add_value);
                self.registers.write8(&dst_register, result);

                let flags = &mut self.registers.flag_register;
                flags.set_flag(Flags::ZERO, result == 0);
                flags.set_flag(Flags::SUBSTRACTION, false);
                flags.set_flag(Flags::HALF_CARRY, (dst_value & 0x0F) == 0x00);
                flags.set_flag(Flags::CARRY, (dst_value as u16 + to_add_value as u16) > 0xFF);

                if reg_num == 6 { 8 } else { 4 }
            },

            //ADC A r8 | [HL] | n8
            0x88..=0x8F | 0xCE => {
                let dst_register = Reg8::A;
                let dst_value = self.registers.read8(&dst_register);
                
                let reg_num = opcode & 0x07;

                let to_add_value = match opcode {
                    0x8E => mmu.read8(self.registers.read16(&Reg16::HL)),
                    0xCE => self.fetch_byte(mmu),
                    _ => self.registers.read8(&Reg8::from(reg_num))
                };

                let carry_in = self.registers.flag_register.get_flag(Flags::CARRY) as u8;
                let result = dst_value
                    .wrapping_add(to_add_value)
                    .wrapping_add(carry_in);
                
                self.registers.write8(&dst_register, result);

                let flags = &mut self.registers.flag_register;
                flags.set_flag(Flags::ZERO, result == 0);
                flags.set_flag(Flags::SUBSTRACTION, false);
                flags.set_flag(Flags::HALF_CARRY, ((dst_value & 0x0F) + (to_add_value & 0x0F) + carry_in) > 0x0F);
                flags.set_flag(Flags::CARRY, (dst_value as u16 + to_add_value as u16 + carry_in as u16) > 0xFF);

                if reg_num == 6 { 8 } else { 4 }
            },

            //SBC A r8 | [HL] | n8
            0x98..=0x9F | 0xDE => {
                let dst_register = Reg8::A;
                let dst_value = self.registers.read8(&dst_register);
                
                let reg_num = opcode & 0x07;

                let to_sub_value = match opcode {
                    0x9E => mmu.read8(self.registers.read16(&Reg16::HL)),
                    0xDE => self.fetch_byte(mmu),
                    _ => self.registers.read8(&Reg8::from(reg_num))
                };

                let carry_in = self.registers.flag_register.get_flag(Flags::CARRY) as u8;
                let result = dst_value
                    .wrapping_sub(to_sub_value)
                    .wrapping_sub(carry_in);
                
                self.registers.write8(&dst_register, result);

                let flags = &mut self.registers.flag_register;
                flags.set_flag(Flags::ZERO, result == 0);
                flags.set_flag(Flags::SUBSTRACTION, true);
                flags.set_flag(Flags::HALF_CARRY, dst_value & 0x0F < (to_sub_value & 0x0F + carry_in));
                flags.set_flag(Flags::CARRY, dst_value < to_sub_value + carry_in);

                if reg_num == 6 { 8 } else { 4 }
            },

            //SUB A r8 | [HL] | n8
            0x90..=0x97 | 0xD6 | 
            //ComPare A r8 | [HL] | n8
            0xB8..=0xBF | 0xFE => {
                let dst_register = Reg8::A;
                let dst_value = self.registers.read8(&dst_register);

                let reg_num = opcode & 0x07;

                let to_sub_value = match opcode {
                    0x96 | 0xBE => mmu.read8(self.registers.read16(&Reg16::HL)),
                    0xD6 | 0xFE => self.fetch_byte(mmu),
                    _ => self.registers.read8(&Reg8::from(reg_num))
                };

                let result = dst_value.wrapping_sub(to_sub_value);
                //if SUB
                if (opcode >> 3 & 0x01) == 0 {
                    self.registers.write8(&dst_register, result);
                }

                let flags = &mut self.registers.flag_register;
                flags.set_flag(Flags::ZERO, result == 0);
                flags.set_flag(Flags::SUBSTRACTION, true);
                flags.set_flag(Flags::HALF_CARRY, (dst_value & 0x0F) < (to_sub_value & 0x0F));
                flags.set_flag(Flags::CARRY, to_sub_value > dst_value);

                if reg_num == 6 { 8 } else { 4 }
            },

            // AND A r8 | [HL] | n8
            0xA0..=0xA7 | 0xE6 => {
                let dst_register = Reg8::A;
                let dst_value = self.registers.read8(&dst_register);

                let reg_num = opcode & 0x07;

                let to_and_value = match opcode {
                    0xA6 => mmu.read8(self.registers.read16(&Reg16::HL)),
                    0xE6 => self.fetch_byte(mmu),
                    _ => self.registers.read8(&Reg8::from(reg_num))
                };

                let result = dst_value & to_and_value;
                self.registers.write8(&dst_register, result);

                let flags = &mut self.registers.flag_register;
                flags.set_flag(Flags::ZERO, result == 0);
                flags.set_flag(Flags::SUBSTRACTION, false);
                flags.set_flag(Flags::HALF_CARRY, true);
                flags.set_flag(Flags::CARRY, false);

                if reg_num == 6 { 8 } else { 4 }
            },

            //XOR A r8 | [HL] | n8
            0xA8..=0xAF | 0xEE => {
                let dst_register = Reg8::A;
                let dst_value = self.registers.read8(&dst_register);

                let reg_num = opcode & 0x07;

                let to_xor_value = match opcode {
                    0xAE => mmu.read8(self.registers.read16(&Reg16::HL)),
                    0xEE => self.fetch_byte(mmu),
                    _ => self.registers.read8(&Reg8::from(reg_num))
                };

                let result = dst_value ^ to_xor_value;
                self.registers.write8(&dst_register, result);

                let flags = &mut self.registers.flag_register;
                flags.set_flag(Flags::ZERO, result == 0);
                flags.set_flag(Flags::SUBSTRACTION, false);
                flags.set_flag(Flags::HALF_CARRY, false);
                flags.set_flag(Flags::CARRY, false);

                if reg_num == 6 { 8 } else { 4 }
            },

            //OR A r8 | [HL] | n8
            0xB0..=0xB7 | 0xF6 => {
                let dst_register = Reg8::A;
                let dst_value = self.registers.read8(&dst_register);

                let reg_num = opcode & 0x07;

                let to_or_value = match opcode {
                    0xB6 => mmu.read8(self.registers.read16(&Reg16::HL)),
                    0xF6 => self.fetch_byte(mmu),
                    _ => self.registers.read8(&Reg8::from(reg_num))
                };

                let result = dst_value | to_or_value;
                self.registers.write8(&dst_register, result);
                
                let flags = &mut self.registers.flag_register;
                flags.set_flag(Flags::ZERO, result == 0);
                flags.set_flag(Flags::SUBSTRACTION, false);
                flags.set_flag(Flags::HALF_CARRY, false);
                flags.set_flag(Flags::CARRY, false);

                if reg_num == 6 { 8 } else { 4 }
            },

            //PREFIX
            0xCB => {
                let prefixed_opcode = self.fetch_byte(mmu);
                let cycles = self.handle_prefixed(prefixed_opcode, mmu);

                cycles
            },
            
            //JP
            0xC2 | 0xC3 | 0xCA | 0xD2 | 0xDA => {
                let flags = &self.registers.flag_register;
                let condition = match opcode {
                    //JP a16
                    0xC3 => true,
                    //JP NZ a16
                    0xC2 => flags.get_flag(Flags::ZERO),
                    //JP Z a16
                    0xCA => flags.get_flag(Flags::ZERO),
                    //JP NC a16
                    0xD2 => flags.get_flag(Flags::CARRY),
                    //JP C a16
                    0xDA => flags.get_flag(Flags::CARRY),

                    _ => unreachable!()
                };

                if !condition {
                    return 12;
                }
                
                self.program_counter = self.fetch_word(mmu);

                16
            },

            //RET
            0xC0 | 0xC8 | 0xC9 | 0xD0 | 0xD8 | 0xD9 => {
                let flags = &self.registers.flag_register;
                let condition = match opcode {
                    //RET | RETI
                    0xC9 | 0xD9 => true,
                    //RET NZ
                    0xC0 => flags.get_flag(Flags::ZERO),
                    //RET Z
                    0xC8 => flags.get_flag(Flags::ZERO),
                    //RET NC
                    0xD0 => flags.get_flag(Flags::CARRY),
                    //RET C
                    0xD8 => flags.get_flag(Flags::CARRY),

                    _ => unreachable!()
                };
                
                if !condition {
                    return 8;
                }

                let lower_byte = mmu.read8(self.stack_pointer) as u16;
                self.stack_pointer += 1;

                let higher_byte = mmu.read8(self.stack_pointer) as u16;
                self.stack_pointer += 1;

                let jmp_addr = (higher_byte << 8) | lower_byte;
                self.program_counter = jmp_addr;

                //RETI
                if opcode == 0xD9 {
                    self.ime = true;
                }

                if opcode == 0xC9 || opcode == 0xD9 { 16 } else { 20 }
            },

            //CALL
            0xC4 | 0xCC | 0xCD | 0xD4 | 0xDC => {
                let flags = &self.registers.flag_register;
                let condition = match opcode {
                    //CALL a16
                    0xCD => true,
                    //CALL NZ a16
                    0xC4 => flags.get_flag(Flags::ZERO),
                    //CALL Z a16
                    0xCC => flags.get_flag(Flags::ZERO),
                    //CALL NC a16
                    0xD4 => flags.get_flag(Flags::CARRY),
                    //CALL C a16
                    0xDC => flags.get_flag(Flags::CARRY),

                    _ => unreachable!()
                };
                
                if !condition {
                    return 12;
                }

                let jmp_addr = self.fetch_word(mmu);

                let pc_high = (self.program_counter >> 8) as u8;
                let pc_low = (self.program_counter as u8) & 0xFF;

                self.stack_pointer -= 1;
                mmu.write8(self.stack_pointer, pc_high);

                self.stack_pointer -= 1;
                mmu.write8(self.stack_pointer, pc_low);
                
                self.program_counter = jmp_addr;

                24
            },

            //POP
            0xC1 | 0xD1 | 0xE1 | 0xF1 => {
                let reg16 = Reg16::from((opcode >> 4) & 0x03);
                let mut val = mmu.read16(self.stack_pointer);

                self.stack_pointer += 2;

                if reg16 == Reg16::AF {
                    val &= 0xF0;
                }
                
                self.registers.write16(&reg16, val);
            
                12
            },
            
            //PUSH
            0xC5 | 0xD5 | 0xE5 | 0xF5 => {
                let reg16 = Reg16::from((opcode >> 4) & 0x03);
                let val = self.registers.read16(&reg16);
                
                self.stack_pointer -= 2;
                mmu.write16(self.stack_pointer, val);
            
                16
            },

            //RST vec
            0xC7 | 0xD7 | 0xE7 | 0xF7 | 0xCF | 0xDF | 0xEF | 0xFF => {
                let vec = opcode & 0x38;

                let pc_high = (self.program_counter >> 8) as u8;
                let pc_low = (self.program_counter as u8) & 0xFF;

                self.stack_pointer -= 1;
                mmu.write8(self.stack_pointer, pc_high);

                self.stack_pointer -= 1;
                mmu.write8(self.stack_pointer, pc_low);

                self.program_counter = vec as u16;

                16
            },

            //DI
            0xF3 => {
                self.ime = false;
                4
            },

            //DE
            0xFB => {
                self.pending_ime = true;
                4
            },

            _ => panic!("Unimplemented opcode {:02X}", opcode)
        };

        if self.pending_ime {
            self.pending_ime = false;
            self.ime = true;
        }

        cylces
    }

    fn handle_interrupts(&mut self, mmu: &mut Mmu) -> bool {
        if !self.ime {
            return false;
        }

        let ie = mmu.read8(0xFFFF);
        let mut iflag = mmu.read8(0xFF0F);

        let pending = ie & iflag;
        if pending == 0 {
            return false;
        }
            
            self.ime = false;

        let pc_high = (self.program_counter >> 8) as u8;
        let pc_low = self.program_counter as u8;

        self.stack_pointer -= 1;
        mmu.write8(self.stack_pointer, pc_high);
        self.stack_pointer -= 1;
        mmu.write8(self.stack_pointer, pc_low);

        let (vector, bit) = match INTERRUPTS
            .iter()
            .find(|(bit, _)| pending & (1 << bit) != 0)
        {
            Some((bit, vector)) => (*vector, *bit),
            None => return false,
        };

        iflag &= !(1 << bit);
        mmu.write8(0xFF0F, iflag);

        self.program_counter = vector;

        true
    }

    /// fetches next byte from MMU
    /// 
    /// increments program counter by 1
    fn fetch_byte(&mut self, mmu: &Mmu) -> u8 {
        let val = mmu.read8(self.program_counter);
        self.program_counter += 1;

        val
    }

    /// fetches next word (2 bytes) from MMU
    /// 
    /// increments program counter by 2
    fn fetch_word(&mut self, mmu: &Mmu) -> u16 {
        let low_byte = self.fetch_byte(mmu) as u16;
        let high_byte = self.fetch_byte(mmu) as u16;

        (high_byte << 8) + low_byte
    }

    fn set_rotate_register_flags(&mut self, result: u8, pushed_out: u8) {
        let flags = &mut self.registers.flag_register;
        flags.set_flag(Flags::ZERO, result == 0);
        flags.set_flag(Flags::SUBSTRACTION, false);
        flags.set_flag(Flags::HALF_CARRY, false);
        flags.set_flag(Flags::CARRY, pushed_out == 1); 
    }

    fn handle_prefixed(&mut self, prefixed_opcode: u8, mmu: &mut Mmu) -> u8 {

        let destination_num = prefixed_opcode & 0x07;
        let destination = match destination_num {
            6 => Operand8::IndirectHL,
            _ => Operand8::Register(Reg8::from(destination_num))
        };

        let dst_value = match destination {
            Operand8::IndirectHL => mmu.read8(self.registers.read16(&Reg16::HL)),
            Operand8::Register(ref reg) => self.registers.read8(reg),
        };

        let result: u8 = match prefixed_opcode {
            //RLC r8 | [HL]
            0x00..=0x07 => {
                let pushed_out = (dst_value & 0x80) >> 7;
                let result = dst_value.rotate_left(1);
                self.set_rotate_register_flags(result, pushed_out);

                result
            },

            //RRC r8 | [HL]
            0x08..=0x0F =>  {
                let pushed_out = dst_value & 1;
                let result = dst_value.rotate_right(1);
                self.set_rotate_register_flags(result, pushed_out);

                result
            },

            //RL r8 | [HL]
            0x10..=0x17 => {
                let pushed_out = (dst_value & 0x80) >> 7;
                let carry_flag = self.registers.flag_register.get_flag(Flags::CARRY) as u8;
                let result = (dst_value << 1) | carry_flag;
                self.set_rotate_register_flags(result, pushed_out);

                result
            },

            //RR r8 | [HL]
            0x18..=0x1F => {
                let pushed_out = dst_value & 1;
                let carry_flag = self.registers.flag_register.get_flag(Flags::CARRY) as u8;
                let result = (dst_value >> 1) | (carry_flag << 7);
                self.set_rotate_register_flags(result, pushed_out);

                result
            },

            //SLA r8 | [HL]
            0x20..=0x27 => {
                let pushed_out = (dst_value & 0x80) >> 7;
                let result = dst_value << 1;
                self.set_rotate_register_flags(result, pushed_out);

                result
            },

            //SRA r8 | [HL]
            0x28..=0x2F => {
                let pushed_out = dst_value & 1;
                let highest_bit = dst_value & 0x80;
                let result = (dst_value >> 1) | highest_bit;
                self.set_rotate_register_flags(result, pushed_out);

                result
            },

            //SWAP r8 | [HL] //TODO
            0x30..=0x37 => {
                let result = ((dst_value & 0x0F) << 4) | ((dst_value & 0xF0) >> 4);

                let flags = &mut self.registers.flag_register;
                flags.set_flag(Flags::ZERO, result == 0);
                flags.set_flag(Flags::SUBSTRACTION, false);
                flags.set_flag(Flags::HALF_CARRY, false);
                flags.set_flag(Flags::CARRY, false);

                result
            },

            //SRL r8 | [HL]
            0x38..=0x3F => {
                let pushed_out = dst_value & 1;
                let result = dst_value >> 1;
                self.set_rotate_register_flags(result, pushed_out);

                result
            },

            //BIT u3 r8 | [HL]
            0x40..=0x7F => {            
                let test_bit = (prefixed_opcode >> 3) & 0x07;
                let result = (dst_value >> test_bit) & 0x01;

                let flags = &mut self.registers.flag_register;
                flags.set_flag(Flags::ZERO, result == 0);
                flags.set_flag(Flags::SUBSTRACTION, false);
                flags.set_flag(Flags::HALF_CARRY, true);

                result
            },

            //RES | SET u3 r8 | [HL]
            0x80..=0xFF => {
                let test_bit = (prefixed_opcode >> 3) & 0x07;
                let bit_mask: u8 = 1 << test_bit;

                //if bit 6 is 0: RES else SET
                let result = match (prefixed_opcode >> 0x06) & 1 {
                    0 => dst_value & !bit_mask,
                    _ => dst_value | bit_mask
                };

                result
            }
        };

        let bit_op_range = 0x40..=0x7F;
        if bit_op_range.contains(&prefixed_opcode) {
            return if destination_num == 6 { 12 } else { 8 }
        }
        
        match destination {
            Operand8::IndirectHL => mmu.write8(self.registers.read16(&Reg16::HL), result),
            Operand8::Register(ref reg) => self.registers.write8(reg, result),
        }
        
        if destination_num == 6 { 12 } else { 8 }
        
    }
}
