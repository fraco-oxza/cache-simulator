use crate::cache::AccessType::{Read, Write};
use crate::cache::ValueType::{Data, Instruction};
use crate::cache::{Cache, MemoryAddress};
use crate::cli_parser::ParsedArgs;
use crate::logger::Logger;
use std::error::Error;
use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader};

pub struct TraceSimulator {
    cache: Cache,
    file_reader: BufReader<File>,
}

impl TraceSimulator {
    pub fn new(args: ParsedArgs) -> io::Result<TraceSimulator> {
        let cache = Cache::new(
            args.block_size,
            args.cache_size,
            args.map_strategy_factory,
            args.write_policy,
            args.write_miss_policy,
        );

        let file = File::open(args.file_path)?;
        let file_reader = BufReader::new(file);
        Ok(TraceSimulator { cache, file_reader })
    }

    pub fn run(mut self) -> Result<Logger, Box<dyn Error>> {
        for line in self.file_reader.lines() {
            let line = line?;

            let mut splited = line.split_whitespace();

            let instruction_number: u8 = splited.next().unwrap().parse()?;
            let address: MemoryAddress =
                MemoryAddress::from_str_radix(&splited.next().unwrap(), 16)?;

            let instruction = match instruction_number {
                0 => Read(Data),
                1 => Write,
                2 => Read(Instruction),
                _ => panic!("Invalid instruction number: {}", instruction_number),
            };

            self.cache.access(instruction, address);
        }

        Ok(self.cache.log)
    }
}
