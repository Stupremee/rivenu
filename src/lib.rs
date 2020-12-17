//! todo
#![feature(once_cell)]
#![deny(
    rust_2018_idioms,
    clippy::pedantic,
    private_intra_doc_links,
    broken_intra_doc_links
)]
#![allow(
    clippy::must_use_candidate,
    clippy::cast_possible_truncation,
    clippy::too_many_lines,
    clippy::cast_possible_wrap
)]

mod config;
pub use config::*;

pub mod cpu;
pub mod instruction;
pub mod memory;
pub mod trap;

use num_traits::Num;
use std::convert::TryInto;

/// This trait represents every type that can be used as an
/// address of the CPU/Memory.
pub trait Address: Num + TryInto<usize> + Clone + Copy {}

impl<T: Num + TryInto<usize> + Clone + Copy> Address for T {}

/// A [`Base`] represents the different RISC-V
/// base ISA.
///
/// Available features are [`RV64I`] and [`RV32I`].
pub trait Base {
    /// The address type of this `Base`.
    type Addr: Address;

    /// The XLEN specifies the number of bits in the address type
    /// for this base.
    const XLEN: usize;

    /// The number of integer registers for this base.
    ///
    /// Currently not required, but will be needed for the RV32E base.
    const REG_COUNT: usize;

    /// Returns whether this base ISA contains the `RV64I` instruction set.
    ///
    /// The `RV32I` instruction set will always be available.
    fn supports_rv64() -> bool;
}

/// The RV32I base integer instruction set.
pub struct RV32I;

impl Base for RV32I {
    type Addr = u32;
    const XLEN: usize = 32;
    const REG_COUNT: usize = 32;

    fn supports_rv64() -> bool {
        false
    }
}

/// The RV64I base integer instruction set.
pub struct RV64I;

impl Base for RV64I {
    type Addr = u64;
    const XLEN: usize = 64;
    const REG_COUNT: usize = 32;

    fn supports_rv64() -> bool {
        true
    }
}
