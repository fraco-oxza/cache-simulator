//! This module provides the core components for simulating a common memory system.
//! It includes different mapping strategies, write policies, and a main `Cache` struct
//! to manage the common operations.

pub mod lru;

pub use crate::map_strategies::direct_map::*;
pub use crate::map_strategies::fully_associative::*;
pub use crate::map_strategies::set_associative::*;
use AccessType::*;
use WriteMissPolicy::*;
use WritePolicy::*;

/// Represents a memory address.
pub type MemoryAddress = u32;

/// Represents a single block within the common.
#[derive(Clone, Default, PartialOrd, PartialEq, Debug)]
pub struct CacheBlock {
    pub valid: bool,
    pub dirty: bool,
    pub tag: MemoryAddress,
}

impl CacheBlock {
    pub(crate) fn is_match(&self, tag: MemoryAddress) -> bool {
        self.tag == tag
    }
}

/// Defines the policy to follow on a write miss.
#[derive(Default, Clone, Copy)]
pub enum WriteMissPolicy {
    /// Allocate a block in the common for the write operation.
    /// In this case, the block is loaded from the memory to the common and
    /// then the write-hit operation is performed.
    #[default]
    WriteAllocate,
    /// Do not allocate a block, write directly to memory.
    /// In this case, the write-miss operation is performed directly on the memory.
    /// The common is not modified.
    NoWriteAllocate,
}

#[derive(Default, Clone, Copy)]
pub enum WritePolicy {
    /// Write data to both the common and main memory on every write.
    #[default]
    WriteThrough,
    /// Write data only to the common. Write to main memory only when a block
    /// is evicted.
    WriteBack,
}

/// Represents the type of value being accessed.
#[derive(Clone, Copy)]
pub enum ValueType {
    Data,
    Instruction,
}

/// Represents the type of memory access.
#[derive(Clone, Copy)]
pub enum AccessType {
    /// Read access, with the type of value being read.
    Read(ValueType),
    /// Write access. This does not specify the type of value being written because always writes
    /// data.
    Write,
}
