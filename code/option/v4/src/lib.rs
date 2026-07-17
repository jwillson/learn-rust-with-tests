// ANCHOR: code
pub fn largest(numbers: &[i32]) -> Option<i32> {
    numbers.iter().max().copied()
}
// ANCHOR_END: code

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_the_largest_number() {
        let got = largest(&[3, 7, 2]);
        let want = Some(7);

        assert_eq!(got, want);
    }

    #[test]
    fn an_empty_collection_has_no_largest_number() {
        let got = largest(&[]);
        let want = None;

        assert_eq!(got, want);
    }
}
