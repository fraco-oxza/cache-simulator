use crate::cache_block::CacheBlock;
use crate::lru::Lru;
use crate::map_strategies::{MapStrategy, MapStrategyFactory};
use crate::MemoryAddress;

use crate::WORD_SIZE;

#[derive(Debug)]
pub struct FullyAssociativeFactory;

impl MapStrategyFactory for FullyAssociativeFactory {
    fn generate(&self, block_size: usize, cache_size: usize) -> Box<dyn MapStrategy> {
        let block_mask_size = (block_size.ilog2() + WORD_SIZE.ilog2()) as usize;
        let replacement_policy = Lru::new(cache_size);

        let map_strategy = FullyAssociative {
            block_mask_size,
            replacement_policy,
        };

        Box::new(map_strategy)
    }
}

pub struct FullyAssociative {
    block_mask_size: usize,
    replacement_policy: Lru,
}

impl MapStrategy for FullyAssociative {
    fn map(&mut self, address: MemoryAddress, blocks: &[CacheBlock]) -> MemoryAddress {
        let tag = self.get_tag(address);
        let possible_block = blocks
            .iter()
            .position(|block| block.valid && block.is_match(tag));
        if let Some(idx) = possible_block {
            self.replacement_policy.mark_use(idx);
            return idx as MemoryAddress;
        }

        let result = self.replacement_policy.get_lru();
        self.replacement_policy.mark_use(result);

        result as MemoryAddress
    }

    fn get_tag(&self, address: MemoryAddress) -> MemoryAddress {
        address >> self.block_mask_size
    }
}
