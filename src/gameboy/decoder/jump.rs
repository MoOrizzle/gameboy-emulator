use crate::gameboy::{cpu::{Condition, Cpu, Instruction, JumpKind}, mmu::Mmu};

pub fn decode_jr(opcode: u8, cpu: &mut Cpu, mmu: &Mmu) -> Instruction {
    let offset = cpu.fetch_byte(mmu) as i8;

    let condition = match opcode {
        0x18 => Condition::Always,
        0x20 => Condition::NZ,
        0x28 => Condition::Z,
        0x30 => Condition::NC,
        0x38 => Condition::C,
        _ => unreachable!("invalid JR opcode"),
    };

    Instruction::Jump { 
        condition, 
        kind: JumpKind::Relative(offset) 
    }
}

pub fn decode_jp(opcode: u8, cpu: &mut Cpu, mmu: &Mmu) -> Instruction {
    let addr = cpu.fetch_word(mmu);

    let condition = match opcode {
        0xC3 => Condition::Always,
        0xC2 => Condition::NZ,
        0xCA => Condition::Z,
        0xD2 => Condition::NC,
        0xDA => Condition::C,
        _ => unreachable!("invalid JR opcode"),
    };

    Instruction::Jump { 
        condition, 
        kind: JumpKind::Absolute(addr) 
    }
}

pub fn decode_jp_hl() -> Instruction {
    Instruction::JumpHl
}