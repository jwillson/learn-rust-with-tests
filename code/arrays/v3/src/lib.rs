// ANCHOR: sum
pub fn sum(numbers: &[i32]) -> i32 {
    numbers.iter().sum()
}
// ANCHOR_END: sum

// ANCHOR: sum_all
pub fn sum_all(numbers_to_sum: &[&[i32]]) -> Vec<i32> {
    let mut sums = Vec::new();
    for numbers in numbers_to_sum {
        sums.push(sum(numbers));
    }
    sums
}
// ANCHOR_END: sum_all

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sums_a_collection_of_any_size() {
        let numbers = [1, 2, 3];

        let got = sum(&numbers);
        let want = 6;

        assert_eq!(got, want, "given {numbers:?}");
    }

    // ANCHOR: sum_all_test
    #[test]
    fn sums_several_collections() {
        let got = sum_all(&[&[1, 2], &[0, 9]]);
        let want = vec![3, 9];

        assert_eq!(got, want);
    }
    // ANCHOR_END: sum_all_test
}
