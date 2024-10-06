mod cache;
mod cli_parser;
mod common;
mod logger;
mod map_strategies;
mod trace_simulator;

use crate::cli_parser::ParsedArgs;
use crate::trace_simulator::TraceSimulator;
use std::error::Error;
use std::time::Duration;

const HIT_DURATION: Duration = Duration::from_nanos(5);
const MISS_DURATION: Duration = Duration::from_nanos(100);
const WORD_SIZE: usize = 4;
const DEFAULT_BLOCK_SIZE: usize = 64;
const DEFAULT_CACHE_SIZE: usize = 256;

fn main() -> Result<(), Box<dyn Error>> {
    let args = std::env::args();
    let parsed_args = ParsedArgs::parse(args)?;

    let simulator = TraceSimulator::new(parsed_args)?;
    let results = simulator.run()?;

    println!("{}", results);

    Ok(())
}
