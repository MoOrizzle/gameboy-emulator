use super::super::{cpu::{Cpu, HighAddr, LoadHighOperand, LoadDir, Instruction, Operand8, Operand16}, mmu::Mmu};

pub fn decode_ld_r8_n8(opcode: u8) -> Instruction {
    let dst = Operand8::from((opcode >> 3) & 0x07);
    let src = Operand8::Imm8;
    Instruction::Load8 { dst, src }
}

pub fn decode_ld_r8_r8(opcode: u8) -> Instruction {
    let dst = Operand8::from((opcode >> 3) & 0x07);
    let src = Operand8::from(opcode & 0x07);
    Instruction::Load8 { dst, src }
}

pub fn decode_ld_r16_n16(opcode: u8) -> Instruction {
    let dst = Operand16::from((opcode >> 4) & 0x03);
    let src = Operand16::Imm16;
    Instruction::Load16 { dst, src }
}

pub fn decode_ld_hl_sp_e8(cpu: &mut Cpu, mmu: &Mmu) -> Instruction {
    let offset = cpu.fetch_byte(mmu) as i8;
    Instruction::LoadHLFromSP { offset }
}


pub fn decode_ldh(opcode: u8, cpu: &mut Cpu, mmu: &Mmu) -> Instruction {
    match opcode {
        // LDH (a8), A
        0xE0 => {
            let offset = cpu.fetch_byte(mmu);
            Instruction::LoadHigh { src: LoadHighOperand::RegA, dst: LoadHighOperand::Addr(HighAddr::Imm8(offset)) }
        },
        // LDH A, (a8)
        0xF0 => {
            let offset = cpu.fetch_byte(mmu);
            Instruction::LoadHigh { src: LoadHighOperand::Addr(HighAddr::Imm8(offset)), dst: LoadHighOperand::RegA }
        },
        // LDH (C), A
        0xE2 => Instruction::LoadHigh { src: LoadHighOperand::RegA, dst: LoadHighOperand::Addr(HighAddr::RegC) },
        // LDH A, (C)
        0xF2 => Instruction::LoadHigh { src: LoadHighOperand::Addr(HighAddr::RegC), dst: LoadHighOperand::RegA },

        _ => unreachable!(),
    }
}

pub fn decode_ld_a16(opcode: u8, cpu: &mut Cpu, mmu: &Mmu) -> Instruction {
    let lo = cpu.fetch_byte(mmu) as u16;
    let hi = cpu.fetch_byte(mmu) as u16;
    let addr = (hi << 8) | lo;

    match opcode {
        // LD (a16), A
        0xEA => Instruction::LoadA16 { addr, direction: LoadDir::ToMem },
        // LD A, (a16)
        0xFA => Instruction::LoadA16 { addr, direction: LoadDir::FromMem },

        _ => unreachable!(),
    }
}

pub fn decode_ld_hl_inc_dec(opcode: u8) -> Instruction {
    let (direction, inc) = match opcode {
        0x22 => (LoadDir::ToMem,   true),
        0x2A => (LoadDir::FromMem, true),
        0x32 => (LoadDir::ToMem,   false),
        0x3A => (LoadDir::FromMem, false),
        _ => unreachable!(),
    };

    Instruction::LoadHlIncDec { direction, inc }
}

