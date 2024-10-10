use std::time::Duration;

pub mod cache;
pub mod cache_block;
pub mod cli_parser;
pub mod logger;
pub mod lru;
pub mod map_strategies;
pub mod trace_simulator;

pub const HIT_DURATION: Duration = Duration::from_nanos(5);
pub const MISS_DURATION: Duration = Duration::from_nanos(100);
pub const WORD_SIZE: usize = 4;
pub const DEFAULT_BLOCK_SIZE: usize = 64;
pub const DEFAULT_CACHE_SIZE: usize = 256;
pub type MemoryAddress = u32;
