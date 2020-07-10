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
const I_KIND_TABLE: Lazy<HashMap<(u8, u8), Kind>> = Lazy::new(|| {
    let mut map = HashMap::new();
    if cfg!(feature = "rv32i_inst") {
        map.insert((0b0010011, 0b000), Kind::ADDI);
        map.insert((0b0010011, 0b010), Kind::SLTI);
        map.insert((0b0010011, 0b011), Kind::SLTIU);
        map.insert((0b0010011, 0b100), Kind::XORI);
        map.insert((0b0010011, 0b110), Kind::ORI);
        map.insert((0b0010011, 0b111), Kind::ANDI);

        map.insert((0b0000011, 0b000), Kind::LB);
        map.insert((0b0000011, 0b001), Kind::LH);
        map.insert((0b0000011, 0b010), Kind::LW);
        map.insert((0b0000011, 0b100), Kind::LBU);
        map.insert((0b0000011, 0b101), Kind::LHU);

        map.insert((0b1100111, 0b00), Kind::JALR);

        map.insert((0b0001111, 0b000), Kind::FENCE);
    }

    if cfg!(feature = "rv64i_inst") {
        map.insert((0b0000011, 0b110), Kind::LWU);
        map.insert((0b0000011, 0b011), Kind::LD);

        map.insert((0b0001111, 0b001), Kind::FENCE_I);
    }

    map
});

const S_KIND_TABLE: Lazy<HashMap<(u8, u8), Kind>> = Lazy::new(|| {
    let mut map = HashMap::new();

    if cfg!(feature = "rv32i_inst") {
        map.insert((0b0100011, 0b000), Kind::SB);
        map.insert((0b0100011, 0b001), Kind::SH);
        map.insert((0b0100011, 0b010), Kind::SW);
    }

    if cfg!(feature = "rv64i_inst") {
        map.insert((0b0100011, 0b011), Kind::SD);
    }

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
                let funct3 = ((inst >> 12) & 0x7) as u8;
                let rd = (inst >> 7) & 0x1F;

                // ECALL and EBREAK instructions
                if opcode == 0b1110011 {
                    let kind = match imm {
                        0 => Kind::ECALL,
                        _ => Kind::EBREAK,
                    };
                    return Some(Instruction {
                        variant: Variant::I {
                            val: 0,
                            rd: 0,
                            rs1: 0,
                        },
                        kind,
                        raw: inst,
                    });
                }

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
                        let kind = I_KIND_TABLE.get(&(opcode, funct3))?.clone();
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
            Type::U => {
                let rd = (inst >> 7) & 0x1F;
                let imm = (inst & 0xFFFFF000) as i32;

                let kind = match opcode {
                    0b0110111 => Kind::LUI,
                    0b0010111 => Kind::AUIPC,
                    _ => return None,
                };

                Some(Instruction {
                    variant: Variant::U {
                        val: imm,
                        rd: rd as usize,
                    },
                    kind,
                    raw: inst,
                })
            }
            Type::S => {
                let imm = (inst >> 25) & 0x7F;
                let imm = (imm << 5) | ((inst >> 7) & 0xF);
                // Sign extend immediate value
                let imm = ((imm as i32) << 20) >> 20;

                let rs1 = (inst >> 15) & 0x1F;
                let rs2 = (inst >> 20) & 0x1F;
                let funct3 = ((inst >> 12) & 0x7) as u8;

                let kind = S_KIND_TABLE.get(&(opcode, funct3))?.clone();

                Some(Instruction {
                    variant: Variant::S {
                        val: imm,
                        rs1: rs1 as usize,
                        rs2: rs2 as usize,
                    },
                    kind,
                    raw: inst,
                })
            }
            _ => None,
        }
    }
}

pub fn decode(raw_inst: u32) -> Option<Instruction> {
    let opcode = raw_inst & 0x7F;

    if let Some(variant) = &TYPE_TABLE[opcode as usize] {
        variant.decode(raw_inst)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert(inst: u32, s: &str) {
        let decoded = decode(inst);
        assert!(decoded.is_some());
        assert_eq!(&decoded.unwrap().to_string(), s);
    }

    #[test]
    fn test_i_type() {
        assert(0x0B040413, "addi r8 r8 0xb0");
        assert(0xC00B4B13, "xori r22 r22 0xfffffc00");
        assert(0x0407E793, "ori r15 r15 0x40");
        assert(0x4807F713, "andi r14 r15 0x480");
        assert(0x01093403, "ld r8 r18 0x10");
        assert(0x00000073, "ecall");
    }

    #[test]
    fn test_u_type() {
        assert(0x00011537, "lui r10 0x11000");
        assert(0xFFFFE6B7, "lui r13 0xffffe000");
    }

    #[test]
    fn test_s_type() {
        assert(0x00B70723, "sb 0xe r14 r11");
    }
}
