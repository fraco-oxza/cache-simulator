use crate::cache::{AccessType::{self, *}, BaseCache, Cache, CacheBlock, MemoryAccess, MemoryAddress, WriteMissPolicy, WriteMissPolicy::*, WritePolicy, WritePolicy::*};
use crate::Logger;

pub struct DirectMap {
    base_cache: BaseCache,
    tag_mask_size: usize,
    block_mask_size: usize,
    log: Logger,
}

impl DirectMap {
    fn calculate_block_index(&self, address: MemoryAddress) -> MemoryAddress {
        let ld = self.tag_mask_size;
        let rs = self.block_mask_size;

        address << ld >> (ld + rs)
    }
}

impl MemoryAccess for DirectMap {
    fn memory_write_word(&mut self) {
        self.log.memory_write(1);
    }

    fn memory_read_word(&mut self) {
        self.log.memory_read(1);
    }

    fn memory_read_block(&mut self) {
        self.log.memory_read(self.base_cache.block_size as u128);
    }

    fn memory_write_block(&mut self) {
        self.log.memory_write(self.base_cache.block_size as u128);
    }
}

impl Cache for DirectMap {
    fn get_write_policy(&self) -> WritePolicy {
        self.base_cache.write_policy
    }

    fn get_on_miss_write_policy(&self) -> WriteMissPolicy {
        self.base_cache.on_write_miss
    }

    fn calculate_tag(&self, address: MemoryAddress) -> MemoryAddress {
        address >> self.block_mask_size + self.base_cache.cache_mask_size
    }

    fn get_block(&mut self, address: MemoryAddress) -> &mut CacheBlock {
        let block_index = self.calculate_block_index(address) as usize;
        &mut self.base_cache.blocks[block_index]
    }

    fn log_miss(&mut self, access_type: &AccessType) {
        self.log.miss(access_type);
    }

    fn log_reference(&mut self, access_type: &AccessType) {
        self.log.reference(access_type);
    }
}
