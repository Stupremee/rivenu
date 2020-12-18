//! Implementation of Insctruction related things, such as parsing
//! instructions or data strucutres that represent Instructions.
//!
//! There are 6 (and some other that doesn't really matter right now) ways, to encode a instruction and it's
//! operands. R(egister), I(mmediate), U(pper immediate), S(tore), B(ranch), J(ump).
//!
//! See the [`spec`] chapter 2.3 for more information.
//!
//! [`spec`]: https://riscv.org/specifications/isa-spec-pdf/

mod parse;
pub use parse::*;

use crate::cpu::XRegister;
use std::fmt;

/// A general RISC-V Instruction composed of a [`Variant`],
/// a [`Kind`] and the raw instruction bytes.
///
/// [`Variant`]: ./enum.Variant.html
/// [`Kind`]: ./enum.Kind.html
#[derive(Debug)]
pub struct Instruction {
    /// The operands of this `Instruction`
    pub variant: Variant,
    /// The `Kind` of this instruction (e.g. `ld`, `addi`, `jal`)
    pub kind: Kind,
    /// The raw bytes of this instruction.
    pub raw: u32,
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Kind::ECALL | Kind::EBREAK = self.kind {
            write!(f, "{}", self.kind)
        } else {
            write!(f, "{} {}", self.kind, self.variant)
        }
    }
}

/// The different encoding variants for immediate values.
///
/// Details can be found in chapter 2.3 in the [`spec`].
///
/// [`spec`]: https://riscv.org/specifications/isa-spec-pdf/
#[derive(Debug)]
pub enum Variant {
    /// The R(egister) variant is used to process data from two
    /// source registers, and store the result in a destination register.
    R {
        /// Destination
        rd: XRegister,
        /// Source 1
        rs1: XRegister,
        /// Source 2
        rs2: XRegister,
    },

    /// The I(mmediate) variant is used to process data from a register
    /// and a immediate value, and store the result in a destination register.
    I {
        /// Immediate
        val: i32,
        /// Destination
        rd: XRegister,
        /// Source 1
        rs1: XRegister,
    },

    /// The S(tore) variant is used to store data to some memory
    /// address.
    S {
        /// Immediate
        val: i32,
        /// Source 1
        rs1: XRegister,
        /// Source 2
        rs2: XRegister,
    },

    /// The B(ranch) variant is used to compare two registers
    /// and then jump to a relative address if some condition is true.
    B {
        /// The relative offset encoded in multiples of 2 bytes.
        ///
        /// The conditional branch range is ±4 KiB.
        val: i32,
        /// Source 1
        rs1: XRegister,
        /// Source 2
        rs2: XRegister,
    },

    /// The U(pper immediate) variant is like immediate, but without
    /// a source register and just a destination register and immediate
    /// value.
    U {
        /// Immediate
        val: i32,
        /// Destination
        rd: XRegister,
    },

    /// The J(ump) variant is used for unconditional jumps to provide
    /// a wider larger branch range (±1 MiB);
    J {
        /// The relative offset encoded in multiples of 2 bytes.
        val: i32,
        /// Destination
        rd: XRegister,
    },
}

impl fmt::Display for Variant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Variant::R { rd, rs1, rs2 } => write!(f, "r{} r{} r{}", rd, rs1, rs2),
            Variant::I { val, rd, rs1 } => write!(f, "r{} r{} 0x{:x}", rd, rs1, val),
            Variant::S { val, rs1, rs2 } | Variant::B { val, rs1, rs2 } => {
                write!(f, "0x{:x} r{} r{}", val, rs1, rs2)
            }
            Variant::U { val, rd } | Variant::J { val, rd } => write!(f, "r{} 0x{:x}", rd, val),
        }
    }
}

/// Internel macro to generate the `Kind` enum.
macro_rules! kind_enum {
    ($($entry:ident -> $str:expr),*$(,)?) => {
        use derive_more::Display;
        /// A `Kind` represents any instruction kind (e.g `ld`, `addi`, etc).
        #[allow(non_camel_case_types)]
        #[derive(Display, Debug, Clone, Copy)]
        pub enum Kind {
            $(
                #[display(fmt = $str)]
                $entry,
            )*
        }
    };
}

kind_enum! {
    ADDI -> "addi",
    SLTI -> "slti",
    SLTIU -> "sltiu",
    ANDI -> "andi",
    ORI -> "ori",
    XORI -> "xori",
    SLLI -> "slli",
    SRLI -> "srli",
    SRAI -> "srai",
    ADD -> "add",
    SLT -> "slt",
    SLTU -> "sltu",
    AND -> "and",
    OR -> "or",
    XOR -> "xor",
    SLL -> "sll",
    SLR -> "slr",
    SUB -> "sub",
    SRL -> "srl",
    SRA -> "sra",

    LUI -> "lui",
    AUIPC -> "auipc",

    JAL -> "jal",
    JALR -> "jalr",

    BEQ -> "beq",
    BNE -> "bne",
    BLT -> "blt",
    BGE -> "bge",
    BLTU -> "bltu",
    BGEU -> "bgeu",

    LW -> "lw",
    LH -> "lh",
    LB -> "lb",
    LHU -> "lhu",
    LBU -> "lbu",

    SW -> "sw",
    SH -> "sh",
    SB -> "sb",

    FENCE -> "fence",
    FENCE_I -> "fence_i",

    ECALL -> "ecall",
    EBREAK -> "ebreak",
    LWU -> "lwu",
    LD -> "ld",
    SD -> "sd",

    ADDIW -> "addiw",
    SLLIW -> "slliw",
    SRLIW -> "srliw",
    SRAIW -> "sraiw",

    ADDW -> "addw",
    SUBW -> "subw",
    SLLW -> "sllw",
    SRLW -> "srlw",
    SRAW -> "sraw",
}
