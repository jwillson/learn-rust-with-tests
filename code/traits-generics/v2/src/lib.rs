use std::fmt::Debug;

// ANCHOR: code
#[track_caller]
pub fn assert_equal<T: PartialEq + Debug>(got: T, want: T) {
    if got != want {
        panic!("got {got:?}, want {want:?}");
    }
}

#[track_caller]
pub fn assert_not_equal<T: PartialEq + Debug>(got: T, want: T) {
    if got == want {
        panic!("didn't want {got:?}");
    }
}
// ANCHOR_END: code

#[cfg(test)]
mod tests {
    use super::*;

    // ANCHOR: test
    #[test]
    fn asserting_on_integers() {
        assert_equal(1, 1);
        assert_not_equal(1, 2);
    }

    #[test]
    fn asserting_on_strings() {
        assert_equal("hello", "hello");
        assert_not_equal("hello", "Grace");
    }

    // assert_equal(1, "1"); // uncomment to see the error
    // ANCHOR_END: test
}
