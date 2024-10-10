use crate::cache_block::CacheBlock;
use crate::map_strategies::{MapStrategy, MapStrategyFactory};
use crate::MemoryAddress;
use crate::WORD_SIZE;

#[derive(Debug)]
pub struct DirectMapFactory;

impl MapStrategyFactory for DirectMapFactory {
    fn generate(&self, block_size: usize, cache_size: usize) -> Box<dyn MapStrategy> {
        let block_mask_size = (block_size.ilog2() + WORD_SIZE.ilog2()) as usize;
        let cache_mask_size = cache_size.ilog2() as usize;

        let map_strategy = DirectMap {
            block_mask_size,
            cache_mask_size,
        };

        Box::new(map_strategy)
    }
}

pub struct DirectMap {
    block_mask_size: usize,
    cache_mask_size: usize,
}

impl MapStrategy for DirectMap {
    fn map(&mut self, mut address: MemoryAddress, _blocks: &[CacheBlock]) -> MemoryAddress {
        address %= 1 << (self.block_mask_size + self.cache_mask_size);
        address >> self.block_mask_size
    }

    fn get_tag(&self, address: MemoryAddress) -> MemoryAddress {
        address >> (self.block_mask_size + self.cache_mask_size)
    }
}
