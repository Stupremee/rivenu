//! Memory management unit.
//!
//! The MMU is responsible for the paging process of the CPU.
//! It will take a virtual address, and convert it to a physical address.
//!
//! More details about the implementation can be found in chapter 4.3 and following
//! in the [`spec`].
//!
//! [`spec`]: https://riscv.org/specifications/privileged-isa/

use bitflags::bitflags;

bitflags! {
    struct Flag: u8 {
        const D = 0b10000000;
        const A = 0b01000000;
        const G = 0b00100000;
        const U = 0b00010000;
        const X = 0b00001000;
        const W = 0b00000100;
        const R = 0b00000010;
        const V = 0b00000001;
    }
}

/// A Sv32 page table entry is used, if the Sv32 memory system is used, to map a VPN (Virtual page number)
/// into a PPN (Physical page number) and to store permissions, like
/// read / write / execute.
struct Sv32PagetableEntry {
    ppn1: u16,
    ppn2: u16,
    rsw: u8,
    flags: Flag,
}

impl Sv32PagetableEntry {
    fn from_raw(raw: u32) -> Self {
        let ppn1 = (raw >> 20) & 0xFFF;
        let ppn2 = (raw >> 10) & 0x3FF;
        let rsw = (raw >> 8) & 0x3;
        let flags = raw & 0xFF;
        Self {
            ppn1: ppn1 as u16,
            ppn2: ppn2 as u16,
            rsw: rsw as u8,
            flags: Flag::from_bits_truncate(flags as u8),
        }
    }

    fn to_raw(&self) -> u32 {
        let ppn1 = self.ppn1 as u32;
        let ppn2 = self.ppn2 as u32;
        let rsw = self.rsw as u32;
        let flags = self.flags.bits() as u32;
        (ppn1 << 20) | (ppn2 << 10) | (rsw << 8) | flags
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pte() {
        let num = 0xBAB0C127u32;
        let pte = Sv32PagetableEntry::from_raw(num);
        assert_eq!(pte.to_raw(), num);
    }
}
