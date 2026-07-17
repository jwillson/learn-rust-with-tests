// ANCHOR: struct
pub struct Rectangle {
    width: f64,
    height: f64,
}
// ANCHOR_END: struct

// ANCHOR: code
pub fn perimeter(rectangle: &Rectangle) -> f64 {
    2.0 * (rectangle.width + rectangle.height)
}

pub fn area(rectangle: &Rectangle) -> f64 {
    rectangle.width * rectangle.height
}
// ANCHOR_END: code

#[cfg(test)]
mod tests {
    use super::*;

    // ANCHOR: test
    #[test]
    fn perimeter_of_a_rectangle() {
        let rectangle = Rectangle {
            width: 10.0,
            height: 10.0,
        };

        let got = perimeter(&rectangle);
        let want = 40.0;

        assert_eq!(got, want);
    }
    // ANCHOR_END: test

    #[test]
    fn area_of_a_rectangle() {
        let rectangle = Rectangle {
            width: 12.0,
            height: 6.0,
        };

        let got = area(&rectangle);
        let want = 72.0;

        assert_eq!(got, want);
    }
}
