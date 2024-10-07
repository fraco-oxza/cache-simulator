use crate::cache::AccessType::{Read, Write};
use crate::cache::Cache;
use crate::cache::ValueType::{Data, Instruction};
use crate::cli_parser::ParsedArgs;
use crate::logger::Logger;
use crate::MemoryAddress;
use std::cell::RefCell;
use std::error::Error;
use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader};
use std::rc::Rc;

pub struct TraceSimulator {
    cache: Cache,
    instructions_cache: Option<Cache>,
    file_reader: BufReader<File>,
    logs: Rc<RefCell<Logger>>,
}

impl TraceSimulator {
    pub fn new(args: ParsedArgs) -> io::Result<TraceSimulator> {
        let logs = Rc::new(RefCell::new(Logger::default()));

        let cache = Cache::new(
            args.block_size / (1 + args.split_i_d as usize),
            args.cache_size,
            &*args.map_strategy_factory,
            args.write_policy,
            args.write_miss_policy,
            Rc::clone(&logs),
        );

        let instructions_cache = if args.split_i_d {
            Some(Cache::new(
                args.block_size / 2,
                args.cache_size,
                &*args.map_strategy_factory,
                args.write_policy,
                args.write_miss_policy,
                Rc::clone(&logs),
            ))
        } else {
            None
        };

        let file = File::open(args.file_path)?;
        let file_reader = BufReader::new(file);
        Ok(TraceSimulator {
            cache,
            file_reader,
            instructions_cache,
            logs,
        })
    }

    pub fn run(mut self) -> Result<Logger, Box<dyn Error>> {
        for line in self.file_reader.lines() {
            let line = line?;

            let mut splited = line.split_whitespace();

            let instruction_number: u8 = splited.next().unwrap().parse()?;
            let address: MemoryAddress =
                MemoryAddress::from_str_radix(splited.next().unwrap(), 16)?;

            let instruction = match instruction_number {
                0 => Read(Data),
                1 => Write,
                2 => Read(Instruction),
                _ => panic!("Invalid instruction number: {}", instruction_number),
            };

            if let Some(ref mut cache_i) = self.instructions_cache {
                if matches!(instruction, Read(Instruction)) {
                    cache_i.access(instruction, address)
                } else {
                    self.cache.access(instruction, address);
                }
            } else {
                self.cache.access(instruction, address);
            }
        }

        Ok(self.logs.take())
    }
}
