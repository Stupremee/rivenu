//! Configuration of a CPU.
//!
//! A CPU Configuration consists of a the raw binary code,
//! a starting program counter, mmio devices and other default values that will
//! be set on startup of the CPU.

use crate::{memory::MEMORY_SIZE, Address};
use std::{io, path::Path, string::FromUtf8Error};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ElfError {
    #[error("only RISC-V files are supported.")]
    InvalidMachine,
    #[error("i/o error: {0}")]
    IoError(#[from] io::Error),
    #[error("invalid magic number")]
    InvalidMagic,
    #[error("invalid format")]
    InvalidFormat,
    #[error("invalid utf8 data: {0}")]
    Utf8(#[from] FromUtf8Error),
    #[error("not implemented")]
    NotImplemented,
    #[error("there's no .text section in the file")]
    NoTextSection,
}

impl From<elf::ParseError> for ElfError {
    fn from(err: elf::ParseError) -> Self {
        match err {
            elf::ParseError::IoError(io) => io.into(),
            elf::ParseError::InvalidMagic => ElfError::InvalidMagic,
            elf::ParseError::InvalidFormat(None) => ElfError::InvalidFormat,
            elf::ParseError::InvalidFormat(Some(err)) => err.into(),
            elf::ParseError::NotImplemented => ElfError::NotImplemented,
        }
    }
}

/// The `CpuConfig` is used to set default values at the startup of the CPU.
pub struct CpuConfig {
    /// The address where the CPU should start executing
    pc: Address,
    /// The binary code
    binary: Vec<u8>,
    /// The size of the memory
    memory_size: Address,
}

impl CpuConfig {
    /// Creates a new `CpuConfig` that will execute the given `binary`
    /// code and starts at address 0.
    pub fn raw(binary: Vec<u8>) -> Self {
        Self {
            pc: 0,
            binary,
            memory_size: MEMORY_SIZE as u64,
        }
    }

    /// Creates a new `CpuConfig` by reading the binary code from the given
    /// file and starts executing at address 0.
    pub fn from_file<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let data = std::fs::read(path)?;
        Ok(Self::raw(data))
    }

    pub fn from_elf<P: AsRef<Path>>(path: P) -> Result<Self, ElfError> {
        let file = elf::File::open_path(path)?;

        // 0xF3 is RISC-V. We only want to load RISC-V files.
        if file.ehdr.machine.0 != 0xF3 {
            return Err(ElfError::InvalidMachine);
        }
        // TODO: Check other properties

        let pc = file.ehdr.entry;
        let text_section = file.get_section(".text").ok_or(ElfError::NoTextSection)?;
    }

    /// Returns a mutable reference to the program counter
    /// of this config. This can be used to modify the pc
    /// after creation of the struct.
    pub fn pc_mut(&mut self) -> &mut Address {
        &mut self.pc
    }

    /// Returns a mutable reference to the memory size of this config,
    /// that can be used to modify the size of the memory.
    pub fn memory_mut(&mut self) -> &mut Address {
        &mut self.memory_size
    }
}
