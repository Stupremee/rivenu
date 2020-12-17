//! The memory implementation for the RISC-V architecture.
//!
//! A RISC-V hart has a single byte-addressable address space of `1 << XLEN`
//! bytes for all memory accesses.
//! This does not mean that a RISC-V hart has `1 << XLEN` bytes of memory
//! available.
//!
//! # Memory types
//! - **Halfword** = u16 (2 bytes)
//! - **Word** = u32 (4 bytes)
//! - **Doubleword** = u64 (8 bytes)
//! - **Quadword** = u128 (16 bytes)
//!
//! The address space of the RISC-V ISA is circular.
//! Accordingly, memory addresses are computed using `addr % (1 << XLEN)`.
//!
//! See chapter 1.4 in the [`RISC-V Spec`].
//!
//! [`RISC-V Spec`]: https://riscv.org/specifications/isa-spec-pdf/

use crate::Base;
use std::marker::PhantomData;

/// The default `MEMORY_SIZE` is 128MiB.
pub const MEMORY_SIZE: usize = 0x1000000;

/// The memory that is responsible for reading and writing
/// different types into the raw memory of the CPU.
///
/// Note that `Memory` does not include the Memory Manage Unit.
/// To use the MMU use `Mmu` instead.
pub struct Memory<B: Base> {
    memory: Box<[u8]>,
    _data: PhantomData<B>,
}

impl<B: Base> Memory<B> {
    /// Creates a new [`Memory`] with the [default memory size](MEMORY_SIZE).
    pub fn new() -> Self {
        Self::with_size(MEMORY_SIZE)
    }

    /// Creates a new [`Memory`] with the given size in bytes.
    pub fn with_size(size: usize) -> Self {
        Self {
            memory: vec![0u8; size].into_boxed_slice(),
            _data: PhantomData,
        }
    }
}
