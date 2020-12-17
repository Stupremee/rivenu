//! All things related to traps, exceptions, and interrupts.
//!
//! See section 3.1.16 in the Volume 2 (Priviliged) Specification.

/// All different interrupt kinds.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Interrupt {
    // Software interrupts
    UserSoftware,
    SupervisorSoftware,
    MachineSoftware,

    // Timers
    UserTimer,
    SupervisorTimer,
    MachineTimer,

    // External interrupts
    UserExternal,
    SupervisorExternal,
    MachineExternal,
}

/// All different exception kinds.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Exception {
    InstructionAddressMisaligned,
    InstructionAccessFault,
    IllegalInstruction,
    Breakpoint,
    LoadAddressMisaligned,
    LoadAccessFault,
    StoreAddressMisaligned,
    StoreAccessFault,
    UserModeEnvironmentCall,
    SupervisorModeEnvironmentCall,
    MachineModeEnvironmentCall,
    InstructionPageFault,
    LoadPageFault,
    StorePageFault,
    Reserved,
}
