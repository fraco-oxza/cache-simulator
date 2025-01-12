use crate::cache::{WriteMissPolicy, WritePolicy};
use crate::map_strategies::direct_map::DirectMapFactory;
use crate::map_strategies::fully_associative::FullyAssociativeFactory;
use crate::map_strategies::set_associative::SetAssociativeFactory;
use crate::map_strategies::MapStrategyFactory;
use crate::{DEFAULT_BLOCK_SIZE, DEFAULT_CACHE_SIZE};
use std::env::Args;
use std::error::Error;
use std::fmt::Display;
use std::path::PathBuf;

#[derive(Debug)]
pub struct ParsedArgs {
    pub block_size: usize,
    pub cache_size: usize,
    pub write_policy: WritePolicy,
    pub write_miss_policy: WriteMissPolicy,
    pub map_strategy_factory: Box<dyn MapStrategyFactory>,
    pub split_i_d: bool,
    pub file_path: PathBuf,
}

impl ParsedArgs {
    pub fn parse(params: Args) -> Result<ParsedArgs, Box<dyn Error>> {
        let params: Vec<String> = params.collect();

        let block_size = params
            .iter()
            .position(|a| a == "-bs")
            .map(|idx| params[idx + 1].clone())
            .map(|raw_bs| raw_bs.parse::<usize>())
            .transpose()?
            .unwrap_or(DEFAULT_BLOCK_SIZE);

        let cache_size = params
            .iter()
            .position(|a| a == "-cs")
            .map(|idx| params[idx + 1].clone())
            .map(|raw_bs| raw_bs.parse::<usize>())
            .transpose()?
            .unwrap_or(DEFAULT_CACHE_SIZE);

        let write_policy = if params.iter().any(|a| a == "-wt") {
            WritePolicy::WriteThrough
        } else {
            WritePolicy::WriteBack
        };

        let split_i_d = params.iter().any(|x| x == "-split");

        let write_miss_policy = if params.iter().any(|x| x == "-wna") {
            WriteMissPolicy::NoWriteAllocate
        } else {
            WriteMissPolicy::WriteAllocate
        };

        let map_strategy_factory: Box<dyn MapStrategyFactory> =
            if params.contains(&"-fa".to_owned()) {
                Box::new(FullyAssociativeFactory)
            } else if let Some(idx) = params.iter().position(|x| x == "-sa") {
                Box::new(SetAssociativeFactory {
                    sets: params[idx + 1].parse()?,
                })
            } else {
                Box::new(DirectMapFactory)
            };

        let file_path = PathBuf::from(params.last().unwrap().clone());

        Ok(Self {
            block_size,
            cache_size,
            write_policy,
            write_miss_policy,
            split_i_d,
            map_strategy_factory,
            file_path,
        })
    }
}

impl Display for ParsedArgs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Block Size: {}", self.block_size)?;
        writeln!(f, "Cache Size: {}", self.cache_size)?;
        writeln!(f, "Write Policy: {:?}", self.write_policy)?;
        writeln!(f, "Write Miss Policy: {:?}", self.write_miss_policy)?;
        writeln!(f, "Map Strategy: {:?}", self.map_strategy_factory)?;
        writeln!(f, "Split I/D: {}", self.split_i_d)?;
        writeln!(f, "File Path: {:?}", self.file_path)
    }
}
