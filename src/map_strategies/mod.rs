use crate::common::{CacheBlock, MemoryAddress};

pub mod set_associative;
pub mod direct_map;
pub mod fully_associative;

/// A factory trait for creating mapping strategies.
pub trait MapStrategyFactory {
    /// Generates a new mapping strategy instance.
    fn generate(&self, block_size: usize, cache_size: usize) -> Box<dyn MapStrategy>;
}

/// Defines the behavior of a common mapping strategy.
pub trait MapStrategy {
    /// Maps a memory address to a common block index.
    ///
    /// This function returns the index of the common block where the data
    /// corresponding to the given `address` **should** be stored according
    /// to the mapping strategy.
    ///
    /// This function only indicates the potential location of
    /// the data. It does not guarantee that the data is actually present in
    /// that block. The caller is responsible for verifying the presence of
    /// the data by comparing the block's tag with the tag returned by
    /// the `get_tag` function.
    fn map(&mut self, address: MemoryAddress, blocks: &[CacheBlock]) -> MemoryAddress;

    /// Extracts the tag from a given memory address.
    ///
    /// The tag is a portion of the memory address used to identify
    /// whether a specific common block contains the desired data.
    fn get_tag(&self, address: MemoryAddress) -> MemoryAddress;
}