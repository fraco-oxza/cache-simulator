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

pub type MemoryAddress = u32;

pub trait MapStrategyFactory {
    fn generate(&self, block_size: usize, cache_size: usize) -> Box<dyn MapStrategy>;
}

pub trait MapStrategy {
    fn map(&mut self, address: MemoryAddress, blocks: &[CacheBlock]) -> MemoryAddress;
    fn get_tag(&self, address: MemoryAddress) -> MemoryAddress;
}

pub struct Cache {
    block_size: usize,
    cache_size: usize,
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
            cache_size,
            map_strategy,
            blocks,
            write_policy,
            on_write_miss,
            log,
        }
    }
}

#[derive(Clone, Default)]
pub struct CacheBlock {
    pub valid: bool,
    pub dirty: bool,
    pub tag: MemoryAddress, // TODO: Maybe improve this
}

impl CacheBlock {
    fn is_match(&self, tag: MemoryAddress) -> bool {
        self.tag == tag
    }
}

#[derive(Default, Clone, Copy)]
pub enum WriteMissPolicy {
    #[default]
    WriteAllocate,
    NoWriteAllocate,
}

#[derive(Default, Clone, Copy)]
pub enum WritePolicy {
    #[default]
    WriteThrough,
    WriteBack,
}

#[derive(Clone, Copy)]
pub enum ValueType {
    Data,
    Instruction,
}

#[derive(Clone, Copy)]
pub enum AccessType {
    Read(ValueType),
    Write,
}

pub trait MemoryAccess {
    fn memory_write_word(&mut self);
    fn memory_read_word(&mut self);
    fn memory_read_block(&mut self);
    fn memory_write_block(&mut self);
}

impl MemoryAccess for Cache {
    fn memory_write_word(&mut self) {
        self.log.memory_write(1);
    }

    fn memory_read_word(&mut self) {
        self.log.memory_read(1);
    }

    fn memory_read_block(&mut self) {
        self.log.memory_read(self.block_size as u128);
    }

    fn memory_write_block(&mut self) {
        self.log.memory_write(self.block_size as u128);
    }
}

impl Cache {
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
                block.tag = tag; // FIXME: This line should be down, but I had problems with lifetimes
                let was_valid = block.valid;
                block.valid = true;

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
}
