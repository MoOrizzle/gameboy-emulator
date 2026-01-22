mod alu;
mod call;
mod jump;
mod load;
mod misc;
mod prefixed;
mod ret;
mod stack;

use super::cpu::{Cpu, Instruction, MiscOperation};
use super::mmu::Mmu;


pub fn decode(opcode: u8, cpu: &mut Cpu, mmu: &Mmu) -> Instruction {
    match opcode {
        0x00 => Instruction::Misc { operation: MiscOperation::Nop },
        
        0x76 => Instruction::Misc { operation: MiscOperation::Halt },

        0x10 => Instruction::Misc { operation: MiscOperation::Stop },

        0x08 => stack::decode_ld_a16_sp(cpu, mmu),

        0x09 | 0x19 | 0x29 | 0x39 => alu::decode_add_hl_r16(opcode),

        0x07 | 0x0F | 0x17 | 0x1F => misc::decode_rotate_a(opcode),

        0x06 | 0x0E | 0x16 | 0x1E | 
        0x26 | 0x2E | 0x36 | 0x3E => load::decode_ld_r8_n8(opcode),

        0x27 | 0x2F | 0x37 | 0x3F => misc::decode_misc(opcode),

        0x40..=0x7F => load::decode_ld_r8_r8(opcode),

        0x04 | 0x0C | 0x14 | 0x1C | 
        0x24 | 0x2C | 0x34 | 0x3C |

        0x05 | 0x0D | 0x15 | 0x1D | 
        0x25 | 0x2D | 0x35 | 0x3D => alu::decode_inc8_dec8(opcode),

        0x03 | 0x13 | 0x23 |
        0x0B | 0x1B | 0x2B => alu::decode_inc16_dec16(opcode),

        0x22 | 0x2A | 0x32 | 0x3A => load::decode_ld_hl_inc_dec(opcode),

        0x33 | 0x3B => stack::decode_inc_dec_sp(opcode),

        0x80..=0xBF => alu::decode_alu8_r8(opcode),

        0xC6 | 0xCE | 0xD6 | 0xDE | 
        0xE6 | 0xEE | 0xF6 | 0xFE => alu::decode_alu8_n8(opcode),
        
        0x18 | 0x20 | 0x28 | 0x30 | 0x38 => jump::decode_jr(opcode, cpu, mmu),
        0xC2 | 0xC3 | 0xCA | 0xD2 | 0xDA => jump::decode_jp(opcode, cpu, mmu),

        0xC1 | 0xD1 | 0xE1 | 0xF1 | 
        0xC5 | 0xD5 | 0xE5 | 0xF5 => stack::decode_stack(opcode),

        // PREFIX
        0xCB => {
            let prefixed_opcode = cpu.fetch_byte(mmu); 
            prefixed::decode_prefixed(prefixed_opcode)
        },

        0x31 | 0xE8 => stack::decode_stackpointer(opcode),

        0xCD | 0xC4 | 0xCC | 0xD4 | 0xDC => call::decode_call(opcode),

        0xC0 | 0xC8 | 0xC9 | 0xD0 | 0xD8 | 0xD9 => ret::decode_ret(opcode),

        0xC7 | 0xD7 | 0xE7 | 0xF7 | 0xCF | 0xDF | 0xEF | 0xFF => call::decode_rst(opcode),

        0xE0 | 0xF0 | 0xE2 | 0xF2 => load::decode_ldh(opcode, cpu, mmu),

        0xEA | 0xFA => load::decode_ld_a16(opcode, cpu, mmu),

        0xE9 => jump::decode_jp_hl(),

        0xF3 => Instruction::Misc { operation: MiscOperation::Di },

        0xF8 => load::decode_ld_hl_sp_e8(cpu, mmu),

        0xF9 => stack::decode_ld_sp_hl(),

        0xFB => Instruction::Misc { operation: MiscOperation::Ei },

        _ => unimplemented!()
    }
}