# Cache Simulator: Compilation and Execution Guide

This document explains how to compile and run the Rust-based cache simulator, and also provides information on using precompiled executables to skip the compilation process.

## Prerequisites

Before proceeding, ensure you have the following installed on your system:

- **Rust**: You can install it using [rustup](https://rustup.rs/).
- **Cargo**: This comes bundled with Rust.

## Compilation Steps

If you want to compile the project yourself, follow these steps:

1. **Clone the Repository**  
   First, clone the repository to your local machine:

   ```bash
   git clone <repository_url>
   cd <repository_directory>
   ```

2. **Build the Project**  
   Use `cargo` to build the project:

   ```bash
   cargo build --release
   ```

   This will create an optimized version of the binaries in the `target/release` directory.

3. **Run the Executable**  
   After the project is compiled, you can run the executable by specifying the necessary arguments:

   ```bash
   ./target/release/cache-simulator -bs <block_size> -cs <cache_size> [options] <file_path>
   ```

   For example:

   ```bash
   ./target/release/cache-simulator -bs 64 -cs 1024 -wt -fa tracefile.txt
   ```

## Using Precompiled Executables

To avoid compiling the project, you can download precompiled executables that are built with **musl** for better compatibility across systems.

1. **Download the Executables**  
   Navigate to the **Releases** section of the repository and download the precompiled binaries.

2. **Run the Executable**  
   Once downloaded, you can directly run the executable:

   ```bash
   ./cache-simulator -bs <block_size> -cs <cache_size> [options] <file_path>
   ```

   The options include:

   - `-bs`: Set the block size (e.g., `-bs 64`)
   - `-cs`: Set the cache size (e.g., `-cs 1024`)
   - `-wt`: Use write-through policy (default is write-back)
   - `-fa`: Use fully associative cache mapping strategy
   - `-sa <num>`: Use set-associative mapping strategy with the specified number of sets
   - `-split`: Enable split instruction and data caches
   - `-wna`: Use no-write-allocate for write misses (default is write-allocate)

   For example:

   ```bash
   ./cache-simulator -bs 64 -cs 1024 -wt -fa tracefile.txt
   ```

## Options Explained

- `-bs`: Block size (in bytes)
- `-cs`: Cache size (in bytes)
- `-wt`: Use write-through policy (default is write-back)
- `-fa`: Fully associative cache
- `-sa <num>`: Set-associative cache with `<num>` sets
- `-split`: Separate instruction and data caches
- `-wna`: No write allocate (default is write allocate)

Make sure to provide the required parameters and a valid trace file path when
executing the simulator.

## grid-search

This program simulates a fully associative cache with various configurable parameters and evaluates its performance based on different metrics using trace files.

### Usage

To run the simulator, compile the code and then execute the binary with the following command-line arguments:

```
grid-search <metric> <trace_file> [options]
```

#### Arguments

- **`<metric>`:** The metric to optimize the cache for. Available options are:

  - `instruction_misses`
  - `data_misses`
  - `total_misses`
  - `memory_reads`
  - `memory_writes`
  - `miss_ratio`
  - `execution_time`
  - `combined_performance` (normalized sum of miss ratio, memory operations, and execution time)

- **`<trace_file>`:** Path to the trace file containing memory access instructions.

- **`[options]`:** Optional parameters to lock specific cache configurations. These include:
  - `-bs <size>`: Lock the block size to the specified value.
  - `-cs <size>`: Lock the cache size to the specified value.
  - `-wp <policy>`: Lock the write policy. Options are `writethrough` and `writeback`.
  - `--wmp <policy>`: Lock the write miss policy. Options are `writeallocate` and `nowriteallocate`.
  - `-split <bool>`: Lock whether to split the cache for instructions and data. Options are `true` and `false`.

#### Example

To find the best cache configuration for minimizing the miss ratio using the `trace.txt` file, with a locked block size of 64 bytes and a write-back policy, you would run:

```
grid-search miss_ratio trace.txt -bs 64 -wp writeback
```

### How it Works

The simulator iterates through a range of cache sizes (powers of 2) and
explores different block sizes, write policies, write miss policies, and the
option to split the cache for instructions and data.

For each configuration, it simulates the cache behavior using the provided
trace file and calculates the chosen metric. The configuration that yields the
best (lowest) value for the metric is then reported as the optimal
configuration for that specific cache size.

The simulator uses a fully associative mapping strategy for the cache.

### Output

The program outputs the following information:

- The chosen metric and any locked parameters.
- For each tested total cache size:
  - The best-found configuration parameters (block size, write policy, write
    miss policy, split I/D).
  - The value of the chosen metric for the best configuration.
  - Detailed simulation results, including the number of instruction and data
    references, misses, memory reads and writes, and execution time.
  - If the combined performance metric is used, it also displays the
    normalization factors used for calculating the combined score.
