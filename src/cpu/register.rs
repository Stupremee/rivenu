use crate::Base;
use derive_more::{Display, From, Into};
use num_traits::Zero;

/// Represents a register by his index.
#[allow(clippy::module_name_repetitions)]
#[repr(transparent)]
#[derive(Debug, Display, Clone, Copy, Hash, Eq, PartialEq, PartialOrd, Ord, From, Into)]
pub struct RegisterIndex(usize);

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
    xregs: Box<[B::Addr]>,
    /// The current program counter.
    pc: B::Addr,
}

impl<B: Base> Registers<B> {
    /// Creates a new `Registers` struct, with all registers set to 0.
    pub fn new() -> Self {
        Self {
            xregs: vec![B::Addr::zero(); B::REG_COUNT - 1].into_boxed_slice(),
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
    pub fn read_x_reg(&self, reg: RegisterIndex) -> B::Addr {
        if reg.0 == 0 {
            B::Addr::zero()
        } else {
            self.xregs[reg.0]
        }
    }

    /// Writes the given `val` into the integer register `reg`.
    ///
    /// A write to `x0` will result in a noop, and write into a register
    /// that is not valid, will cause a panic.
    pub fn write_x_reg(&mut self, reg: RegisterIndex, val: B::Addr) {
        if reg.0 != 0 {
            self.xregs[reg.0] = val;
        }
    }
}
