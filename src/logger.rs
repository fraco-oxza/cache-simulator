use crate::cache::{AccessType::{self, *}, ValueType::*};
use crate::{HIT_DURATION, MISS_DURATION, WORD_SIZE};
use std::time::Duration;

#[derive(Default)]
pub struct Logger {
    instruction_references: u128,
    data_references: u128,
    instruction_hits: u128,
    data_hits: u128,
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
            Write => {
                self.instruction_references += 1
            }
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
            Write => {
                self.instruction_misses += 1
            }
        }
    }

    pub fn memory_write(&mut self, words: u128) {
        self.memory_writes += words;
    }

    pub fn memory_read(&mut self, words: u128) {
        self.memory_reads += words;
    }
}
