//! Parsing of RISC-V instructions.

use super::{Instruction, Kind, Variant};
use crate::Base;
use std::collections::HashMap;
use std::lazy::SyncLazy;

macro_rules! kind_table {
    ($($key:expr => $kind:ident),*$(,)?) => {
        ::std::lazy::SyncLazy::new(|| {
            let mut map = ::std::collections::HashMap::new();

            $(
                map.insert($key, $crate::instruction::Kind::$kind);
            )*

            map
        });
    }
}

fn instruction_type(opcode: u8) -> Option<Type> {
    match opcode {
        0b000_0011 | 0b000_1111 | 0b001_0011 | 0b001_1011 | 0b110_0111 | 0b111_0011 => {
            Some(Type::I)
        }
        0b001_0111 | 0b011_0111 => Some(Type::U),
        0b011_0011 | 0b011_1011 => Some(Type::R),
        0b010_0011 => Some(Type::S),
        0b110_0011 => Some(Type::B),
        0b110_1111 => Some(Type::J),
        _ => None,
    }
}

fn i_kind_get<B: Base>(opcode: u8, funct3: u8) -> Option<Kind> {
    static RV32_TABLE: SyncLazy<HashMap<(u8, u8), Kind>> = kind_table! {
        (0b001_0011, 0b000) => ADDI,
        (0b001_0011, 0b010) => SLTI,
        (0b001_0011, 0b011) => SLTIU,
        (0b001_0011, 0b100) => XORI,
        (0b001_0011, 0b110) => ORI,
        (0b001_0011, 0b111) => ANDI,

        (0b000_0011, 0b000) => LB,
        (0b000_0011, 0b001) => LH,
        (0b000_0011, 0b010) => LW,
        (0b000_0011, 0b100) => LBU,
        (0b000_0011, 0b101) => LHU,

        (0b110_0111, 0b00) => JALR,

        (0b000_1111, 0b000) => FENCE,
    };

    static RV64_TABLE: SyncLazy<HashMap<(u8, u8), Kind>> = kind_table! {
        (0b000_0011, 0b110) => LWU,
        (0b000_0011, 0b011) => LD,
        (0b000_1111, 0b001) => FENCE_I,
    };

    RV32_TABLE.get(&(opcode, funct3)).cloned().or_else(|| {
        B::supports_rv64()
            .then(|| RV64_TABLE.get(&(opcode, funct3)).cloned())
            .flatten()
    })
}

fn s_kind_get<B: Base>(opcode: u8, funct3: u8) -> Option<Kind> {
    static RV32_TABLE: SyncLazy<HashMap<(u8, u8), Kind>> = kind_table! {
        (0b010_0011, 0b000) => SB,
        (0b010_0011, 0b001) => SH,
        (0b010_0011, 0b010) => SW,
    };

    static RV64_TABLE: SyncLazy<HashMap<(u8, u8), Kind>> = kind_table! {
        (0b010_0011, 0b011) => SD,
    };

    RV32_TABLE.get(&(opcode, funct3)).cloned().or_else(|| {
        B::supports_rv64()
            .then(|| RV64_TABLE.get(&(opcode, funct3)).cloned())
            .flatten()
    })
}

fn r_kind_get<B: Base>(opcode: u8, funct3: u8, funct7: u8) -> Option<Kind> {
    static RV32_TABLE: SyncLazy<HashMap<(u8, u8, u8), Kind>> = kind_table! {
        (0b011_0011, 0b000, 0b000_0000) => ADD,
        (0b011_0011, 0b000, 0b010_0000) => SUB,
        (0b011_0011, 0b001, 0b000_0000) => SLL,
        (0b011_0011, 0b010, 0b000_0000) => SLT,
        (0b011_0011, 0b011, 0b000_0000) => SLTU,
        (0b011_0011, 0b100, 0b000_0000) => XOR,
        (0b011_0011, 0b101, 0b000_0000) => SRL,
        (0b011_0011, 0b101, 0b010_0000) => SRA,
        (0b011_0011, 0b110, 0b000_0000) => OR,
        (0b011_0011, 0b111, 0b000_0000) => AND,
    };

    static RV64_TABLE: SyncLazy<HashMap<(u8, u8, u8), Kind>> = kind_table! {
        (0b011_1011, 0b000, 0b000_0000) => ADDW,
        (0b011_1011, 0b000, 0b010_0000) => SUBW,
        (0b011_1011, 0b001, 0b000_0000) => SLLW,
        (0b011_1011, 0b101, 0b000_0000) => SRLW,
        (0b011_1011, 0b101, 0b100_0000) => SRAW,
    };

    RV32_TABLE
        .get(&(opcode, funct3, funct7))
        .cloned()
        .or_else(|| {
            B::supports_rv64()
                .then(|| RV64_TABLE.get(&(opcode, funct3, funct7)).cloned())
                .flatten()
        })
}

fn b_kind_get<B: Base>(opcode: u8, funct3: u8) -> Option<Kind> {
    static RV32_TABLE: SyncLazy<HashMap<(u8, u8), Kind>> = kind_table! {
        (0b110_0011, 0b000) => BEQ,
        (0b110_0011, 0b001) => BNE,
        (0b110_0011, 0b100) => BLT,
        (0b110_0011, 0b101) => BGE,
        (0b110_0011, 0b110) => BLTU,
        (0b110_0011, 0b111) => BGEU,
    };

    RV32_TABLE.get(&(opcode, funct3)).cloned()
}

enum Type {
    R,
    I,
    S,
    B,
    U,
    J,
}

impl Type {
    #[allow(clippy::similar_names)]
    fn decode<B: Base>(&self, inst: u32) -> Option<Instruction> {
        let opcode = (inst & 0x7F) as u8;

        match self {
            Type::R => {
                let rs1 = (inst >> 15) & 0x1F;
                let rs2 = (inst >> 20) & 0x1F;
                let rd = (inst >> 7) & 0x1F;
                let funct3 = ((inst >> 12) & 0x7) as u8;
                let funct7 = ((inst >> 25) & 0x7F) as u8;

                let kind = r_kind_get::<B>(opcode, funct3, funct7)?;
                Some(Instruction {
                    variant: Variant::R {
                        rd: (rd as usize).into(),
                        rs1: (rs1 as usize).into(),
                        rs2: (rs2 as usize).into(),
                    },
                    kind,
                    raw: inst,
                })
            }
            Type::I => {
                let imm = (inst >> 20) & 0xFFF;

                let rs1 = (inst >> 15) & 0x1F;
                let funct3 = ((inst >> 12) & 0x7) as u8;
                let rd = (inst >> 7) & 0x1F;

                // ECALL and EBREAK instructions
                if opcode == 0b111_0011 {
                    let kind = match imm {
                        0 => Kind::ECALL,
                        _ => Kind::EBREAK,
                    };
                    return Some(Instruction {
                        variant: Variant::I {
                            val: 0,
                            rd: 0.into(),
                            rs1: 0.into(),
                        },
                        kind,
                        raw: inst,
                    });
                } else if B::supports_rv64() && opcode == 0b001_1011 {
                    let shifttop = (imm >> 6) & 0x7F;
                    let shamt = imm & 0x1F;

                    let kind = match funct3 {
                        0b000 => Kind::ADDIW,
                        0b001 => Kind::SLLIW,
                        0b101 if shifttop == 0 => Kind::SRLIW,
                        0b101 if shifttop != 0 => Kind::SRAIW,
                        _ => return None,
                    };

                    return Some(Instruction {
                        variant: Variant::I {
                            val: shamt as i32,
                            rd: (rd as usize).into(),
                            rs1: (rs1 as usize).into(),
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
                        let imm = if B::supports_rv64() {
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
                        let kind = i_kind_get::<B>(opcode, funct3)?;
                        (kind, imm)
                    }
                };

                Some(Instruction {
                    variant: Variant::I {
                        val: imm,
                        rd: (rd as usize).into(),
                        rs1: (rs1 as usize).into(),
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

                let kind = s_kind_get::<B>(opcode, funct3)?;

                Some(Instruction {
                    variant: Variant::S {
                        val: imm,
                        rs1: (rs1 as usize).into(),
                        rs2: (rs2 as usize).into(),
                    },
                    kind,
                    raw: inst,
                })
            }
            Type::U => {
                let rd = (inst >> 7) & 0x1F;
                let imm = (inst & 0xFFFF_F000) as i32;

                let kind = match opcode {
                    0b011_0111 => Kind::LUI,
                    0b001_0111 => Kind::AUIPC,
                    _ => return None,
                };

                Some(Instruction {
                    variant: Variant::U {
                        val: imm,
                        rd: (rd as usize).into(),
                    },
                    kind,
                    raw: inst,
                })
            }
            Type::B => {
                let imm12105 = (inst >> 25) & 0x7F;
                let imm4111 = (inst >> 7) & 0x1F;

                let imm12 = (imm12105 & 0x40) >> 6;
                let imm105 = imm12105 & 0x3F;
                let imm41 = (imm4111 & 0x1E) >> 1;
                let imm11 = imm4111 & 0x1;

                // Sign extend the immediate
                let imm = (imm12 << 12) | (imm11 << 11) | (imm105 << 5) | (imm41 << 1);
                let imm = ((imm as i32) << 19) >> 19;

                let rs1 = (inst >> 15) & 0x1F;
                let rs2 = (inst >> 20) & 0x1F;
                let funct3 = ((inst >> 12) & 0x7) as u8;

                let kind = b_kind_get::<B>(opcode, funct3)?;

                Some(Instruction {
                    variant: Variant::B {
                        val: imm,
                        rs1: (rs1 as usize).into(),
                        rs2: (rs2 as usize).into(),
                    },
                    kind,
                    raw: inst,
                })
            }
            Type::J => {
                let imm = (inst & 0xFFFF_F000) >> 12;
                let rd = (inst >> 7) & 0x1F;

                let imm20 = (imm >> 19) & 0x1;
                let imm101 = (imm >> 9) & 0x3FF;
                let imm11 = (imm >> 8) & 0x1;
                let imm1912 = imm & 0xFF;

                // Sign extend immediate
                let imm_sign = (imm20 << 20) | (imm1912 << 12) | (imm11 << 11) | (imm101 << 1);
                let imm_sign = ((imm_sign as i32) << 11) >> 11;

                let kind = match opcode {
                    0b110_1111 => Kind::JAL,
                    _ => return None,
                };

                Some(Instruction {
                    variant: Variant::J {
                        val: imm_sign,
                        rd: (rd as usize).into(),
                    },
                    kind,
                    raw: inst,
                })
            }
        }
    }
}

/// Decodes a raw 32bit instruction.
///
/// See [`spec`] for more information on how to decode instructions.
///
/// [`spec`]: https://riscv.org/specifications/isa-spec-pdf/
pub fn decode<B: Base>(raw_inst: u32) -> Option<Instruction> {
    let opcode = raw_inst & 0x7F;

    instruction_type(opcode as u8).and_then(|variant| variant.decode::<B>(raw_inst))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert(inst: u32, s: &str) {
        let decoded = decode::<crate::RV64I>(inst);
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
        assert(0x00269693, "slli r13 r13 0x2");
        assert(0x000FD013, "srli r0 r31 0x0");
        assert(0x400FD013, "srai r0 r31 0x0");
        assert(0x4000D71B, "sraiw r14 r1 0x0");
        assert(0x0010D71B, "srliw r14 r1 0x1");
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

    #[test]
    fn test_r_type() {
        assert(0x00E686B3, "add r13 r13 r14");
        assert(0x40F70733, "sub r14 r14 r15");
        assert(0x43F55513, "srai r10 r10 0x3f");
        assert(0x0020873B, "addw r14 r1 r2");
    }

    #[test]
    fn test_b_type() {
        assert(0x040B8463, "beq 0x48 r23 r0");
        assert(0x3EB51A63, "bne 0x3f4 r10 r11");
    }

    #[test]
    fn test_j_type() {
        assert(0x00C000EF, "jal r1 0xc");
    }
}
