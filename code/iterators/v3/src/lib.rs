pub fn sum(numbers: &[i32]) -> i32 {
    numbers.iter().sum()
}

// ANCHOR: code
pub fn sum_all_tails(numbers_to_sum: &[&[i32]]) -> Vec<i32> {
    numbers_to_sum
        .iter()
        .map(|numbers| sum(numbers.get(1..).unwrap_or(&[])))
        .collect()
}
// ANCHOR_END: code

#[cfg(test)]
mod tests {
    use super::*;

    #[track_caller]
    fn check_sums(got: Vec<i32>, want: Vec<i32>) {
        assert_eq!(got, want);
    }

    #[test]
    fn sums_the_tails_of_several_collections() {
        check_sums(sum_all_tails(&[&[1, 2], &[0, 9]]), vec![2, 9]);
    }

    #[test]
    fn safely_sums_empty_collections() {
        check_sums(sum_all_tails(&[&[], &[3, 4, 5]]), vec![0, 9]);
    }
}
