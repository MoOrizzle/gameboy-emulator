use crate::gameboy::cpu::{Instruction, MiscOperation, RotateAOperation};

pub fn decode_rotate_a(opcode: u8) -> Instruction {
    let operation = match (opcode >> 3) & 0x03 {
        0 => RotateAOperation::RLCA,
        1 => RotateAOperation::RRCA,
        2 => RotateAOperation::RLA,
        3 => RotateAOperation::RRA,
        _ => unreachable!()
    };

    Instruction::RotateA { operation }
}

pub fn decode_misc(opcode: u8) -> Instruction {
    let operation = match opcode {
        0x27 => MiscOperation::Daa,
        0x2F => MiscOperation::Cpl,
        0x37 => MiscOperation::Scf,
        0x3F => MiscOperation::Ccf,
        _ => unreachable!(),
    };

    Instruction::Misc { operation }
}