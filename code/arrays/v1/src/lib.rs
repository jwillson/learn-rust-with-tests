// ANCHOR: code
pub fn sum(numbers: [i32; 5]) -> i32 {
    let mut total = 0;
    for number in numbers {
        total += number;
    }
    total
}
// ANCHOR_END: code

// ANCHOR: test
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sums_a_collection_of_numbers() {
        let numbers = [1, 2, 3, 4, 5];

        let got = sum(numbers);
        let want = 15;

        assert_eq!(got, want, "given {numbers:?}");
    }
}
// ANCHOR_END: test
