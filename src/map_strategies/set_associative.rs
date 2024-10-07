use crate::cache_block::CacheBlock;
use crate::lru::Lru;
use crate::map_strategies::{MapStrategy, MapStrategyFactory};
use crate::MemoryAddress;
use crate::WORD_SIZE;

pub struct SetAssociativeFactory {
    pub sets: usize,
}

impl MapStrategyFactory for SetAssociativeFactory {
    fn generate(&self, block_size: usize, cache_size: usize) -> Box<dyn MapStrategy> {
        let block_mask_size = (block_size.ilog2() + WORD_SIZE.ilog2()) as usize;
        let replacement_policy =
            vec![Lru::new(cache_size / self.sets); self.sets].into_boxed_slice();
        let set_mask_size = self.sets.ilog2() as usize;

        let map_strategy = SetAssociative {
            cache_size,
            block_mask_size,
            set_mask_size,
            replacement_policy,
            sets: self.sets,
        };

        Box::new(map_strategy)
    }
}

pub struct SetAssociative {
    cache_size: usize,
    block_mask_size: usize,
    set_mask_size: usize,
    replacement_policy: Box<[Lru]>,
    sets: usize,
}

impl SetAssociative {
    fn get_set(&self, mut address: MemoryAddress) -> MemoryAddress {
        let mask = (1 << (self.block_mask_size + self.set_mask_size)) - 1;
        address &= mask;
        address >> self.block_mask_size
    }
}

impl MapStrategy for SetAssociative {
    fn map(&mut self, address: MemoryAddress, blocks: &[CacheBlock]) -> MemoryAddress {
        let set = self.get_set(address) as usize;
        let elements = self.cache_size / self.sets;
        let start = set * elements;
        let end = start + elements;
        let tag = self.get_tag(address);

        let possible_block = blocks[start..end]
            .iter()
            .position(|block| block.valid && block.is_match(tag));

        if let Some(idx) = possible_block {
            self.replacement_policy[set].mark_use(idx);
            return (start + idx) as MemoryAddress;
        }

        let result = self.replacement_policy[set].get_lru();
        self.replacement_policy[set].mark_use(result);

        (start + result) as MemoryAddress
    }

    fn get_tag(&self, address: MemoryAddress) -> MemoryAddress {
        address >> self.block_mask_size
    }
}
