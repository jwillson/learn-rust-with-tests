use std::io::Write;
use std::time::Duration;

const FINAL_WORD: &str = "Go!";
const COUNTDOWN_START: u32 = 3;

// ANCHOR: sleeper
pub trait Sleeper {
    fn sleep(&self);
}

pub struct DefaultSleeper;

impl Sleeper for DefaultSleeper {
    fn sleep(&self) {
        std::thread::sleep(Duration::from_secs(1));
    }
}
// ANCHOR_END: sleeper

// ANCHOR: code
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
// ANCHOR_END: code

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    // ANCHOR: spy
    struct SpySleeper {
        calls: RefCell<u32>,
    }

    impl Sleeper for SpySleeper {
        fn sleep(&self) {
            *self.calls.borrow_mut() += 1;
        }
    }
    // ANCHOR_END: spy

    // ANCHOR: test
    #[test]
    fn counts_down_from_three_to_go() {
        let mut buffer = Vec::new();
        let spy_sleeper = SpySleeper {
            calls: RefCell::new(0),
        };

        countdown(&mut buffer, &spy_sleeper).unwrap();

        let got = String::from_utf8(buffer).unwrap();
        let want = "3\n2\n1\nGo!";

        assert_eq!(got, want);
        assert_eq!(
            *spy_sleeper.calls.borrow(),
            3,
            "not enough calls to sleeper"
        );
    }
    // ANCHOR_END: test
}
