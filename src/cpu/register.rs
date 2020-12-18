use crate::Base;
use derive_more::{Display, From, Into};
use num_traits::Zero;
use std::cell::Cell;

macro_rules! register_consts {
    ($($name:ident = $val:literal;)*$(,)?) => {
        pub mod csr {
            use super::CsrRegister;
            $(
                pub const $name: CsrRegister = CsrRegister($val);
            )*
        }
    };
}

// Definitions of the CSR numbers.
//
// Not every register is defined here.
register_consts! {
    USTATUS = 0x000;
    UIE = 0x004;
    UTVEC = 0x005;

    USCRATCH = 0x040;
    UEPC = 0x041;
    UCAUSE = 0x042;
    UTVAL = 0x043;
    UIP = 0x044;


    SSTATUS = 0x100;
    SEDELEG = 0x102;
    SIDELEG = 0x103;
    SIE = 0x104;
    STVEC = 0x105;
    SCOUNTEREN = 0x106;

    SSCRATCH = 0x140;
    SEPC = 0x141;
    SCAUSE = 0x142;
    STVAL = 0x143;
    SIP = 0x144;

    SATP = 0x180;


    MVENDORID = 0xF11;
    MARCHID = 0xF12;
    MIMPID = 0xF13;
    MHARTID = 0xF14;

    MSTATUS = 0x300;
    MISA = 0x301;
    MEDELEG = 0x302;
    MIDELEG = 0x303;
    MIE = 0x304;
    MTVEC = 0x305;
    MCOUNTEREN = 0x306;

    MSCRATCH = 0x340;
    MEPC = 0x341;
    MCAUSE = 0x342;
    MTVAL = 0x343;
    MIP = 0x344;
}

/// Number of CSR registers.
pub const CSR_CAPACITY: usize = 4096;

/// Represents a X Register by his index.
#[repr(transparent)]
#[derive(Debug, Display, Clone, Copy, Hash, Eq, PartialEq, PartialOrd, Ord, From, Into)]
pub struct XRegister(u8);

/// Represents the number of a CSR register.
#[repr(transparent)]
#[derive(Debug, Display, Clone, Copy, Hash, Eq, PartialEq, PartialOrd, Ord, From, Into)]
pub struct CsrRegister(u16);

/// Implementation of the registers for the RISC-V ISA.
///
/// RISC-V has 32 integer registers which are [`XLEN`](crate::Base::XLEN) bits wide.
/// There's also one additional register, `pc` the program counter.
/// There's no dedicated register that is used for the stack pointer,
/// or subroutine return address. The instruction encoding allows any
/// x register to be used for that purpose.
///
/// However the standard calling conventions uses `x1` to store the return address of call,
/// with `x5` as an alternative link register, and `x2` as the stack pointer.
///
/// The `x0` register is hardwired to zero and will ignore any writes.
/// `x1`-`x31` are general purpose registers
#[derive(Default)]
pub struct Registers<B: Base> {
    /// The x registers, or integer registers.
    ///
    /// We only hold 31 registers here, because the `x0` register is hardcoded
    /// in the read / write methods.
    xregs: Box<[Cell<B::Addr>]>,
    /// The list of control and status registers.
    csr: Box<[Cell<B::Addr>]>,
    /// The current program counter.
    pc: B::Addr,
}

impl<B: Base> Registers<B> {
    /// Creates a new `Registers` struct, with all registers set to 0.
    pub fn new() -> Self {
        Self {
            xregs: vec![Cell::new(B::Addr::zero()); 31].into_boxed_slice(),
            // TODO: Initialize special register, like `misa`
            csr: vec![Cell::new(B::Addr::zero()); CSR_CAPACITY].into_boxed_slice(),
            pc: B::Addr::zero(),
        }
    }

    /// Returns a copy of the current program counter.
    pub fn pc(&self) -> B::Addr {
        self.pc
    }

    /// Returns a mutable reference to the program counter,
    /// which can be used to mutate the `pc`.
    pub fn pc_mut(&mut self) -> &mut B::Addr {
        &mut self.pc
    }

    /// Reads the value of the from the integer register `reg`.
    ///
    /// Panics if the given register index is out of bounds.
    pub fn read_x(&self, reg: XRegister) -> B::Addr {
        if reg.0 == 0 {
            B::Addr::zero()
        } else {
            self.xregs[reg.0 as usize].get()
        }
    }

    /// Writes the given `val` into the integer register `reg`.
    ///
    /// A write to `x0` will result in a noop, and write into a register
    /// that is not valid, will cause a panic.
    pub fn write_x(&self, reg: XRegister, val: B::Addr) {
        if reg.0 != 0 {
            self.xregs[reg.0 as usize].set(val);
        }
    }

    /// Reads a value from CSR register identified by it's number.
    pub fn read_csr(&self, reg: CsrRegister) -> B::Addr {
        self.csr[reg.0 as usize].get()
    }

    /// Writes a value into a CSR register identified by his number.
    pub fn write_csr(&self, reg: CsrRegister, value: B::Addr) {
        const READ_ONLY_REGS: &[CsrRegister] =
            &[csr::MVENDORID, csr::MARCHID, csr::MIMPID, csr::MHARTID];

        if READ_ONLY_REGS.contains(&reg) {
            return;
        }

        self.csr[reg.0 as usize].set(value);
    }
}
