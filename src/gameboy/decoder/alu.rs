use super::super::cpu::{Instruction, Operand8, Operand16, Reg16Source, alu::{AluOperation, Alu16Operation}, registers::Reg16};

pub fn decode_alu8_r8(opcode: u8) -> Instruction {
    let operation = match (opcode >> 3) & 0x07 {
        0 => AluOperation::ADD,
        1 => AluOperation::ADC,
        2 => AluOperation::SUB,
        3 => AluOperation::SBC,
        4 => AluOperation::AND,
        5 => AluOperation::XOR,
        6 => AluOperation::OR,
        7 => AluOperation::CP,
        _ => unreachable!()
    };

    let operand = Operand8::from(opcode & 0x07);

    Instruction::Alu8 { operation, operand }
}

pub fn decode_alu8_n8(opcode: u8) -> Instruction {
    let operation = match (opcode >> 3) & 0x07 {
        0 => AluOperation::ADD,
        1 => AluOperation::ADC,
        2 => AluOperation::SUB,
        3 => AluOperation::SBC,
        4 => AluOperation::AND,
        5 => AluOperation::XOR,
        6 => AluOperation::OR,
        7 => AluOperation::CP,
        _ => unreachable!()
    };

    Instruction::Alu8 { operation, operand: Operand8::Imm8 }
}

pub fn decode_inc8_dec8(opcode: u8) -> Instruction {
    let operation = match opcode & 0x01 {
        0 => AluOperation::INC,
        1 => AluOperation::DEC,
        _ => unreachable!()
    };

    let operand= Operand8::from((opcode >> 3) & 0x07);
    Instruction::Alu8 { operation, operand }
}

pub fn decode_inc16_dec16(opcode: u8) -> Instruction {
    let operation = match (opcode >> 3) & 0x01 {
        0 => Alu16Operation::INC,
        1 => Alu16Operation::DEC,
        _ => unreachable!()
    };

    let operand = Operand16::from((opcode >> 4) & 0x07);
    Instruction::Alu16 { operation, operand }
}

pub fn decode_add_hl_r16(opcode: u8) -> Instruction {
    let src = match opcode {
        0x09 => Reg16Source::Reg(Reg16::BC),
        0x19 => Reg16Source::Reg(Reg16::DE),
        0x29 => Reg16Source::Reg(Reg16::HL),
        0x39 => Reg16Source::SP,
        _ => unreachable!(),
    };

    Instruction::AddHlR16 { src }
}