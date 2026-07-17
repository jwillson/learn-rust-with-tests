// ANCHOR: code
#[derive(Debug)]
pub enum Shape {
    Rectangle { width: f64, height: f64 },
    Circle { radius: f64 },
}

pub fn area(shape: &Shape) -> f64 {
    match shape {
        Shape::Rectangle { width, height } => width * height,
        Shape::Circle { radius } => std::f64::consts::PI * radius * radius,
    }
}
// ANCHOR_END: code

#[cfg(test)]
mod tests {
    use super::*;

    // ANCHOR: test
    #[test]
    fn area_of_a_rectangle() {
        let rectangle = Shape::Rectangle {
            width: 12.0,
            height: 6.0,
        };

        assert_eq!(area(&rectangle), 72.0);
    }

    #[test]
    fn area_of_a_circle() {
        let circle = Shape::Circle { radius: 10.0 };

        assert_eq!(area(&circle), 314.1592653589793);
    }
    // ANCHOR_END: test
}
