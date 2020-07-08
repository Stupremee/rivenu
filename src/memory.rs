//! The memory implementation for the RISC-V architecture.
//!
//! A RISC-V hart has a single byte-addressable address space of 1 << [`XLEN`] bytes for all
//! memory accesses.
//!
//! # Memory types
//! - **Halfword** = u16 (2 bytes)
//! - **Word** = u32 (4 bytes)
//! - **Doubleword** = u64 (8 bytes)
//! - **Quadword** = u128 (16 bytes)
//!
//! The address space of the RISC-V ISA is circular.
//! Accordingly, memory addresses are computed by wrapping around modulo 1 << [`XLEN`].
//!
//! See chapter 1.4 in the [`RISC-V Spec`].
//!
//! [`XLEN`]: ../constant.XLEN.html
//! [`RISC-V Spec`]: https://riscv.org/specifications/isa-spec-pdf/

use crate::Address;

/// The default `MEMORY_SIZE` is 128MiB.
// TODO: This can be changed via CLI parameter
pub const MEMORY_SIZE: usize = 0x1000000;

/// A device thats maps at one, or more addresses and will be used to read / write
/// to the addresses.
///
/// Such a device can be, for example UART.
pub trait Mmio {
    /// Reads from the given `addr` in this Mmio device.
    fn read(&self, addr: Address) -> u8;

    /// Writes to the given `addr` in this Mmio device.
    fn write(&mut self, addr: Address, val: u8);

    /// Whether this Mmio device will be used to read / write to the given address.
    fn maps_at(&self, addr: Address) -> bool;
}

/// The memory struct.
pub struct Memory {
    ram: Vec<u8>,
    mmios: Vec<Box<dyn Mmio>>,
}

impl Memory {
    /// Creates a new `Memory` struct with the default [`MEMORY_SIZE`] and no [`Mmio`] devices.
    ///
    /// [`MEMORY_SIZE`]: ./constant.MEMORY_SIZE.html
    /// [`Mmio`]: ./trait.Mmio.html
    pub fn new() -> Self {
        Self {
            ram: vec![0; MEMORY_SIZE],
            mmios: Vec::new(),
        }
    }

    /// Registers a new [`Mmio`] device.
    pub fn register(&mut self, mmio: Box<dyn Mmio>) {
        self.mmios.push(mmio);
    }

    /// Reads a single byte from the `ram` or the registered [`Mmio`] devices.
    ///
    /// It'll read from the **first** [`Mmio`] device that maps at the `addr`.
    ///
    /// [`Mmio`]: ./trait.Mmio.html
    pub fn read<V: MemoryValue>(&self, addr: Address) -> V {
        let size = std::mem::size_of::<V>();
        let addr = addr % self.ram.len() as u64;

        let bytes = if let Some(mmio) = self.mmios.iter().find(|mmio| mmio.maps_at(addr)) {
            (0..size)
                .map(|i| mmio.read(addr + i as u64))
                .collect::<Vec<u8>>()
        } else {
            (0..size)
                .map(|i| self.ram[addr as usize + i])
                .collect::<Vec<u8>>()
        };
        V::from_bytes(&bytes)
    }

    /// Writes a single byte into the `ram` or the registered [`Mmio`] devices.
    ///
    /// It'll write into the **first** [`Mmio`] device that maps at the `addr`.
    ///
    /// [`Mmio`]: ./trait.Mmio.html
    pub fn write<V: MemoryValue>(&mut self, addr: Address, val: V) {
        let addr = addr % self.ram.len() as u64;
        let bytes = val.to_bytes();

        let mmio = self.mmios.iter_mut().find(|mmio| mmio.maps_at(addr));

        if let Some(mmio) = mmio {
            bytes
                .into_iter()
                .enumerate()
                .for_each(|(i, val)| mmio.write(addr + i as u64, val));
        } else {
            bytes
                .into_iter()
                .enumerate()
                .for_each(|(i, val)| self.ram[addr as usize + i] = val);
        }
    }
}

macro_rules! value_impl {
    ($($t:ty),*) => {
        $(impl MemoryValue for $t {
            fn to_bytes(self) -> Vec<u8> {
                self.to_le_bytes().into()
            }

            fn from_bytes(bytes: &[u8]) -> Self {
                use std::convert::TryInto;
                Self::from_le_bytes(bytes.try_into().unwrap())
            }
        })*
    };
}

/// Any value that can be converted to Little Endian bytes
/// and then be written into [`Memory`].
///
/// [`Memory`]: ./struct.Memory.html
pub trait MemoryValue {
    /// Converts `self` into little endian bytes.
    // TODO: Don't allocate bytes here.
    fn to_bytes(self) -> Vec<u8>;

    /// Converts the given bytes into `Self`.
    ///
    /// Panics if `bytes` are too much / less.
    fn from_bytes(bytes: &[u8]) -> Self;
}

value_impl!(u8, i8, u16, i16, u32, i32, u64, i64);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_write_memory() {
        let mut mem = Memory::new();

        mem.write(0xDEAD, 12u8);
        assert_eq!(mem.read::<u8>(0xDEAD), 12);

        mem.write(0xABCD, 1337u32);
        assert_eq!(mem.read::<u32>(0xABCD), 1337);

        mem.write(0xDDDD, -1234i64);
        assert_eq!(mem.read::<i64>(0xDDDD), -1234);
    }
}
