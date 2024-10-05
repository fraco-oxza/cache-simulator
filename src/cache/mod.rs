mod direct_map;

use AccessType::*;
use ValueType::*;
use WriteMissPolicy::*;
use WritePolicy::*;

type MemoryAddress = u32;


struct BaseCache {
    block_size: usize,
    cache_size: usize,
    cache_mask_size: usize,
    blocks: Box<[CacheBlock]>,
    write_policy: WritePolicy,
    on_write_miss: WriteMissPolicy,
}

#[derive(Clone)]
pub struct CacheBlock {
    pub valid: bool,
    pub dirty: bool,
    pub tag: MemoryAddress, // TODO: Maybe improve this
}

impl CacheBlock {
    fn new(block_size: usize) -> Self {
        Self {
            valid: false,
            dirty: false,
            tag: 0,
        }
    }

    fn is_match(&self, tag: MemoryAddress) -> bool {
        self.tag == tag
    }
}

#[derive(Default, Clone, Copy)]
enum WriteMissPolicy {
    #[default]
    WriteAllocate,
    NoWriteAllocate,
}

#[derive(Default, Clone, Copy)]
enum WritePolicy {
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

pub trait Cache: MemoryAccess {
    fn get_write_policy(&self) -> WritePolicy;
    fn get_on_miss_write_policy(&self) -> WriteMissPolicy;

    fn calculate_tag(&self, address: MemoryAddress) -> MemoryAddress;

    fn get_block(&mut self, address: MemoryAddress) -> &mut CacheBlock;

    fn log_miss(&mut self, access_type: &AccessType);
    fn log_reference(&mut self, access_type: &AccessType);

    fn access(&mut self, access_type: AccessType, address: MemoryAddress) {
        self.log_reference(&access_type);

        let write_policy = self.get_write_policy();
        let on_write_miss = self.get_on_miss_write_policy();
        let tag = self.calculate_tag(address);
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
        self.log_miss(&access_type);
        let block = self.get_block(address);

        match (access_type, write_policy, on_write_miss) {
            (Read(_), WriteThrough, _) => {
                block.tag = tag;
                self.memory_read_block();
            }
            (Read(_), WriteBack, _) => {
                block.tag = tag; // FIXME: This line should be down, but I had problems with lifetimes

                if block.dirty {
                    self.memory_write_block();
                }

                self.memory_read_block();
            }
            (Write, _, NoWriteAllocate) => {
                self.memory_write_word();
            }
            (Write, WriteThrough, WriteAllocate) => {
                block.tag = tag;
                self.memory_read_block();
                self.memory_write_word();
            }
            (Write, WriteBack, WriteAllocate) => {
                block.tag = tag;
                block.dirty = true;

                self.memory_read_block();
                self.memory_write_block();
            }
        }
    }
}

