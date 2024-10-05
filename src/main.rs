mod cache;
mod logger;

use cache::CacheBlock;
use logger::*;


use std::{
    collections::VecDeque,
    time::Duration,
};

const HIT_DURATION: Duration = Duration::from_nanos(5);
const MISS_DURATION: Duration = Duration::from_nanos(100);
const WORD_SIZE: usize = 4;

#[derive(Clone)]
struct LRUFinder {
    nums: VecDeque<usize>,
}

impl LRUFinder {
    fn new(size: usize) -> Self {
        Self {
            nums: (0..size).into_iter().collect(),
        }
    }

    fn log_use(&mut self, cache_address: usize) {
        let idx = self
            .nums
            .iter()
            .position(|&num| num == cache_address)
            .unwrap();

        self.nums.remove(idx);
        self.nums.push_back(cache_address);
    }

    fn get_lru(&self) -> usize {
        *self.nums.front().unwrap()
    }
}

enum CacheType {
    DirectMap,
    FullyAssociative,
    SetAssociative(u32),
}

struct Cache {
    block_size: usize,
    block_mask_size: u32, // Size of the block in the address in bits
    cache_size: usize,
    cache_mask_size: u32, // Size of the cache in the address in bits
    tag_mask_size: u32,   // Size of the tag in the address in bits
    blocks: Box<[CacheBlock]>,
    lru_finder: LRUFinder,
    lru_finder_set_associative: Option<Box<[LRUFinder]>>,
    write_policy: WritePolicy,
    on_write_miss: WriteMissPolicy,
    cache_type: CacheType,
    logger: Logger,
}

enum ReadType {
    Instruction,
    Data,
}

impl Cache {
    fn new(
        block_size: usize,
        cache_size: usize,
        write_policy: WritePolicy,
        on_write_miss: WriteMissPolicy,
        cache_type: CacheType,
    ) -> Self {
        let blocks = vec![CacheBlock::new(block_size); cache_size].into_boxed_slice();
        let lru_finder = LRUFinder::new(cache_size);
        let block_mask_size = block_size.ilog2() + WORD_SIZE.ilog2();
        let cache_mask_size = cache_size.ilog2();
        let tag_mask_size =
            8 * size_of::<MemoryAddress>() as u32 - block_mask_size - cache_mask_size;
        let logger = Logger::default();
        let lru_finder_set_associative = if let CacheType::SetAssociative(sets) = cache_type {
            let lru_finders =
                vec![LRUFinder::new(cache_size / sets as usize); sets as usize].into_boxed_slice();

            Some(lru_finders)
        } else {
            None
        };

        Self {
            block_size,
            block_mask_size,
            cache_size,
            cache_mask_size,
            tag_mask_size,
            blocks,
            logger,
            lru_finder,
            lru_finder_set_associative,
            write_policy,
            on_write_miss,
            cache_type,
        }
    }

    fn read(&mut self, read_type: ReadType, address: MemoryAddress) {
        match self.cache_type {
            CacheType::DirectMap => self.read_direct_map(read_type, address),
            CacheType::FullyAssociative => self.read_fully_associative(read_type, address),
            CacheType::SetAssociative(_) => self.read_set_associative(read_type, address),
        }
    }

    fn read_set_associative(&mut self, read_type: ReadType, address: MemoryAddress) {
        let block_index = self.get_block_index_set_associative(address);
        let sets = match self.cache_type {
            CacheType::SetAssociative(sets) => sets,
            _ => panic!("Expected a set associative cache type"),
        };
        let elements = self.cache_size / sets as usize;
        let set = self.get_set(address);
        let tag = self.get_tag(address);
        let lru = &mut self.lru_finder_set_associative.as_mut().unwrap()[set as usize];

        match read_type {
            ReadType::Instruction => self.logger.log_instruction_reference(),
            ReadType::Data => self.logger.log_data_reference(),
        }

        if let Some(idx) = block_index {
            // Hit
            dbg!("Hit");
            lru.log_use(idx);
            return;
        }

        // Miss
        dbg!("Miss");
        match read_type {
            ReadType::Instruction => self.logger.log_instruction_miss(),
            ReadType::Data => self.logger.log_data_miss(),
        }

        self.logger.log_memory_read();
        let lru_block = lru.get_lru();
        let block = &mut self.blocks[set as usize * elements + lru_block];

        if let WritePolicy::WriteBack = self.write_policy {
            if block.dirty {
                for _ in 0..self.block_size {
                    self.logger.log_memory_write();
                }
            }
        }

        block.tag = tag;
        block.valid = true;
        block.dirty = false;

        lru.log_use(lru_block);
    }

    fn read_direct_map(&mut self, read_type: ReadType, address: MemoryAddress) {
        let block_index = self.get_block_index_direct_map(address);
        let tag = self.get_tag(address);
        let block = &mut self.blocks[block_index];

        match read_type {
            ReadType::Instruction => self.logger.log_instruction_reference(),
            ReadType::Data => self.logger.log_data_reference(),
        }

        if block.valid && block.is_match(tag) {
            // Hit
            dbg!("Hit");
            return;
        }

        //Miss
        dbg!("Miss");
        match read_type {
            ReadType::Instruction => self.logger.log_instruction_miss(),
            ReadType::Data => self.logger.log_data_miss(),
        }

        self.logger.log_memory_read();

        if let WritePolicy::WriteBack = self.write_policy {
            if block.dirty {
                for _ in 0..self.block_size {
                    self.logger.log_memory_write();
                }
            }
        }

        block.tag = tag;
        block.valid = true;
        block.dirty = false;
    }

    fn read_fully_associative(&mut self, read_type: ReadType, address: MemoryAddress) {
        let block_index = self.get_block_index_fully_associative(address);
        let tag = self.get_tag(address);

        match read_type {
            ReadType::Instruction => self.logger.log_instruction_reference(),
            ReadType::Data => self.logger.log_data_reference(),
        }

        dbg!(block_index);
        if let Some(idx) = block_index {
            // Hit
            dbg!("Hit");
            self.lru_finder.log_use(idx);
            return;
        }

        // Miss
        dbg!("Miss");
        match read_type {
            ReadType::Instruction => self.logger.log_instruction_miss(),
            ReadType::Data => self.logger.log_data_miss(),
        }

        self.logger.log_memory_read();
        let lru_block = self.lru_finder.get_lru();
        let block = &mut self.blocks[lru_block];

        if let WritePolicy::WriteBack = self.write_policy {
            if block.dirty {
                for _ in 0..self.block_size {
                    self.logger.log_memory_write();
                }
            }
        }

        block.tag = tag;
        block.valid = true;
        block.dirty = false;

        self.lru_finder.log_use(lru_block);
    }

    fn get_tag(&self, address: MemoryAddress) -> u32 {
        address
            >> match self.cache_type {
            CacheType::DirectMap => self.block_mask_size + self.cache_mask_size,
            CacheType::FullyAssociative => self.block_mask_size,
            CacheType::SetAssociative(sets) => self.block_mask_size + sets.ilog2(),
        }
    }

    fn get_set(&self, mut address: MemoryAddress) -> u32 {
        if let CacheType::SetAssociative(sets) = self.cache_type {
            // First clear the block index
            address >>= self.block_mask_size;
            // Clear the tag
            let delete_size = 8 * size_of::<MemoryAddress>() as u32 - sets.ilog2();

            address <<= delete_size;
            address >> delete_size
        } else {
            panic!("Expected a set associative cache type")
        }
    }

    fn get_block_index(&self, address: MemoryAddress) -> Option<usize> {
        match self.cache_type {
            CacheType::DirectMap => Some(self.get_block_index_direct_map(address)),
            CacheType::FullyAssociative => self.get_block_index_fully_associative(address),
            CacheType::SetAssociative(_) => self.get_block_index_set_associative(address),
        }
    }

    fn get_block_index_direct_map(&self, address: MemoryAddress) -> usize {
        let ld = self.tag_mask_size;
        let rs = self.block_mask_size;

        (address << ld >> (ld + rs)) as usize
    }

    fn get_block_index_fully_associative(&self, address: MemoryAddress) -> Option<usize> {
        self.blocks
            .iter()
            .position(|block| block.valid && block.is_match(self.get_tag(address)))
    }

    fn get_block_index_set_associative(&self, address: MemoryAddress) -> Option<usize> {
        let number_of_sets = match self.cache_type {
            CacheType::SetAssociative(sets) => sets,
            _ => panic!("Expected a set associative cache type"),
        };
        let set = self.get_set(address);
        let elements = self.cache_size / number_of_sets as usize;
        let start = set as usize * elements;
        let end = start + elements;

        self.blocks[start..end]
            .iter()
            .position(|block| block.valid && block.is_match(self.get_tag(address)))
    }

    fn write(&mut self, address: MemoryAddress) {
        unimplemented!()
    }
}

struct CacheBuilder {
    block_size: Option<usize>,
    cache_size: Option<usize>,
    write_policy: Option<WritePolicy>,
    on_write_miss: Option<WriteMissPolicy>,
    cache_type: Option<CacheType>,
}

impl CacheBuilder {
    fn new() -> Self {
        Self {
            block_size: None,
            cache_size: None,
            write_policy: None,
            on_write_miss: None,
            cache_type: None,
        }
    }

    fn set_block_size(mut self, block_size: usize) -> Self {
        self.block_size = Some(block_size);
        self
    }

    fn set_cache_size(mut self, cache_size: usize) -> Self {
        self.cache_size = Some(cache_size);
        self
    }

    fn set_write_policy(mut self, write_policy: WritePolicy) -> Self {
        self.write_policy = Some(write_policy);
        self
    }

    fn set_on_write_miss(mut self, on_write_miss: WriteMissPolicy) -> Self {
        self.on_write_miss = Some(on_write_miss);
        self
    }

    fn set_cache_type(mut self, cache_type: CacheType) -> Self {
        self.cache_type = Some(cache_type);
        self
    }

    fn build(self) -> Cache {
        let block_size = self.block_size.unwrap_or(64);
        let cache_size = self.cache_size.unwrap_or(1024);
        let write_policy = self.write_policy.unwrap_or_default();
        let on_write_miss = self.on_write_miss.unwrap_or_default();
        let cache_type = self.cache_type.unwrap_or_default();

        Cache::new(
            block_size,
            cache_size,
            write_policy,
            on_write_miss,
            cache_type,
        )
    }
}

struct Simulator {
    cache: Cache,
}

fn main() {
    let mut cache = CacheBuilder::new()
        .set_block_size(8)
        .set_cache_size(16)
        .set_cache_type(CacheType::SetAssociative(2))
        .set_write_policy(WritePolicy::WriteThrough)
        .set_on_write_miss(WriteMissPolicy::NoWriteAllocate)
        .build();

    cache.read(ReadType::Data, 0);
    cache.read(ReadType::Data, 32);
    cache.read(ReadType::Data, 0);
    cache.read(ReadType::Data, 32);
}
