use crate::gameboy::{cpu::{Cpu, Instruction, StackOperation, StackPointerOperation, registers::Reg16}, mmu::Mmu};

pub fn decode_stack(opcode: u8) -> Instruction {
    let operation = match (opcode >> 2) & 0x01 {
        0 => StackOperation::Pop,
        1 => StackOperation::Push,
        _ => unreachable!()
    };

    let reg = Reg16::from((opcode >> 4) & 0x03); 
    Instruction::Stack { operation, reg }
}

pub fn decode_stackpointer(opcode: u8) -> Instruction {
    let operation = match (opcode >> 1) & 0x07 {
        0 => StackPointerOperation::LoadImm,
        1 => StackPointerOperation::Inc,
        5 => StackPointerOperation::Dec,
        _ => unreachable!()
    };

    Instruction::StackPointer { operation }
}

pub fn decode_ld_sp_hl() -> Instruction {
    Instruction::LoadSpFromHl
}

pub fn decode_inc_dec_sp(opcode: u8) -> Instruction {
    let inc = match opcode {
        0x33 => true,
        0x3B => false,
        _ => unreachable!(),
    };

    Instruction::IncDecSp { inc }
}

pub fn decode_ld_a16_sp(cpu: &mut Cpu, mmu: &Mmu) -> Instruction {
    let addr = cpu.fetch_word(mmu);
    Instruction::LoadA16Sp { addr }
}