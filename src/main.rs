use cache_simulator::cli_parser::ParsedArgs;
use cache_simulator::trace_simulator::TraceSimulator;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let args = std::env::args();
    let parsed_args = ParsedArgs::parse(args)?;

    let simulator = TraceSimulator::new(parsed_args)?;
    let results = simulator.run()?;

    println!("{}", results);

    Ok(())
}
