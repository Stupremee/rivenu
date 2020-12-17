//! Implementation of the actual CPU emulation.

mod register;
pub use register::*;

/// Specifies the availabe privilege modes that a RISC-V hart
/// can run in.
#[derive(Debug, Clone, Copy)]
pub enum PrivilegeMode {
    User,
    Supervisor,
    Reserved,
    Machine,
}
