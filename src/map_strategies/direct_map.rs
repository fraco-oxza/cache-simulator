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

#[cfg(test)]
mod unit_tests {
    use crate::{map_strategies::MapStrategyFactory, MemoryAddress};
    const FACTORY: super::DirectMapFactory = super::DirectMapFactory;

    fn test_mapping(
        block_size: usize,
        cache_size: usize,
        address: MemoryAddress,
        correct_map: MemoryAddress,
    ) {
        let mut dm = FACTORY.generate(block_size, cache_size);
        assert_eq!(dm.map(address, &[]), correct_map);
    }

    fn test_tag(
        block_size: usize,
        cache_size: usize,
        address: MemoryAddress,
        correct_tag: MemoryAddress,
    ) {
        let dm = FACTORY.generate(block_size, cache_size);
        assert_eq!(dm.get_tag(address), correct_tag);
    }

    #[test]
    fn mapping() {
        test_mapping(4, 16, 0b1101_0010_1010, 0b10);
        test_mapping(1, 16, 0b1101_0010_1010, 0b1010);
        test_mapping(1, 1, 0b10_1010_1011_1010, 0b0);
        test_mapping(16, 1, 0b10_1010_1011_1010, 0b0);
    }

    #[test]
    fn tags() {
        test_tag(4, 16, 0b1101_0010_1010, 0b1101);
        test_tag(1, 16, 0b1101_0010_1010, 0b110100);
        test_tag(1, 1, 0b10_1010_1011_1010, 0b101010101110);
        test_tag(16, 1, 0b10_1010_1011_1010, 0b10101010);
    }
}
