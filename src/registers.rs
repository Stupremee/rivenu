//! Implementation of the registers for the RISC-V ISA.
//!
//! RISC-V has 32 integer registers which are [`XLEN`] bits wide.
//! There's also one additional register, `pc` the program counter.
//! There's no dedicated register that is used for the stack pointer,
//! or subroutine return address. The instruction encoding allows any
//! x register to be used for that purpose.
//!
//! However the standard calling conventions uses `x1` to store the return address of call,
//! with `x5` as an alternative link register, and `x2` as the stack pointer.
//!
//! The `x0` register is hardwired to zero and will ignore any writes.
//! `x1`-`x31` are general purpose registers
//!
//! [`XLEN`]: ../constant.XLEN.html

use crate::Address;

/// The `Register` type specifies the width of every x register.
///
/// The width is 64bits on RV64I and 32bits on R32I.
#[cfg(feature = "rv64i")]
pub type IntRegister = u64;
/// The `Register` type specifies the width of every x register.
///
/// The width is 64bits on RV64I and 32bits on R32I.
#[cfg(feature = "rv32i")]
pub type IntRegister = u32;

/// The `Registers` struct holds all registers that are accessible by
/// the CPU.
#[derive(Default)]
pub struct Registers {
    /// The x registers, or integer registers.
    ///
    /// We only hold 31 registers here, because the `x0` register is hardcoded
    /// in the read / write methods.
    xregs: [IntRegister; 31],
    /// The current program counter.
    pc: Address,
    // TODO: Other registers.
}

impl Registers {
    /// Creates a new `Registers` struct, with all registers set to 0.
    pub fn new() -> Self {
        Default::default()
    }

    /// Returns a copy of the current program counter.
    pub fn pc(&self) -> Address {
        self.pc
    }

    /// Returns a mutable reference to the program counter,
    /// which can be used to mutate the `pc`.
    pub fn pc_mut(&mut self) -> &mut Address {
        &mut self.pc
    }

    /// Reads the value of the from the integer register `reg`.
    ///
    /// Panics if the given register index is out of bounds.
    pub fn read_x_reg(&self, reg: usize) -> IntRegister {
        if reg == 0 {
            0
        } else {
            self.xregs[reg]
        }
    }

    /// Writes the given `val` into the integer register `reg`.
    ///
    /// A write to `x0` will result in a noop, and write into a register
    /// that is not valid, will cause a panic.
    pub fn write_x_reg(&mut self, reg: usize, val: IntRegister) {
        if reg != 0 {
            self.xregs[reg] = val;
        }
    }
}
