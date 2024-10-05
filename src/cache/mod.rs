//! This module provides the core components for simulating a cache memory system.
//! It includes different mapping strategies, write policies, and a main `Cache` struct
//! to manage the cache operations.

mod direct_map;
mod fully_associative;
mod lru;
mod set_associative;

use crate::logger::Logger;
pub use direct_map::*;
pub use fully_associative::*;
pub use set_associative::*;
use AccessType::*;
use WriteMissPolicy::*;
use WritePolicy::*;

/// Represents a memory address.
pub type MemoryAddress = u32;

/// A factory trait for creating mapping strategies.
pub trait MapStrategyFactory {
    /// Generates a new mapping strategy instance.
    fn generate(&self, block_size: usize, cache_size: usize) -> Box<dyn MapStrategy>;
}

/// Defines the behavior of a cache mapping strategy.
pub trait MapStrategy {
    /// Maps a memory address to a cache block index.
    ///
    /// This function returns the index of the cache block where the data
    /// corresponding to the given `address` **should** be stored according
    /// to the mapping strategy.
    ///
    /// This function only indicates the potential location of
    /// the data. It does not guarantee that the data is actually present in
    /// that block. The caller is responsible for verifying the presence of
    /// the data by comparing the block's tag with the tag returned by
    /// the `get_tag` function.
    fn map(&mut self, address: MemoryAddress, blocks: &[CacheBlock]) -> MemoryAddress;

    /// Extracts the tag from a given memory address.
    ///
    /// The tag is a portion of the memory address used to identify
    /// whether a specific cache block contains the desired data.
    fn get_tag(&self, address: MemoryAddress) -> MemoryAddress;
}

/// Represents the main cache structure.
pub struct Cache {
    block_size: usize, // Bytes
    map_strategy: Box<dyn MapStrategy>,
    blocks: Box<[CacheBlock]>,
    write_policy: WritePolicy,
    on_write_miss: WriteMissPolicy,
    pub log: Logger,
}

impl Cache {
    pub fn new(
        block_size: usize,
        cache_size: usize,
        map_strategy_factory: Box<dyn MapStrategyFactory>,
        write_policy: WritePolicy,
        on_write_miss: WriteMissPolicy,
    ) -> Self {
        let map_strategy = map_strategy_factory.generate(block_size, cache_size);
        let blocks = vec![CacheBlock::default(); cache_size].into_boxed_slice();
        let log = Logger::default();

        Cache {
            block_size,
            map_strategy,
            blocks,
            write_policy,
            on_write_miss,
            log,
        }
    }
}

/// Represents a single block within the cache.
#[derive(Clone, Default)]
pub struct CacheBlock {
    pub valid: bool,
    pub dirty: bool,
    pub tag: MemoryAddress,
}

impl CacheBlock {
    fn is_match(&self, tag: MemoryAddress) -> bool {
        self.tag == tag
    }
}

/// Defines the policy to follow on a write miss.
#[derive(Default, Clone, Copy)]
pub enum WriteMissPolicy {
    /// Allocate a block in the cache for the write operation.
    /// In this case, the block is loaded from the memory to the cache and
    /// then the write-hit operation is performed.
    #[default]
    WriteAllocate,
    /// Do not allocate a block, write directly to memory.
    /// In this case, the write-miss operation is performed directly on the memory.
    /// The cache is not modified.
    NoWriteAllocate,
}

#[derive(Default, Clone, Copy)]
pub enum WritePolicy {
    /// Write data to both the cache and main memory on every write.
    #[default]
    WriteThrough,
    /// Write data only to the cache. Write to main memory only when a block
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

impl Cache {
    /// Retrieves a mutable reference to a cache block based on the address.
    fn get_block(&mut self, address: MemoryAddress) -> &mut CacheBlock {
        let block_index = self.map_strategy.map(address, &self.blocks);
        &mut self.blocks[block_index as usize]
    }

    pub fn access(&mut self, access_type: AccessType, address: MemoryAddress) {
        self.log.reference(&access_type);

        let write_policy = self.write_policy;
        let on_write_miss = self.on_write_miss;
        let tag = self.map_strategy.get_tag(address);
        let block = self.get_block(address);

        if block.valid && block.is_match(tag) {
            // HIT
            match write_policy {
                WriteThrough => self.memory_write_word(),
                WriteBack => block.dirty = true,
            }

            return;
        }

        // MISS
        match (access_type, write_policy, on_write_miss) {
            (Read(_), WriteThrough, _) => {
                block.tag = tag;
                block.valid = true;
                self.memory_read_block();
            }
            (Read(_), WriteBack, _) => {
                block.tag = tag;
                let was_valid = block.valid;

                if was_valid && block.dirty {
                    self.memory_write_block();
                }

                self.memory_read_block();
            }
            (Write, _, NoWriteAllocate) => {
                self.memory_write_word();
            }
            (Write, WriteThrough, WriteAllocate) => {
                block.tag = tag;
                block.valid = true;
                self.memory_read_block();
                self.memory_write_word();
            }
            (Write, WriteBack, WriteAllocate) => {
                block.tag = tag;
                block.valid = true;
                block.dirty = true;

                self.memory_read_block();
                self.memory_write_block();
            }
        }

        self.log.miss(&access_type);
    }

    fn memory_write_word(&mut self) {
        self.log.memory_write(1);
    }

    fn memory_read_block(&mut self) {
        self.log.memory_read(self.block_size as u128);
    }

    fn memory_write_block(&mut self) {
        self.log.memory_write(self.block_size as u128);
    }
}
