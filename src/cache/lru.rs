use std::collections::VecDeque;

#[derive(Clone)]
pub struct Lru {
    nums: VecDeque<usize>,
}

impl Lru {
    pub fn new(size: usize) -> Self {
        Self {
            nums: (0..size).collect(),
        }
    }

    pub fn mark_use(&mut self, cache_address: usize) {
        let idx = self
            .nums
            .iter()
            .position(|&num| num == cache_address)
            .unwrap();

        self.nums.remove(idx);
        self.nums.push_back(cache_address);
    }

    pub fn get_lru(&self) -> usize {
        *self.nums.front().unwrap()
    }
}
