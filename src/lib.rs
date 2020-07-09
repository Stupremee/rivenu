//! Some real documentation is coming soon.
//!
//! TODO: Write proper documentation here
#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

// TODO: Both bases can be enabled and the base is provided via a CLI parameter.
#[cfg(all(feature = "rv64i", feature = "rv32i"))]
compile_error!("Only one base can be enabled at the same time.");

#[cfg(all(not(feature = "rv64i"), not(feature = "rv32i")))]
compile_error!("You have to enable on base feature (rv64i or rv32i)");

pub mod instruction;
pub mod memory;
pub mod registers;

/// The XLEN constant specifies the length of the integer registers and the
/// address space.
#[cfg(feature = "rv64i")]
pub const XLEN: usize = 64;
/// The XLEN constant specifies the length of the integer registers and the
/// address space.
#[cfg(feature = "rv32i")]
pub const XLEN: usize = 32;

/// The address type specifies the type which will be used to access
/// the memory.
///
/// The `Address` type is a 64bit wide unsigned integer if the base is `RV64I`
/// and a 32bit wide unsigned integer if the base is `R32I`.
#[cfg(feature = "rv64i")]
pub type Address = u64;
/// The address type specifies the type which will be used to access
/// the memory.
///
/// The `Address` type is a 64bit wide unsigned integer if the base is `RV64I`
/// and a 32bit wide unsigned integer if the base is `R32I`.
#[cfg(feature = "rv32i")]
pub type Address = u32;
