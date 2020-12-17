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

use bytemuck::Pod;

use crate::Base;
use std::{convert::TryInto, marker::PhantomData, mem};

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

    /// Writes a [`Pod`] into the memory at the given address.
    ///
    /// ## Panics
    ///
    /// - if address is out of bounds
    /// - if address is unaligned to `T`
    /// - if transmute to `T` failed
    /// - if address can not be converted into a `usize`
    pub fn write<T: Pod>(&mut self, addr: B::Addr, value: T) {
        let addr = match TryInto::<usize>::try_into(addr) {
            Ok(addr) => addr,
            Err(_) => panic!("address conversion to usize failed"),
        };
        let bytes = bytemuck::bytes_of(&value);
        let target = &mut self.memory[addr..addr + bytes.len()];
        target.copy_from_slice(bytes);
    }

    /// Reads a [`Pod`] from the memory at the given address.
    ///
    /// ## Panics
    ///
    /// - if address is out of bounds
    /// - if address is unaligned to `T`
    /// - if transmute to `T` failed
    /// - if address can not be converted into a `usize`
    pub fn read<T: Pod>(&self, addr: B::Addr) -> T {
        let addr = match TryInto::<usize>::try_into(addr) {
            Ok(addr) => addr,
            Err(_) => panic!("address conversion to usize failed"),
        };
        let bytes = &self.memory[addr..addr + mem::size_of::<T>()];
        bytemuck::from_bytes::<T>(bytes).clone()
    }
}

#[cfg(test)]
mod tests {
    use super::Memory;
    use crate::RV64I;

    #[test]
    fn read_write() {
        let mut memory = Memory::<RV64I>::with_size(1024);

        memory.write(0xA0, 1);
        assert_eq!(memory.read::<i32>(0xA0), 1);

        memory.write(0x00, [13u8, 32, 43, 54]);
        assert_eq!(memory.read::<[u8; 4]>(0x00), [13u8, 32, 43, 54]);

        let num = 0xAAAA_BBBBu32;
        memory.write(0x08, num.to_be());
        assert_eq!(u32::from_be_bytes(memory.read::<[u8; 4]>(0x08)), num);
    }
}
