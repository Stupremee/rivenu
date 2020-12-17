//! todo
#![deny(
    rust_2018_idioms,
    clippy::pedantic,
    private_intra_doc_links,
    broken_intra_doc_links
)]

mod config;
pub use config::*;

pub mod memory;
pub mod trap;

use num_traits::Num;
use std::convert::TryInto;

/// This trait represents every type that can be used as an
/// address of the CPU/Memory.
pub trait Address: Num + TryInto<usize> {}

impl<T: Num + TryInto<usize>> Address for T {}

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
}

/// The RV32I base integer instruction set.
pub struct RV32I;

impl Base for RV32I {
    type Addr = u32;
    const XLEN: usize = 32;
}

/// The RV64I base integer instruction set.
pub struct RV64I;

impl Base for RV64I {
    type Addr = u64;
    const XLEN: usize = 32;
}
