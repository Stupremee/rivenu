use crate::{cpu::PrivilegeMode, memory, trap::Exception, Address, Base};
use bytemuck::Pod;
use derive_more::Display;
use std::mem;

/// Defines the addressing mode that the MMU will use.
#[derive(Debug, Clone, Copy)]
pub enum AddressingMode {
    None,
    SV32,
    SV39,
    SV48,
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

/// Represents a physical address that can directly
/// be used to access the raw memory.
#[repr(transparent)]
#[derive(Debug, Display, Clone, Copy)]
pub struct PhysAddr(u64);
