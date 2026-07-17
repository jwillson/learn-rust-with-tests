use std::sync::Mutex;

// ANCHOR: code
pub struct Counter {
    value: Mutex<u32>,
}

impl Counter {
    pub fn new() -> Counter {
        Counter {
            value: Mutex::new(0),
        }
    }

    pub fn inc(&self) {
        *self.value.lock().unwrap() += 1;
    }

    pub fn value(&self) -> u32 {
        *self.value.lock().unwrap()
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
    use std::sync::Arc;

    #[track_caller]
    fn assert_counter(got: &Counter, want: u32) {
        assert_eq!(got.value(), want);
    }

    #[test]
    fn incrementing_the_counter_3_times_leaves_it_at_3() {
        let counter = Counter::new();
        counter.inc();
        counter.inc();
        counter.inc();

        assert_counter(&counter, 3);
    }

    #[test]
    fn it_runs_safely_concurrently() {
        let wanted_count = 1000;
        let counter = Counter::new();

        std::thread::scope(|scope| {
            for _ in 0..wanted_count {
                scope.spawn(|| {
                    counter.inc();
                });
            }
        });

        assert_counter(&counter, wanted_count);
    }

    // ANCHOR: arc_test
    #[test]
    fn counts_from_threads_that_outlive_their_spawner() {
        let wanted_count = 1000;
        let counter = Arc::new(Counter::new());
        let mut handles = Vec::new();

        for _ in 0..wanted_count {
            let counter = Arc::clone(&counter);
            handles.push(std::thread::spawn(move || {
                counter.inc();
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }

        assert_counter(&counter, wanted_count);
    }
    // ANCHOR_END: arc_test
}
