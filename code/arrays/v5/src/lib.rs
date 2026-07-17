pub fn sum(numbers: &[i32]) -> i32 {
    numbers.iter().sum()
}

// ANCHOR: code
pub fn sum_all_tails(numbers_to_sum: &[&[i32]]) -> Vec<i32> {
    let mut sums = Vec::new();
    for numbers in numbers_to_sum {
        if numbers.is_empty() {
            sums.push(0);
        } else {
            sums.push(sum(&numbers[1..]));
        }
    }
    sums
}
// ANCHOR_END: code

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sums_the_tails_of_several_collections() {
        let got = sum_all_tails(&[&[1, 2], &[0, 9]]);
        let want = vec![2, 9];

        assert_eq!(got, want);
    }

    // ANCHOR: empty_test
    #[test]
    fn safely_sums_empty_collections() {
        let got = sum_all_tails(&[&[], &[3, 4, 5]]);
        let want = vec![0, 9];

        assert_eq!(got, want);
    }
    // ANCHOR_END: empty_test
}
