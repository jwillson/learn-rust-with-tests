// ANCHOR: code
pub fn perimeter(width: f64, height: f64) -> f64 {
    2.0 * (width + height)
}

pub fn area(width: f64, height: f64) -> f64 {
    width * height
}
// ANCHOR_END: code

#[cfg(test)]
mod tests {
    use super::*;

    // ANCHOR: test
    #[test]
    fn perimeter_of_a_rectangle() {
        let got = perimeter(10.0, 10.0);
        let want = 40.0;

        assert_eq!(got, want);
    }
    // ANCHOR_END: test

    #[test]
    fn area_of_a_rectangle() {
        let got = area(12.0, 6.0);
        let want = 72.0;

        assert_eq!(got, want);
    }
}
