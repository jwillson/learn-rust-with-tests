// ANCHOR: code
pub fn add(x: i32, y: i32) -> i32 {
    x + y
}
// ANCHOR_END: code

// ANCHOR: test
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adding_two_numbers() {
        let sum = add(2, 2);
        let want = 4;

        assert_eq!(sum, want);
    }
}
// ANCHOR_END: test
