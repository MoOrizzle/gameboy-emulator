use super::super::cpu::{BitOperation, Instruction, Operand8, RotateOperation};

pub fn decode_prefixed(opcode: u8) -> Instruction {
     let group = opcode >> 6;
     let bit = (opcode >> 3) & 0x07;
     let reg_num = opcode & 0x07;

     let operand = Operand8::from(reg_num);

     match group {
          0 => {
               let operation = match bit {
                    0 => RotateOperation::RLC,
                    1 => RotateOperation::RRC,
                    2 => RotateOperation::RL,
                    3 => RotateOperation::RR,
                    4 => RotateOperation::SLA,
                    5 => RotateOperation::SRA,
                    6 => RotateOperation::SWAP,
                    7 => RotateOperation::SRL,
                    _ => unreachable!(),
               };

               Instruction::Rotate { operation, operand }
          },

          1 => Instruction::Bit { operation: BitOperation::Test, bit, operand },
          2 => Instruction::Bit { operation: BitOperation::Reset, bit, operand },
          3 => Instruction::Bit { operation: BitOperation::Set, bit, operand },

          _ => unreachable!()
     }
}