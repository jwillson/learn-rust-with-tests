// ANCHOR: trait
pub trait Shape {
    fn area(&self) -> f64;
}
// ANCHOR_END: trait

pub struct Rectangle {
    width: f64,
    height: f64,
}

// ANCHOR: impls
impl Shape for Rectangle {
    fn area(&self) -> f64 {
        self.width * self.height
    }
}

impl Shape for Circle {
    fn area(&self) -> f64 {
        std::f64::consts::PI * self.radius * self.radius
    }
}
// ANCHOR_END: impls

pub struct Circle {
    radius: f64,
}

// ANCHOR: test
#[cfg(test)]
mod tests {
    use super::*;

    #[track_caller]
    fn check_area(shape: &impl Shape, want: f64) {
        let got = shape.area();
        assert_eq!(got, want);
    }

    #[test]
    fn area_of_a_rectangle() {
        let rectangle = Rectangle {
            width: 12.0,
            height: 6.0,
        };

        check_area(&rectangle, 72.0);
    }

    #[test]
    fn area_of_a_circle() {
        let circle = Circle { radius: 10.0 };

        check_area(&circle, 314.1592653589793);
    }
}
// ANCHOR_END: test
