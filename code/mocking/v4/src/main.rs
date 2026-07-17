use std::io::Write;
use std::time::Duration;

const FINAL_WORD: &str = "Go!";
const COUNTDOWN_START: u32 = 3;

pub trait Sleeper {
    fn sleep(&self);
}

// ANCHOR: configurable
pub struct ConfigurableSleeper<F: Fn(Duration)> {
    duration: Duration,
    sleep_fn: F,
}

impl<F: Fn(Duration)> Sleeper for ConfigurableSleeper<F> {
    fn sleep(&self) {
        (self.sleep_fn)(self.duration);
    }
}
// ANCHOR_END: configurable

fn countdown(out: &mut impl Write, sleeper: &impl Sleeper) -> std::io::Result<()> {
    for i in (1..=COUNTDOWN_START).rev() {
        writeln!(out, "{i}")?;
        sleeper.sleep();
    }

    write!(out, "{FINAL_WORD}")
}

// ANCHOR: main
fn main() -> std::io::Result<()> {
    let sleeper = ConfigurableSleeper {
        duration: Duration::from_secs(1),
        sleep_fn: std::thread::sleep,
    };

    countdown(&mut std::io::stdout(), &sleeper)
}
// ANCHOR_END: main

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    #[derive(Debug, PartialEq, Eq, Clone, Copy)]
    enum Operation {
        Write,
        Sleep,
    }

    struct SpySleeper {
        calls: Rc<RefCell<Vec<Operation>>>,
    }

    impl Sleeper for SpySleeper {
        fn sleep(&self) {
            self.calls.borrow_mut().push(Operation::Sleep);
        }
    }

    struct SpyWriter {
        calls: Rc<RefCell<Vec<Operation>>>,
    }

    impl Write for SpyWriter {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.calls.borrow_mut().push(Operation::Write);
            Ok(buf.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }

        fn write_fmt(&mut self, _: std::fmt::Arguments<'_>) -> std::io::Result<()> {
            self.calls.borrow_mut().push(Operation::Write);
            Ok(())
        }
    }

    #[test]
    fn counts_down_from_three_to_go() {
        let mut buffer = Vec::new();
        let spy_sleeper = SpySleeper {
            calls: Rc::new(RefCell::new(Vec::new())),
        };

        countdown(&mut buffer, &spy_sleeper).unwrap();

        let got = String::from_utf8(buffer).unwrap();
        let want = "3\n2\n1\nGo!";

        assert_eq!(got, want);
    }

    #[test]
    fn sleeps_before_every_print() {
        let calls = Rc::new(RefCell::new(Vec::new()));
        let mut writer = SpyWriter {
            calls: Rc::clone(&calls),
        };
        let sleeper = SpySleeper {
            calls: Rc::clone(&calls),
        };

        countdown(&mut writer, &sleeper).unwrap();

        use Operation::*;
        assert_eq!(
            *calls.borrow(),
            vec![Write, Sleep, Write, Sleep, Write, Sleep, Write]
        );
    }

    // ANCHOR: configurable_test
    #[test]
    fn configurable_sleeper_sleeps_for_the_configured_duration() {
        let slept = RefCell::new(Duration::ZERO);
        let sleeper = ConfigurableSleeper {
            duration: Duration::from_secs(5),
            sleep_fn: |duration| *slept.borrow_mut() = duration,
        };

        sleeper.sleep();

        assert_eq!(*slept.borrow(), Duration::from_secs(5));
    }
    // ANCHOR_END: configurable_test
}
