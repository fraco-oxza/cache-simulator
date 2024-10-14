use cache_simulator::cache::{WriteMissPolicy, WritePolicy};
use cache_simulator::logger::Logger;
use cache_simulator::map_strategies::fully_associative::FullyAssociativeFactory;
use cache_simulator::{cli_parser::ParsedArgs, trace_simulator::TraceSimulator, WORD_SIZE};
use std::error::Error;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy)]
struct LockedParams {
    block_size: Option<usize>,
    cache_size: Option<usize>,
    write_policy: Option<WritePolicy>,
    write_miss_policy: Option<WriteMissPolicy>,
    split_i_d: Option<bool>,
}

impl LockedParams {
    fn new() -> Self {
        Self {
            block_size: None,
            cache_size: None,
            write_policy: None,
            write_miss_policy: None,
            split_i_d: None,
        }
    }

    fn from_args(args: &[String]) -> Result<Self, Box<dyn Error>> {
        let mut locked = Self::new();
        let mut i = 0;
        while i < args.len() {
            match args[i].as_str() {
                "-bs" => {
                    i += 1;
                    locked.block_size = Some(args[i].parse()?);
                }
                "-cs" => {
                    i += 1;
                    locked.cache_size = Some(args[i].parse()?);
                }
                "-wp" => {
                    i += 1;
                    locked.write_policy = Some(match args[i].to_lowercase().as_str() {
                        "writethrough" => WritePolicy::WriteThrough,
                        "writeback" => WritePolicy::WriteBack,
                        _ => return Err("Invalid write policy".into()),
                    });
                }
                "--wmp" => {
                    i += 1;
                    locked.write_miss_policy = Some(match args[i].to_lowercase().as_str() {
                        "writeallocate" => WriteMissPolicy::WriteAllocate,
                        "nowriteallocate" => WriteMissPolicy::NoWriteAllocate,
                        _ => return Err("Invalid write miss policy".into()),
                    });
                }
                "-split" => {
                    i += 1;
                    locked.split_i_d = Some(args[i].parse()?);
                }
                _ => {}
            }
            i += 1;
        }
        Ok(locked)
    }
}

#[derive(Debug, Clone, Default)]
struct MetricStats {
    max_miss_ratio: f64,
    max_words: f64,
    max_time: f64,
}

impl MetricStats {
    fn update(&mut self, log: &Logger) {
        let miss_ratio = if log.instruction_references + log.data_references > 0 {
            (log.instruction_misses + log.data_misses) as f64 
            / (log.instruction_references + log.data_references) as f64
        } else {
            0.0
        };
        self.max_miss_ratio = self.max_miss_ratio.max(miss_ratio);

        let words = (log.memory_reads + log.memory_writes) as f64;
        self.max_words = self.max_words.max(words);

        let time = log.running_time.as_secs_f64();
        self.max_time = self.max_time.max(time);
    }
}

#[derive(Debug, Clone, Copy)]
enum Metric {
    InstructionMisses,
    DataMisses,
    TotalMisses,
    MemoryReads,
    MemoryWrites,
    MissRatio,
    ExecutionTime,
    CombinedPerformance,
}

impl Metric {
    fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "instruction_misses" => Some(Self::InstructionMisses),
            "data_misses" => Some(Self::DataMisses),
            "total_misses" => Some(Self::TotalMisses),
            "memory_reads" => Some(Self::MemoryReads),
            "memory_writes" => Some(Self::MemoryWrites),
            "miss_ratio" => Some(Self::MissRatio),
            "execution_time" => Some(Self::ExecutionTime),
            "combined_performance" => Some(Self::CombinedPerformance),
            _ => None,
        }
    }

    fn get_value(&self, log: &Logger, stats: &MetricStats) -> f64 {
        match self {
            Metric::InstructionMisses => log.instruction_misses as f64,
            Metric::DataMisses => log.data_misses as f64,
            Metric::TotalMisses => (log.instruction_misses + log.data_misses) as f64,
            Metric::MemoryReads => log.memory_reads as f64,
            Metric::MemoryWrites => log.memory_writes as f64,
            Metric::MissRatio => {
                let total_refs = log.instruction_references + log.data_references;
                let total_misses = log.instruction_misses + log.data_misses;
                if total_refs > 0 {
                    total_misses as f64 / total_refs as f64
                } else {
                    0.0
                }
            }
            Metric::ExecutionTime => log.running_time.as_secs_f64(),
            Metric::CombinedPerformance => Self::calculate_combined_performance(log, stats),
        }
    }

    fn calculate_combined_performance(log: &Logger, stats: &MetricStats) -> f64 {
        let miss_ratio = if log.instruction_references + log.data_references > 0 {
            (log.instruction_misses + log.data_misses) as f64 
            / (log.instruction_references + log.data_references) as f64
        } else {
            0.0
        };

        let words = (log.memory_reads + log.memory_writes) as f64;
        let time = log.running_time.as_secs_f64();

        let normalized_miss_ratio = if stats.max_miss_ratio > 0.0 { miss_ratio / stats.max_miss_ratio } else { 0.0 };
        let normalized_words = if stats.max_words > 0.0 { words / stats.max_words } else { 0.0 };
        let normalized_time = if stats.max_time > 0.0 { time / stats.max_time } else { 0.0 };

        normalized_miss_ratio + normalized_words + normalized_time
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 3 {
        print_usage(&args[0]);
        return Ok(());
    }

    let metric = Metric::from_str(&args[1]).ok_or("Invalid metric specified")?;
    let file_path = PathBuf::from(&args[2]);

    let locked_params = LockedParams::from_args(&args[3..])?;

    let byte_sizes: Vec<usize> = vec![
        4, 8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096, 8192, 16384, 32768, 65536,
    ];

    println!("Using metric: {:?}", metric);
    println!("Locked parameters: {:?}", locked_params);

    let mut metric_stats = MetricStats::default();

    for size in byte_sizes {
        let (best_args, best_metric_value, best_logs) = find_best_configuration(
            size, &locked_params, &file_path, &metric, &mut metric_stats
        )?;

        if let (Some(args), Some(logs)) = (best_args, best_logs) {
            println!("Cache Total size of {}", size);
            show_results(args, best_metric_value, logs, size, metric, &metric_stats);
        }
    }

    Ok(())
}

fn find_best_configuration(
    size: usize,
    locked_params: &LockedParams,
    file_path: &PathBuf,
    metric: &Metric,
    metric_stats: &mut MetricStats,
) -> Result<(Option<ParsedArgs>, f64, Option<Logger>), Box<dyn Error>> {
    let mut best_args = None;
    let mut best_metric_value = f64::INFINITY;
    let mut best_logs = None;

    let cache_sizes = if let Some(cs) = locked_params.cache_size {
        vec![cs]
    } else {
        (0..=(size.ilog2())).map(|x| 1 << x).collect()
    };

    for cache_size in cache_sizes {
        let block_sizes = if let Some(bs) = locked_params.block_size {
            vec![bs]
        } else {
            let max_block_size = (size / cache_size) / WORD_SIZE;
            if max_block_size == 0 {
                continue;
            }
            vec![max_block_size]
        };

        for block_size in block_sizes {
            if block_size == 0 {
                continue;
            }

            let write_miss_policies = if let Some(wmp) = locked_params.write_miss_policy {
                vec![wmp]
            } else {
                vec![WriteMissPolicy::WriteAllocate, WriteMissPolicy::NoWriteAllocate]
            };

            let write_policies = if let Some(wp) = locked_params.write_policy {
                vec![wp]
            } else {
                vec![WritePolicy::WriteThrough, WritePolicy::WriteBack]
            };

            let split_i_d_options = if let Some(sid) = locked_params.split_i_d {
                vec![sid]
            } else {
                vec![false, true]
            };

            for write_miss_policy in write_miss_policies {
                for write_policy in write_policies.iter() {
                    for split_i_d in split_i_d_options.iter() {
                        if *split_i_d && cache_size < 2 {
                            continue;
                        }

                        let args = ParsedArgs {
                            block_size,
                            cache_size,
                            file_path: file_path.clone(),
                            map_strategy_factory: Box::new(FullyAssociativeFactory),
                            split_i_d: *split_i_d,
                            write_miss_policy,
                            write_policy: *write_policy,
                        };

                        let trace_sim = TraceSimulator::new(args)?;
                        let logs = trace_sim.run()?;
                        metric_stats.update(&logs);
                        let current_metric = metric.get_value(&logs, metric_stats);

                        if current_metric < best_metric_value {
                            best_metric_value = current_metric;
                            best_logs = Some(logs);
                            best_args = Some(ParsedArgs {
                                block_size,
                                cache_size,
                                file_path: file_path.clone(),
                                map_strategy_factory: Box::new(FullyAssociativeFactory),
                                split_i_d: *split_i_d,
                                write_miss_policy,
                                write_policy: *write_policy,
                            });
                        }
                    }
                }
            }
        }
    }

    Ok((best_args, best_metric_value, best_logs))
}

fn show_results(best_args: ParsedArgs, best_metric: f64, logs: Logger, total_size: usize, metric: Metric, stats: &MetricStats) {
    println!("╔═══════════════════════════════════════════════════════╗");
    println!("║           Best Configuration for {:8} Bytes       ║", total_size);
    println!("╠═══════════════════╤═══════════════════════════════════╣");
    println!("║ Cache Parameters  │ Value                             ║");
    println!("╟───────────────────┼───────────────────────────────────╢");
    println!("║ File              │ {:<33} ║", best_args.file_path.display());
    println!("║ Cache Size        │ {:<33} ║", best_args.cache_size);
    println!("║ Block Size        │ {:<33} ║", best_args.block_size);
    println!("║ Write Policy      │ {:<33} ║", format!("{:?}", best_args.write_policy));
    println!("║ Write Miss Policy │ {:<33} ║", format!("{:?}", best_args.write_miss_policy));
    println!("║ Split I/D         │ {:<33} ║", best_args.split_i_d);
    println!("║ Metric Used       │ {:<33} ║", format!("{:?}", metric));
    println!("║ Best Metric Value │ {:<33.6} ║", best_metric);
    println!("╟───────────────────┼───────────────────────────────────╢");
    println!("║ Results           │ Count                             ║");
    println!("╟───────────────────┼───────────────────────────────────╢");
    println!("║ Instr Refs        │ {:<33} ║", logs.instruction_references);
    println!("║ Data Refs         │ {:<33} ║", logs.data_references);
    println!("║ Instr Misses      │ {:<33} ║", logs.instruction_misses);
    println!("║ Data Misses       │ {:<33} ║", logs.data_misses);
    println!("║ Memory Reads      │ {:<33} ║", logs.memory_reads);
    println!("║ Memory Writes     │ {:<33} ║", logs.memory_writes);
    println!("║ Runtime           │ {:<33?} ║", logs.running_time);
    println!("╚═══════════════════╧═══════════════════════════════════╝");

    if let Metric::CombinedPerformance = metric {
        println!("\nNormalization factors for Combined Performance:");
        println!("Max Miss Ratio: {:.6}", stats.max_miss_ratio);
        println!("Max Words (Read + Written): {:.0}", stats.max_words);
        println!("Max Execution Time: {:.6} seconds", stats.max_time);
    }
}

fn print_usage(program_name: &str) {
    eprintln!("Usage: {} <metric> <trace_file> [options]", program_name);
    eprintln!("Available metrics:");
    eprintln!("  instruction_misses");
    eprintln!("  data_misses");
    eprintln!("  total_misses");
    eprintln!("  memory_reads");
    eprintln!("  memory_writes");
    eprintln!("  miss_ratio");
    eprintln!("  execution_time");
    eprintln!("  combined_performance");
    eprintln!("Options:");
    eprintln!("  --block-size <size>          Lock block size");
    eprintln!("  --cache-size <size>          Lock cache size");
    eprintln!("  --write-policy <policy>      Lock write policy (writethrough/writeback)");
    eprintln!("  --write-miss-policy <policy> Lock write miss policy (writeallocate/nowriteallocate)");
    eprintln!("  --split-i-d <bool>           Lock split I/D (true/false)");
}
