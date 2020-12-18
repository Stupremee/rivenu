//! todo
#![feature(once_cell)]
#![deny(rust_2018_idioms, private_intra_doc_links, broken_intra_doc_links)]
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

use num_traits::{FromPrimitive, Num, ToPrimitive};

/// This trait represents every type that can be used as an
/// address of the CPU/Memory.
pub trait Address: Num + Clone + Copy {
    /// Convert `self` address to a `u64`.
    fn to_u64(&self) -> u64;

    /// Convert a `u64` to a `Self`.
    ///
    /// A value passed to this function should always
    /// fit in the inner storage type (e.g. `u32` for `RV32I`)
    fn from_u64(num: u64) -> Self;
}

impl<T: Num + ToPrimitive + FromPrimitive + Clone + Copy> Address for T {
    fn to_u64(&self) -> u64 {
        self.to_u64().expect("address conversion to u64 failed")
    }

    fn from_u64(num: u64) -> Self {
        Self::from_u64(num).expect("address conversion from u64 failed")
    }
}

/// A [`Base`] represents the different RISC-V
/// base ISA.
///
/// Available features are [`RV64I`] and [`RV32I`].
pub trait Base: sealed::Sealed {
    /// The address type of this `Base`.
    type Addr: Address;

    /// The XLEN specifies the number of bits in the address type
    /// for this base.
    const XLEN: usize;

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

    fn supports_rv64() -> bool {
        false
    }
}

/// The RV64I base integer instruction set.
pub struct RV64I;

impl Base for RV64I {
    type Addr = u64;
    const XLEN: usize = 64;

    fn supports_rv64() -> bool {
        true
    }
}

mod sealed {
    pub trait Sealed {}

    impl Sealed for super::RV64I {}
    impl Sealed for super::RV32I {}
}
