// ANCHOR: code
#[derive(Debug)]
pub enum Shape {
    Rectangle { width: f64, height: f64 },
    Circle { radius: f64 },
    Triangle { base: f64, height: f64 },
}

pub fn area(shape: &Shape) -> f64 {
    match shape {
        Shape::Rectangle { width, height } => width * height,
        Shape::Circle { radius } => std::f64::consts::PI * radius * radius,
        Shape::Triangle { base, height } => (base * height) * 0.5,
    }
}
// ANCHOR_END: code

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn area_of_shapes() {
        let area_tests = [
            (
                "rectangle",
                Shape::Rectangle {
                    width: 12.0,
                    height: 6.0,
                },
                72.0,
            ),
            ("circle", Shape::Circle { radius: 10.0 }, 314.1592653589793),
            (
                "triangle",
                Shape::Triangle {
                    base: 12.0,
                    height: 6.0,
                },
                36.0,
            ),
        ];

        for (name, shape, want) in &area_tests {
            let got = area(shape);
            assert_eq!(got, *want, "{name}: {shape:?}");
        }
    }
}
