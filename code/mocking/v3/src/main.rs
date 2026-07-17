use std::io::Write;
use std::time::Duration;

const FINAL_WORD: &str = "Go!";
const COUNTDOWN_START: u32 = 3;

pub trait Sleeper {
    fn sleep(&self);
}

pub struct DefaultSleeper;

impl Sleeper for DefaultSleeper {
    fn sleep(&self) {
        std::thread::sleep(Duration::from_secs(1));
    }
}

fn countdown(out: &mut impl Write, sleeper: &impl Sleeper) -> std::io::Result<()> {
    for i in (1..=COUNTDOWN_START).rev() {
        writeln!(out, "{i}")?;
        sleeper.sleep();
    }

    write!(out, "{FINAL_WORD}")
}

fn main() -> std::io::Result<()> {
    countdown(&mut std::io::stdout(), &DefaultSleeper)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    // ANCHOR: spy
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

        // `writeln!` makes one `write_fmt` call, which by default becomes
        // several small `write` calls (one per formatted fragment). We record
        // at the granularity we care about: one operation per print.
        fn write_fmt(&mut self, _: std::fmt::Arguments<'_>) -> std::io::Result<()> {
            self.calls.borrow_mut().push(Operation::Write);
            Ok(())
        }
    }
    // ANCHOR_END: spy

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

    // ANCHOR: test
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
    // ANCHOR_END: test
}
