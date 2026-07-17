use std::fmt::Debug;

// ANCHOR: trait
pub trait Shape: Debug {
    fn area(&self) -> f64;
}
// ANCHOR_END: trait

// ANCHOR: shapes
#[derive(Debug)]
pub struct Rectangle {
    width: f64,
    height: f64,
}

impl Shape for Rectangle {
    fn area(&self) -> f64 {
        self.width * self.height
    }
}

#[derive(Debug)]
pub struct Circle {
    radius: f64,
}

impl Shape for Circle {
    fn area(&self) -> f64 {
        std::f64::consts::PI * self.radius * self.radius
    }
}

#[derive(Debug)]
pub struct Triangle {
    base: f64,
    height: f64,
}

impl Shape for Triangle {
    fn area(&self) -> f64 {
        (self.base * self.height) * 0.5
    }
}
// ANCHOR_END: shapes

// ANCHOR: test
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn area_of_shapes() {
        let area_tests: &[(&str, &dyn Shape, f64)] = &[
            (
                "rectangle",
                &Rectangle {
                    width: 12.0,
                    height: 6.0,
                },
                72.0,
            ),
            ("circle", &Circle { radius: 10.0 }, 314.1592653589793),
            (
                "triangle",
                &Triangle {
                    base: 12.0,
                    height: 6.0,
                },
                36.0,
            ),
        ];

        for (name, shape, want) in area_tests {
            let got = shape.area();
            assert_eq!(got, *want, "{name}: {shape:?}");
        }
    }
}
// ANCHOR_END: test
