// ANCHOR: code
/// Adds two integers and returns the sum.
///
/// # Examples
///
/// ```
/// use integers_v2::add;
///
/// let sum = add(1, 5);
///
/// assert_eq!(sum, 6);
/// ```
pub fn add(x: i32, y: i32) -> i32 {
    x + y
}
// ANCHOR_END: code

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
