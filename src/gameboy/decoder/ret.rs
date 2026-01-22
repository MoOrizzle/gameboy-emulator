use crate::gameboy::cpu::{Instruction, Condition};

pub fn decode_ret(opcode: u8) -> Instruction {
    let condition = match opcode {
        0xC9 | 0xD9 => Condition::Always,
        0xC0 => Condition::NZ,
        0xC8 => Condition::Z,
        0xD0 => Condition::NC,
        0xD8 => Condition::C,
        _ => unreachable!("invalid JR opcode"),
    };

    Instruction::Ret { condition }
}