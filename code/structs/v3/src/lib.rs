// ANCHOR: code
pub struct Rectangle {
    width: f64,
    height: f64,
}

impl Rectangle {
    pub fn area(&self) -> f64 {
        self.width * self.height
    }

    pub fn perimeter(&self) -> f64 {
        2.0 * (self.width + self.height)
    }
}

pub struct Circle {
    radius: f64,
}

impl Circle {
    pub fn area(&self) -> f64 {
        std::f64::consts::PI * self.radius * self.radius
    }
}
// ANCHOR_END: code

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn perimeter_of_a_rectangle() {
        let rectangle = Rectangle {
            width: 10.0,
            height: 10.0,
        };

        assert_eq!(rectangle.perimeter(), 40.0);
    }

    // ANCHOR: test
    #[test]
    fn area_of_a_rectangle() {
        let rectangle = Rectangle {
            width: 12.0,
            height: 6.0,
        };

        let got = rectangle.area();
        let want = 72.0;

        assert_eq!(got, want);
    }

    #[test]
    fn area_of_a_circle() {
        let circle = Circle { radius: 10.0 };

        let got = circle.area();
        let want = 314.1592653589793;

        assert_eq!(got, want);
    }
    // ANCHOR_END: test
}
