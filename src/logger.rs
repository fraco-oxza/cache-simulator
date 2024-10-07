use crate::common::{
    AccessType::{self, *},
    ValueType::*,
};
use crate::{HIT_DURATION, MISS_DURATION, WORD_SIZE};
use std::fmt::{Display, Formatter};
use std::ops::Add;
use std::time::Duration;

#[derive(Default, Clone)]
pub struct Logger {
    instruction_references: u128,
    data_references: u128,
    instruction_misses: u128,
    data_misses: u128,
    memory_reads: u128,
    memory_writes: u128,
    running_time: Duration,
}

impl Logger {
    pub fn reference(&mut self, access_type: &AccessType) {
        match access_type {
            Read(value_type) => {
                match value_type {
                    Instruction => self.instruction_references += 1,
                    Data => self.data_references += 1,
                }

                self.running_time += HIT_DURATION;
            }
            Write => self.data_references += 1,
        }
    }

    pub fn miss(&mut self, access_type: &AccessType) {
        match access_type {
            Read(value_type) => {
                match value_type {
                    Instruction => self.instruction_misses += 1,
                    Data => self.data_misses += 1,
                }

                self.running_time += MISS_DURATION * WORD_SIZE as u32;
            }
            Write => self.data_misses += 1,
        }
    }

    pub fn memory_write(&mut self, words: u128) {
        self.memory_writes += words;
    }

    pub fn memory_read(&mut self, words: u128) {
        self.memory_reads += words;
    }
}

impl Display for Logger {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "┌──────────────────────────┬────────────────┐")?;
        writeln!(f, "│ Metric                   │ Value          │")?;
        writeln!(f, "├──────────────────────────┼────────────────┤")?;
        writeln!(
            f,
            "│ Instruction References   │ {:<14} │",
            self.instruction_references
        )?;
        writeln!(
            f,
            "│ Data References          │ {:<14} │",
            self.data_references
        )?;
        writeln!(
            f,
            "│ Instruction Misses       │ {:<14} │",
            self.instruction_misses
        )?;
        writeln!(f, "│ Data Misses              │ {:<14} │", self.data_misses)?;
        writeln!(
            f,
            "│ Memory Read Words        │ {:<14} │",
            self.memory_reads
        )?;
        writeln!(
            f,
            "│ Memory Write Words       │ {:<14} │",
            self.memory_writes
        )?;
        writeln!(
            f,
            "│ Running Time             │ {:<14?} │",
            self.running_time
        )?;
        writeln!(f, "└──────────────────────────┴────────────────┘")
    }
}

impl Add for Logger {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self {
            instruction_references: self.instruction_references + other.instruction_references,
            data_references: self.data_references + other.data_references,
            instruction_misses: self.instruction_misses + other.instruction_misses,
            data_misses: self.data_misses + other.data_misses,
            memory_reads: self.memory_reads + other.memory_reads,
            memory_writes: self.memory_writes + other.memory_writes,
            running_time: self.running_time + other.running_time,
        }
    }
}
