use crate::cache_block::CacheBlock;
use crate::common::AccessType::{Read, Write};
use crate::common::WriteMissPolicy::{NoWriteAllocate, WriteAllocate};
use crate::common::WritePolicy::{WriteBack, WriteThrough};
use crate::common::{AccessType, WriteMissPolicy, WritePolicy};
use crate::logger::Logger;
use crate::map_strategies::{MapStrategy, MapStrategyFactory};
use crate::MemoryAddress;
use std::cell::RefCell;
use std::rc::Rc;

/// Represents the main common structure.
pub struct Cache {
    block_size: usize, // Bytes
    map_strategy: Box<dyn MapStrategy>,
    blocks: Box<[CacheBlock]>,
    write_policy: WritePolicy,
    on_write_miss: WriteMissPolicy,
    log: Rc<RefCell<Logger>>,
}

impl Cache {
    pub fn new(
        block_size: usize,
        cache_size: usize,
        map_strategy_factory: &dyn MapStrategyFactory,
        write_policy: WritePolicy,
        on_write_miss: WriteMissPolicy,
        log: Rc<RefCell<Logger>>,
    ) -> Self {
        let map_strategy = map_strategy_factory.generate(block_size, cache_size);
        let blocks = vec![CacheBlock::default(); cache_size].into_boxed_slice();

        Cache {
            block_size,
            map_strategy,
            blocks,
            write_policy,
            on_write_miss,
            log,
        }
    }

    /// Retrieves a mutable reference to a common block based on the address.
    fn get_block(&mut self, address: MemoryAddress) -> &mut CacheBlock {
        let block_index = self.map_strategy.map(address, &self.blocks);
        &mut self.blocks[block_index as usize]
    }

    pub fn access(&mut self, access_type: AccessType, address: MemoryAddress) {
        self.log.borrow_mut().reference(&access_type);

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

        self.log.borrow_mut().miss(&access_type);
    }

    fn memory_write_word(&mut self) {
        self.log.borrow_mut().memory_write(1);
    }

    fn memory_read_block(&mut self) {
        self.log.borrow_mut().memory_read(self.block_size as u128);
    }

    fn memory_write_block(&mut self) {
        self.log.borrow_mut().memory_write(self.block_size as u128);
    }
}
