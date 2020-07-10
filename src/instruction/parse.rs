//! Parsing of RISC-V instructions.

use super::{Instruction, Kind, Variant};
use once_cell::sync::Lazy;
use std::collections::HashMap;

const TYPE_TABLE: [Option<Type>; 128] = [
    /* 0b0000000 */ None,
    /* 0b0000001 */ None,
    /* 0b0000010 */ None,
    /* 0b0000011 */ Some(Type::I),
    /* 0b0000100 */ None,
    /* 0b0000101 */ None,
    /* 0b0000110 */ None,
    /* 0b0000111 */ None,
    /* 0b0001000 */ None,
    /* 0b0001001 */ None,
    /* 0b0001010 */ None,
    /* 0b0001011 */ None,
    /* 0b0001100 */ None,
    /* 0b0001101 */ None,
    /* 0b0001110 */ None,
    /* 0b0001111 */ Some(Type::I),
    /* 0b0010000 */ None,
    /* 0b0010001 */ None,
    /* 0b0010010 */ None,
    /* 0b0010011 */ Some(Type::I),
    /* 0b0010100 */ None,
    /* 0b0010101 */ None,
    /* 0b0010110 */ None,
    /* 0b0010111 */ Some(Type::U),
    /* 0b0011000 */ None,
    /* 0b0011001 */ None,
    /* 0b0011010 */ None,
    /* 0b0011011 */ None,
    /* 0b0011100 */ None,
    /* 0b0011101 */ None,
    /* 0b0011110 */ None,
    /* 0b0011111 */ None,
    /* 0b0100000 */ None,
    /* 0b0100001 */ None,
    /* 0b0100010 */ None,
    /* 0b0100011 */ Some(Type::S),
    /* 0b0100100 */ None,
    /* 0b0100101 */ None,
    /* 0b0100110 */ None,
    /* 0b0100111 */ None,
    /* 0b0101000 */ None,
    /* 0b0101001 */ None,
    /* 0b0101010 */ None,
    /* 0b0101011 */ None,
    /* 0b0101100 */ None,
    /* 0b0101101 */ None,
    /* 0b0101110 */ None,
    /* 0b0101111 */ None,
    /* 0b0110000 */ None,
    /* 0b0110001 */ None,
    /* 0b0110010 */ None,
    /* 0b0110011 */ Some(Type::R),
    /* 0b0110100 */ None,
    /* 0b0110101 */ None,
    /* 0b0110110 */ None,
    /* 0b0110111 */ Some(Type::U),
    /* 0b0111000 */ None,
    /* 0b0111001 */ None,
    /* 0b0111010 */ None,
    /* 0b0111011 */ None,
    /* 0b0111100 */ None,
    /* 0b0111101 */ None,
    /* 0b0111110 */ None,
    /* 0b0111111 */ None,
    /* 0b1000000 */ None,
    /* 0b1000001 */ None,
    /* 0b1000010 */ None,
    /* 0b1000011 */ None,
    /* 0b1000100 */ None,
    /* 0b1000101 */ None,
    /* 0b1000110 */ None,
    /* 0b1000111 */ None,
    /* 0b1001000 */ None,
    /* 0b1001001 */ None,
    /* 0b1001010 */ None,
    /* 0b1001011 */ None,
    /* 0b1001100 */ None,
    /* 0b1001101 */ None,
    /* 0b1001110 */ None,
    /* 0b1001111 */ None,
    /* 0b1010000 */ None,
    /* 0b1010001 */ None,
    /* 0b1010010 */ None,
    /* 0b1010011 */ None,
    /* 0b1010100 */ None,
    /* 0b1010101 */ None,
    /* 0b1010110 */ None,
    /* 0b1010111 */ None,
    /* 0b1011000 */ None,
    /* 0b1011001 */ None,
    /* 0b1011010 */ None,
    /* 0b1011011 */ None,
    /* 0b1011100 */ None,
    /* 0b1011101 */ None,
    /* 0b1011110 */ None,
    /* 0b1011111 */ None,
    /* 0b1100000 */ None,
    /* 0b1100001 */ None,
    /* 0b1100010 */ None,
    /* 0b1100011 */ Some(Type::B),
    /* 0b1100100 */ None,
    /* 0b1100101 */ None,
    /* 0b1100110 */ None,
    /* 0b1100111 */ Some(Type::I),
    /* 0b1101000 */ None,
    /* 0b1101001 */ None,
    /* 0b1101010 */ None,
    /* 0b1101011 */ None,
    /* 0b1101100 */ None,
    /* 0b1101101 */ None,
    /* 0b1101110 */ None,
    /* 0b1101111 */ Some(Type::J),
    /* 0b1110000 */ None,
    /* 0b1110001 */ None,
    /* 0b1110010 */ None,
    /* 0b1110011 */ Some(Type::I),
    /* 0b1110100 */ None,
    /* 0b1110101 */ None,
    /* 0b1110110 */ None,
    /* 0b1110111 */ None,
    /* 0b1111000 */ None,
    /* 0b1111001 */ None,
    /* 0b1111010 */ None,
    /* 0b1111011 */ None,
    /* 0b1111100 */ None,
    /* 0b1111101 */ None,
    /* 0b1111110 */ None,
    /* 0b1111111 */ None,
];

/// Maps a (opcode, funct3) to a `Kind`.
const KIND_TABLE: Lazy<HashMap<(u8, u8), Kind>> = Lazy::new(|| {
    let mut map = HashMap::new();
    map.insert((0b0010011, 000), Kind::ADDI);
    map.insert((0b0010011, 010), Kind::SLTI);
    map.insert((0b0010011, 011), Kind::SLTIU);
    map.insert((0b0010011, 100), Kind::XORI);
    map.insert((0b0010011, 110), Kind::ORI);
    map.insert((0b0010011, 111), Kind::ANDI);
    map
});

enum Type {
    R,
    I,
    S,
    B,
    U,
    J,
}

impl Type {
    fn decode(&self, inst: u32) -> Option<Instruction> {
        let opcode = (inst & 0x7F) as u8;

        match self {
            Type::I => {
                let imm = (inst >> 20) & 0xFFF;

                let rs1 = (inst >> 15) & 0x1F;
                let funct3 = (inst >> 12) & 0x7;
                let rd = (inst >> 7) & 0x1F;

                let (kind, imm) = match funct3 {
                    0b001 | 0b101 => {
                        let shifttop = (imm >> 6) & 0x3F;
                        // In the case this is a shift operations,
                        // the `imm` value represents the shift amount.
                        let imm = if cfg!(feature = "rv64i") {
                            imm & 0x3F
                        } else {
                            imm & 0x1F
                        };
                        let kind = match funct3 {
                            0b001 => Kind::SLLI,
                            _ if shifttop == 0 => Kind::SRLI,
                            _ if shifttop != 0 => Kind::SRAI,
                            _ => unreachable!(),
                        };
                        (kind, imm as i32)
                    }
                    _ => {
                        // Sign extend the immediate
                        let imm = ((imm as i32) << 20) >> 20;
                        let kind = KIND_TABLE.get(&(opcode, funct3 as u8))?.clone();
                        (kind, imm)
                    }
                };

                Some(Instruction {
                    variant: Variant::I {
                        val: imm,
                        rd: rd as usize,
                        rs1: rs1 as usize,
                    },
                    kind,
                    raw: inst,
                })
            }
            _ => None,
        }
    }
}

pub fn decode(raw_inst: u32) {
    let opcode = raw_inst & 0x7F;

    let inst = if let Some(variant) = &TYPE_TABLE[opcode as usize] {
        variant.decode(raw_inst)
    } else {
        None
    };

    if let Some(inst) = inst {
        println!("{}", inst);
    } else {
        println!("Instruction {:#010x} is invalid", raw_inst);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let inst = 0xFF010113u32;
        decode(inst);
        let inst = 0x00269693;
        decode(inst);
        let inst = 0x0207d793;
        decode(inst);
        let inst = 0x4037d493;
        decode(inst);
    }
}
