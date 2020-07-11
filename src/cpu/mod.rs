//! The CPU implementation for the RISC-V ISA.
//!
//! The CPU implemented here is totally generic and only
//! used to execute instructions. Mmio devices must be registered manually
//! by using [`Memory::register`].
//!
//! [`Memory::register`]: ../struct.Memory.html#register

mod config;

pub use config::CpuConfig;

use crate::{memory::Memory, registers::Registers};

/// The CPU.
pub struct Cpu {
    regs: Registers,
    memory: Memory,
}
