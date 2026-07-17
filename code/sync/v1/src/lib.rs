// ANCHOR: code
pub struct Counter {
    value: u32,
}

impl Counter {
    pub fn new() -> Counter {
        Counter { value: 0 }
    }

    pub fn inc(&mut self) {
        self.value += 1;
    }

    pub fn value(&self) -> u32 {
        self.value
    }
}
// ANCHOR_END: code

impl Default for Counter {
    fn default() -> Counter {
        Counter::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ANCHOR: helper
    #[track_caller]
    fn assert_counter(got: &Counter, want: u32) {
        assert_eq!(got.value(), want);
    }
    // ANCHOR_END: helper

    // ANCHOR: test
    #[test]
    fn incrementing_the_counter_3_times_leaves_it_at_3() {
        let mut counter = Counter::new();
        counter.inc();
        counter.inc();
        counter.inc();

        assert_counter(&counter, 3);
    }
    // ANCHOR_END: test
}
