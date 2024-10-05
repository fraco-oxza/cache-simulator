use crate::cache::{CacheBlock, MapStrategy, MapStrategyFactory, MemoryAddress};
use crate::WORD_SIZE;

pub struct DirectMapFactory;

impl MapStrategyFactory for DirectMapFactory {
    fn generate(&self, block_size: usize, cache_size: usize) -> Box<dyn MapStrategy> {
        let block_mask_size = (block_size.ilog2() + WORD_SIZE.ilog2()) as usize;
        let cache_mask_size = cache_size.ilog2() as usize;
        let tag_mask_size = MemoryAddress::BITS as usize - block_mask_size - cache_mask_size;

        let map_strategy = DirectMap {
            block_mask_size,
            cache_mask_size,
            tag_mask_size,
        };

        Box::new(map_strategy)
    }
}

pub struct DirectMap {
    tag_mask_size: usize,
    block_mask_size: usize,
    cache_mask_size: usize,
}

impl MapStrategy for DirectMap {
    fn map(&mut self, address: MemoryAddress, _blocks: &[CacheBlock]) -> MemoryAddress {
        let ld = self.tag_mask_size;
        let rs = self.block_mask_size;

        address << ld >> (ld + rs)
    }

    fn get_tag(&self, address: MemoryAddress) -> MemoryAddress {
        address >> (self.block_mask_size + self.cache_mask_size)
    }
}
