use crate::{
    cpu::{csr, PrivilegeMode, Registers},
    memory,
    trap::Exception,
    Address, Base,
};
use bitflags::bitflags;
use bytemuck::Pod;
use derive_more::Display;
use std::{mem, rc::Rc};

/// The result type for MMU operations.
pub type Result<T, E = Exception> = std::result::Result<T, E>;

/// The size of a page inside the MMU is 4KiB.
pub const PAGE_SIZE: u64 = 1 << 12;

/// Defines the addressing mode that the MMU will use.
#[derive(Debug, Clone, Copy, Display)]
pub enum AddressingMode {
    None,
    SV32,
    SV39,
}

impl AddressingMode {
    pub(crate) fn levels(&self) -> u64 {
        match self {
            AddressingMode::SV32 => 2,
            AddressingMode::SV39 => 3,
            AddressingMode::None => unreachable!("levels shouldn't called on `None` mode"),
        }
    }

    pub(crate) fn pte_size(&self) -> u64 {
        match self {
            AddressingMode::SV32 => 4,
            AddressingMode::SV39 => 8,
            AddressingMode::None => unreachable!("levels shouldn't called on `None` mode"),
        }
    }

    pub(crate) fn pte_count(&self) -> u64 {
        match self {
            AddressingMode::SV32 => 1 << 10,
            AddressingMode::SV39 => 1 << 9,
            AddressingMode::None => unreachable!("levels shouldn't called on `None` mode"),
        }
    }
}

/// Specifies different methods of accessing memory.
#[derive(Debug, Clone, Copy)]
pub enum AccessMode {
    Execute,
    Read,
    Write,
}

/// Represents a virtual address that has to be converted
/// to a physical address by the MMU.
#[repr(transparent)]
#[derive(Debug, Display, Clone, Copy)]
pub struct VirtAddr(u64);

impl VirtAddr {
    pub(crate) fn vpn(&self, idx: u64, mode: AddressingMode) -> u16 {
        use AddressingMode::*;

        match (mode, idx) {
            (SV32, 0) => ((self.0 >> 12) & 0x3FF) as u16,
            (SV32, 1) => ((self.0 >> 22) & 0x3FF) as u16,

            (SV39, 0) => ((self.0 >> 12) & 0x1FF) as u16,
            (SV39, 1) => ((self.0 >> 21) & 0x1FF) as u16,
            (SV39, 2) => ((self.0 >> 30) & 0x1FF) as u16,

            (SV48, 0) => ((self.0 >> 12) & 0x1FF) as u16,
            (SV48, 1) => ((self.0 >> 21) & 0x1FF) as u16,
            (SV48, 2) => ((self.0 >> 30) & 0x1FF) as u16,
            (SV48, 3) => ((self.0 >> 39) & 0x1FF) as u16,

            (mode, idx) => panic!("VPN[{}] is not available in {} mode", idx, mode),
        }
    }
}

impl<A: Address> From<A> for VirtAddr {
    fn from(addr: A) -> Self {
        Self(addr.to_u64())
    }
}

/// Represents a physical address that can directly
/// be used to access the raw memory.
#[repr(transparent)]
#[derive(Debug, Display, Clone, Copy)]
pub struct PhysAddr(u64);

bitflags! {
    /// The low byte of a Sv32 page table entry.
    ///
    /// See section 4.3.1 in the priviliged specification.
    pub struct PteFlags: u8 {
        // TODO: Implement behaviour for this.
        const D = 0b1000_0000;
        const A = 0b0100_0000;
        /// Designates a global mapping
        const G = 0b0010_0000;
        /// Indicates whether this entry is accessible by user mode.
        const U = 0b0001_0000;
        /// Execute permission
        const X = 0b0000_1000;
        /// Write permission
        const W = 0b0000_0100;
        /// Read permission
        const R = 0b0000_0010;
        /// Indicates if this PTE is valid.
        ///
        /// If this bit is zero, all other bits should be ignored.
        const V = 0b0000_0001;
    }
}

/// Memory management unit.
///
/// The MMU is responsible for the paging process of the CPU.
/// It will take a virtual address, and convert it to a physical address.
///
/// More details about the implementation can be found in chapter 4.3 and following
/// in the [`spec`].
///
/// [`spec`]: https://riscv.org/specifications/privileged-isa/
pub struct Mmu<B: Base> {
    registers: Rc<Registers<B>>,
    mode: AddressingMode,
}

impl<B: Base> Mmu<B> {
    /// Creates a new `Mmu` that will read/write from/into the given
    /// registers.
    pub fn new(registers: Rc<Registers<B>>) -> Self {
        let (mode, _, _) = decode_satp_reg::<B>(registers.read_csr(csr::SATP));

        let mode = match (mode, B::XLEN) {
            (0, 32) => AddressingMode::None,
            (1, 32) => AddressingMode::SV32,

            (0, 64) => AddressingMode::None,
            (8, 64) => AddressingMode::SV39,
            (9, 64) => AddressingMode::SV48,

            (mode, xlen) => panic!("invalid addressing mode {} for XLEN {}", mode, xlen),
        };
        Self { registers, mode }
    }

    fn translate_addr(&self, va: VirtAddr) -> Result<PhysAddr, Exception> {
        if let AddressingMode::None = self.mode {
            return Ok(PhysAddr(va.0));
        }

        assert!(matches!(self.mode, AddressingMode::SV32));

        let satp = self.registers.read_csr(csr::SATP);
        let (_, _, ppn) = decode_satp_reg::<B>(satp);

        let a = ppn * PAGE_SIZE;
        let i = self.mode.levels() - 1;
        let pte = a + va.vpn(i, self.mode) as u64 * self.mode.pte_size();

        todo!()
    }
}

/// Takes a value that was read from the `satp` CSR and
/// decodes it into it's components, `mode`, `asid`, and `ppn`.
fn decode_satp_reg<B: Base>(value: B::Addr) -> (u8, u16, u64) {
    let value = value.to_u64();

    match B::XLEN {
        32 => {
            let ppn = value & 0x3F_FFFF;
            let asid = ((value >> 22) & 0x1FF) as u16;
            let mode = (value >> 31) as u8;
            (mode, asid, ppn)
        }
        64 => {
            let ppn = value & ((1 << 44) - 1);
            let asid = ((value >> 44) & 0xFFFF) as u16;
            let mode = ((value >> 60) & 0xF) as u8;
            (mode, asid, ppn)
        }
        _ => panic!("XLEN {} is not supported in the MMU", B::XLEN),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn decode_32_satp_reg() {
        // 0b1_100110111_1101110110101101110111u32;
        let raw = 0xCDF7_6B77u32;

        let (mode, asid, ppn) = super::decode_satp_reg::<crate::RV32I>(raw);
        assert_eq!(mode, 0x1);
        assert_eq!(asid, 0x137);
        assert_eq!(ppn, 0x376B77);
    }

    #[test]
    fn decode_64_satp_reg() {
        // 0b1000_1001110001110011_10100111010001011010010101010100101110111011
        let raw = 0x89C7_3A74_5A55_4BBBu64;

        let (mode, asid, ppn) = super::decode_satp_reg::<crate::RV64I>(raw);
        assert_eq!(mode, 0x8);
        assert_eq!(asid, 0x9C73);
        assert_eq!(ppn, 0xA74_5A55_4BBB);
    }
}
