// ANCHOR: code
pub struct Countdown {
    current: u32,
}

impl Countdown {
    pub fn from(start: u32) -> Countdown {
        Countdown { current: start }
    }
}

impl Iterator for Countdown {
    type Item = u32;

    fn next(&mut self) -> Option<u32> {
        if self.current == 0 {
            None
        } else {
            let value = self.current;
            self.current -= 1;
            Some(value)
        }
    }
}
// ANCHOR_END: code

#[cfg(test)]
mod tests {
    use super::*;

    // ANCHOR: test
    #[test]
    fn counts_down_from_the_start_to_one() {
        let got: Vec<u32> = Countdown::from(3).collect();

        assert_eq!(got, vec![3, 2, 1]);
    }
    // ANCHOR_END: test

    // ANCHOR: free_tests
    #[test]
    fn works_in_a_for_loop() {
        let mut output = String::new();

        for number in Countdown::from(3) {
            output.push_str(&number.to_string());
            output.push('\n');
        }

        assert_eq!(output, "3\n2\n1\n");
    }

    #[test]
    fn every_adaptor_comes_free() {
        let doubled: Vec<u32> = Countdown::from(3).map(|n| n * 10).collect();
        assert_eq!(doubled, vec![30, 20, 10]);

        let total: u32 = Countdown::from(100).sum();
        assert_eq!(total, 5050);

        let evens_only: Vec<u32> = Countdown::from(5).filter(|n| n % 2 == 0).collect();
        assert_eq!(evens_only, vec![4, 2]);
    }
    // ANCHOR_END: free_tests
}
