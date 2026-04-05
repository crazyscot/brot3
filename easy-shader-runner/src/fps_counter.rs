use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};

use num_traits::AsPrimitive;

pub(crate) struct FpsCounter {
    frames: VecDeque<Instant>,
}

impl FpsCounter {
    pub(crate) fn new() -> Self {
        Self {
            frames: VecDeque::default(),
        }
    }

    pub(crate) fn tick(&mut self) -> u32 {
        let one_second_from_now = Instant::now() + Duration::from_secs(1);
        self.frames.push_back(one_second_from_now);
        let now = one_second_from_now
            .checked_sub(Duration::from_secs(1))
            .unwrap();

        while self.frames.front().is_some_and(|t| t < &now) {
            let _ = self.frames.pop_front();
        }

        self.frames.len().as_()
    }
}
