// ANCHOR: code
pub fn largest(numbers: &[i32]) -> Option<i32> {
    let mut result = None;

    for &number in numbers {
        match result {
            None => result = Some(number),
            Some(current) if number > current => result = Some(number),
            Some(_) => {}
        }
    }

    result
}
// ANCHOR_END: code

#[cfg(test)]
mod tests {
    use super::*;

    // ANCHOR: test
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
    // ANCHOR_END: test
}
