use cache_simulator::cache::{WriteMissPolicy, WritePolicy};
use cache_simulator::logger::Logger;
use cache_simulator::map_strategies::fully_associative::FullyAssociativeFactory;
use cache_simulator::{cli_parser::ParsedArgs, trace_simulator::TraceSimulator, WORD_SIZE};
use std::error::Error;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy)]
enum Metric {
    InstructionMisses,
    DataMisses,
    TotalMisses,
    MemoryReads,
    MemoryWrites,
    MissRatio,
    ExecutionTime,
}

impl Metric {
    fn get_value(&self, log: &Logger) -> f64 {
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
        }
    }

    fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "instruction_misses" => Some(Self::InstructionMisses),
            "data_misses" => Some(Self::DataMisses),
            "total_misses" => Some(Self::TotalMisses),
            "memory_reads" => Some(Self::MemoryReads),
            "memory_writes" => Some(Self::MemoryWrites),
            "miss_ratio" => Some(Self::MissRatio),
            "execution_time" => Some(Self::ExecutionTime),
            _ => None,
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = std::env::args();
    let program = args.next().unwrap();

    if args.len() < 2 {
        eprintln!("Usage: {} <metric> <trace_file>", program);
        eprintln!("Available metrics: instruction_misses, data_misses, total_misses, memory_reads, memory_writes, miss_ratio, execution_time");
        return Ok(());
    }

    let metric_str = args.next().unwrap();
    let metric = Metric::from_str(&metric_str).ok_or("Invalid metric specified")?;

    let raw_file_path = args.next().unwrap();
    let file_path = PathBuf::from(raw_file_path);
    let byte_sizes: Vec<usize> = vec![
        4, 8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096, 8192, 16384, 32768, 65536,
    ];

    println!("Using metric: {:?}", metric);

    for size in byte_sizes {
        let mut best_args = None;
        let mut best_metric_value = f64::INFINITY;
        let mut best_logs = None;

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
                        let current_metric = metric.get_value(&logs);

                        if current_metric < best_metric_value {
                            best_metric_value = current_metric;
                            best_logs = Some(logs);
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
        show_results(
            best_args.unwrap(),
            best_metric_value,
            best_logs.unwrap(),
            size,
            metric,
        );
    }

    Ok(())
}

#[rustfmt::skip]
fn show_results<T>(best_args: ParsedArgs, best_metric: T, logs: Logger, total_size: usize, metric: Metric)
where
    T: std::fmt::Debug,
{
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
    println!("║ Best Metric Value │ {:<33?} ║", best_metric);
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
}
