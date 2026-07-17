use std::io::BufRead;
use std::sync::Arc;
use std::time::Duration;

pub trait PlayerStore: Send + Sync {
    fn record_win(&self, name: &str);
}

// ANCHOR: alerter
pub trait BlindAlerter: Send + Sync {
    fn schedule_alert_at(&self, at: Duration, amount: u32);
}
// ANCHOR_END: alerter

// ANCHOR: cli
pub struct Cli<R> {
    store: Arc<dyn PlayerStore>,
    alerter: Arc<dyn BlindAlerter>,
    input: R,
}

impl<R: BufRead> Cli<R> {
    pub fn new(store: Arc<dyn PlayerStore>, alerter: Arc<dyn BlindAlerter>, input: R) -> Cli<R> {
        Cli {
            store,
            alerter,
            input,
        }
    }

    pub fn play_poker(&mut self) {
        self.schedule_blind_alerts();

        let mut line = String::new();
        if self.input.read_line(&mut line).unwrap_or(0) == 0 {
            return;
        }

        if let Some(name) = line.trim().strip_suffix(" wins") {
            self.store.record_win(name);
        }
    }

    fn schedule_blind_alerts(&self) {
        let blinds = [100, 200, 300, 400, 500, 600, 800, 1000, 2000, 4000, 8000];
        let mut blind_time = Duration::ZERO;

        for blind in blinds {
            self.alerter.schedule_alert_at(blind_time, blind);
            blind_time += Duration::from_secs(10 * 60);
        }
    }
}
// ANCHOR_END: cli

// ANCHOR: stdout_alerter
pub struct StdoutBlindAlerter;

impl BlindAlerter for StdoutBlindAlerter {
    fn schedule_alert_at(&self, at: Duration, amount: u32) {
        std::thread::spawn(move || {
            std::thread::sleep(at);
            println!("Blind is now {amount}");
        });
    }
}
// ANCHOR_END: stdout_alerter

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    // ANCHOR: spy
    #[derive(Debug, PartialEq)]
    struct ScheduledAlert {
        at: Duration,
        amount: u32,
    }

    #[derive(Default)]
    struct SpyBlindAlerter {
        alerts: Mutex<Vec<ScheduledAlert>>,
    }

    impl BlindAlerter for SpyBlindAlerter {
        fn schedule_alert_at(&self, at: Duration, amount: u32) {
            self.alerts
                .lock()
                .unwrap()
                .push(ScheduledAlert { at, amount });
        }
    }
    // ANCHOR_END: spy

    #[derive(Default)]
    struct StubPlayerStore {
        win_calls: Mutex<Vec<String>>,
    }

    impl PlayerStore for StubPlayerStore {
        fn record_win(&self, name: &str) {
            self.win_calls.lock().unwrap().push(name.to_string());
        }
    }

    // ANCHOR: win_test
    #[test]
    fn records_a_win_from_user_input() {
        let store = Arc::new(StubPlayerStore::default());
        let alerter = Arc::new(SpyBlindAlerter::default());
        let input = "Chris wins\n".as_bytes();

        let mut cli = Cli::new(store.clone(), alerter, input);
        cli.play_poker();

        assert_eq!(*store.win_calls.lock().unwrap(), vec!["Chris".to_string()]);
    }
    // ANCHOR_END: win_test

    // ANCHOR: schedule_test
    #[test]
    fn schedules_blind_alerts_on_an_increasing_timer() {
        let store = Arc::new(StubPlayerStore::default());
        let alerter = Arc::new(SpyBlindAlerter::default());
        let input = "Chris wins\n".as_bytes();

        let mut cli = Cli::new(store, alerter.clone(), input);
        cli.play_poker();

        let want = vec![
            ScheduledAlert {
                at: minutes(0),
                amount: 100,
            },
            ScheduledAlert {
                at: minutes(10),
                amount: 200,
            },
            ScheduledAlert {
                at: minutes(20),
                amount: 300,
            },
            ScheduledAlert {
                at: minutes(30),
                amount: 400,
            },
            ScheduledAlert {
                at: minutes(40),
                amount: 500,
            },
            ScheduledAlert {
                at: minutes(50),
                amount: 600,
            },
            ScheduledAlert {
                at: minutes(60),
                amount: 800,
            },
            ScheduledAlert {
                at: minutes(70),
                amount: 1000,
            },
            ScheduledAlert {
                at: minutes(80),
                amount: 2000,
            },
            ScheduledAlert {
                at: minutes(90),
                amount: 4000,
            },
            ScheduledAlert {
                at: minutes(100),
                amount: 8000,
            },
        ];

        assert_eq!(*alerter.alerts.lock().unwrap(), want);
    }

    fn minutes(n: u64) -> Duration {
        Duration::from_secs(n * 60)
    }
    // ANCHOR_END: schedule_test
}
