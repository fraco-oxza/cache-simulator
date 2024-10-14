use crate::MemoryAddress;

/// Represents a single block within the cache.
#[derive(Clone, Default, PartialOrd, PartialEq, Debug)]
pub struct CacheBlock {
    pub valid: bool,
    pub dirty: bool,
    pub tag: MemoryAddress,
}

impl CacheBlock {
    pub fn is_match(&self, tag: MemoryAddress) -> bool {
        self.tag == tag
    }
}
