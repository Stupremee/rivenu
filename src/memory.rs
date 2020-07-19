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
use mem_storage::Memory as MemoryStorage;

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
}

impl MemoryStorage for Memory {
    // TODO: Proper traps
    type Error = ();

    fn get<I>(&self, index: I) -> Result<&I::Output, Self::Error>
    where
        I: std::slice::SliceIndex<[u8]>,
    {
        self.ram.get(index).ok_or(())
    }

    fn get_mut<I>(&mut self, index: I) -> Result<&mut I::Output, Self::Error>
    where
        I: std::slice::SliceIndex<[u8]>,
    {
        self.ram.get_mut(index).ok_or(())
    }

    fn try_read_byte(&self, addr: usize) -> Result<u8, Self::Error> {
        if let Some(mmio) = self.mmios.iter().find(|mmio| mmio.maps_at(addr as Address)) {
            Ok(mmio.read(addr as Address))
        } else {
            self.get(addr).map(|x| *x)
        }
    }

    fn try_write_byte(&mut self, addr: usize, byte: u8) -> Result<(), Self::Error> {
        if let Some(mmio) = self
            .mmios
            .iter_mut()
            .find(|mmio| mmio.maps_at(addr as Address))
        {
            mmio.write(addr as Address, byte);
            Ok(())
        } else {
            let entry = self.get_mut(addr)?;
            *entry = byte;
            Ok(())
        }
    }
}

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
