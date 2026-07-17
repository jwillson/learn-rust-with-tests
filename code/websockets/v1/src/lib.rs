use std::sync::Arc;
use std::time::Duration;

pub trait PlayerStore: Send + Sync {
    fn record_win(&self, name: &str);
}

pub trait BlindAlerter: Send + Sync {
    fn schedule_alert_at(&self, at: Duration, amount: u32);
}

// ANCHOR: game
pub trait Game: Send + Sync {
    fn start(&self, number_of_players: u32);
    fn finish(&self, winner: &str);
}
// ANCHOR_END: game

// ANCHOR: texas
pub struct TexasHoldem {
    store: Arc<dyn PlayerStore>,
    alerter: Arc<dyn BlindAlerter>,
}

impl TexasHoldem {
    pub fn new(store: Arc<dyn PlayerStore>, alerter: Arc<dyn BlindAlerter>) -> TexasHoldem {
        TexasHoldem { store, alerter }
    }
}

impl Game for TexasHoldem {
    fn start(&self, number_of_players: u32) {
        let blind_increment = Duration::from_secs(60 * (5 + u64::from(number_of_players)));
        let blinds = [100, 200, 300, 400, 500, 600, 800, 1000, 2000, 4000, 8000];

        let mut blind_time = Duration::ZERO;
        for blind in blinds {
            self.alerter.schedule_alert_at(blind_time, blind);
            blind_time += blind_increment;
        }
    }

    fn finish(&self, winner: &str) {
        self.store.record_win(winner);
    }
}
// ANCHOR_END: texas

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

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

    #[derive(Default)]
    struct StubPlayerStore {
        win_calls: Mutex<Vec<String>>,
    }

    impl PlayerStore for StubPlayerStore {
        fn record_win(&self, name: &str) {
            self.win_calls.lock().unwrap().push(name.to_string());
        }
    }

    // ANCHOR: test
    #[test]
    fn scheduling_gets_slower_with_more_players() {
        let alerter = Arc::new(SpyBlindAlerter::default());
        let game = TexasHoldem::new(Arc::new(StubPlayerStore::default()), alerter.clone());

        game.start(5);

        let alerts = alerter.alerts.lock().unwrap();
        // With 5 players the increment is 10 minutes.
        assert_eq!(alerts[0], scheduled(0, 100));
        assert_eq!(alerts[1], scheduled(10, 200));
        assert_eq!(alerts[2], scheduled(20, 300));
    }

    #[test]
    fn finishing_records_the_winner() {
        let store = Arc::new(StubPlayerStore::default());
        let game = TexasHoldem::new(store.clone(), Arc::new(SpyBlindAlerter::default()));

        game.finish("Ruth");

        assert_eq!(*store.win_calls.lock().unwrap(), vec!["Ruth".to_string()]);
    }

    fn scheduled(minutes: u64, amount: u32) -> ScheduledAlert {
        ScheduledAlert {
            at: Duration::from_secs(minutes * 60),
            amount,
        }
    }
    // ANCHOR_END: test
}
