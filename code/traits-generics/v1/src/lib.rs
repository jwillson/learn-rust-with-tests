// ANCHOR: code
#[track_caller]
pub fn assert_equal(got: i32, want: i32) {
    if got != want {
        panic!("got {got}, want {want}");
    }
}

#[track_caller]
pub fn assert_not_equal(got: i32, want: i32) {
    if got == want {
        panic!("didn't want {got}");
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
    // ANCHOR_END: test
}
