use crate::gameboy::cpu::{Instruction, Condition};

pub fn decode_call(opcode: u8) -> Instruction {
    let condition = match opcode {
        0xCD => Condition::Always,
        0xC4 => Condition::NZ,
        0xCC => Condition::Z,
        0xD4 => Condition::NC,
        0xDC => Condition::C,
        _ => unreachable!("invalid JR opcode"),
    };

    Instruction::Call { condition }
}

pub fn decode_rst(opcode: u8) -> Instruction {
    let vector = (opcode & 0x38) as u16;
    Instruction::Rst { vector }
}  