use std::error::Error;
use std::path::PathBuf;

use cache_simulator::cache::{WriteMissPolicy, WritePolicy};
use cache_simulator::logger::Logger;
use cache_simulator::map_strategies::*;
use cache_simulator::{cli_parser::ParsedArgs, trace_simulator::TraceSimulator, WORD_SIZE};
use fully_associative::FullyAssociativeFactory;

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = std::env::args();
    let byte_sizes: Vec<usize> = vec![512, 1024, 2048, 4096, 8192, 16384, 32768, 65536];
    let raw_file_path = args.nth(1).unwrap();
    let file_path = PathBuf::from(raw_file_path);
    let metric_fn = |log: &Logger| log.get_miss();

    for size in byte_sizes {
        let mut best_args = None;
        let mut best_time = None;

        for cache_size in (0..=(size.ilog2())).map(|x| 1 << x) {
            let block_size = (size / cache_size) / WORD_SIZE;

            if block_size == 0 {
                continue;
            }

            for write_miss_policy in [
                WriteMissPolicy::WriteAllocate,
                WriteMissPolicy::NoWriteAllocate,
            ] {
                for write_policy in [WritePolicy::WriteThrough, WritePolicy::WriteBack] {
                    for split_i_d in [false, true] {
                        if split_i_d && cache_size < 2 {
                            continue;
                        }

                        let args = ParsedArgs {
                            block_size,
                            cache_size,
                            file_path: file_path.clone(),
                            map_strategy_factory: Box::new(FullyAssociativeFactory),
                            split_i_d,
                            write_miss_policy,
                            write_policy,
                        };

                        let trace_sim = TraceSimulator::new(args)?;
                        let logs = trace_sim.run()?;

                        if best_time.is_none() || best_time.unwrap() > metric_fn(&logs) {
                            best_time = Some(metric_fn(&logs));
                            best_args = Some(ParsedArgs {
                                block_size,
                                cache_size,
                                file_path: file_path.clone(),
                                map_strategy_factory: Box::new(FullyAssociativeFactory),
                                split_i_d,
                                write_miss_policy,
                                write_policy,
                            });
                        }
                    }
                }
            }
        }

        println!("Cache Total size of {}", size);
        println!("{}", best_args.unwrap());
        println!("{:?}", best_time.unwrap())
    }

    Ok(())
}
