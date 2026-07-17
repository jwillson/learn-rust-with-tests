use std::f64::consts::PI;

// ANCHOR: types
pub struct Time {
    pub hours: u32,
    pub minutes: u32,
    pub seconds: u32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}
// ANCHOR_END: types

// ANCHOR: radians
pub fn seconds_in_radians(time: &Time) -> f64 {
    PI / (30.0 / f64::from(time.seconds))
}
// ANCHOR_END: radians

// ANCHOR: point_fn
pub fn second_hand_point(time: &Time) -> Point {
    let angle = seconds_in_radians(time);

    Point {
        x: angle.sin(),
        y: angle.cos(),
    }
}
// ANCHOR_END: point_fn

#[cfg(test)]
mod tests {
    use super::*;

    // ANCHOR: helpers
    fn simple_time(hours: u32, minutes: u32, seconds: u32) -> Time {
        Time {
            hours,
            minutes,
            seconds,
        }
    }

    const EQUALITY_THRESHOLD: f64 = 1e-7;

    fn roughly_equal(a: f64, b: f64) -> bool {
        (a - b).abs() < EQUALITY_THRESHOLD
    }

    fn roughly_equal_point(a: Point, b: Point) -> bool {
        roughly_equal(a.x, b.x) && roughly_equal(a.y, b.y)
    }
    // ANCHOR_END: helpers

    // ANCHOR: radians_test
    #[test]
    fn converts_seconds_to_an_angle_in_radians() {
        let cases = [
            (simple_time(0, 0, 0), 0.0),
            (simple_time(0, 0, 30), PI),
            (simple_time(0, 0, 45), (PI / 2.0) * 3.0),
            (simple_time(0, 0, 7), (PI / 30.0) * 7.0),
        ];

        for (time, want) in cases {
            let got = seconds_in_radians(&time);
            assert!(
                roughly_equal(got, want),
                "at {} seconds wanted {want} radians, got {got}",
                time.seconds
            );
        }
    }
    // ANCHOR_END: radians_test

    // ANCHOR: point_test
    #[test]
    fn finds_the_unit_vector_for_the_second_hand() {
        let cases = [
            (simple_time(0, 0, 30), Point { x: 0.0, y: -1.0 }),
            (simple_time(0, 0, 45), Point { x: -1.0, y: 0.0 }),
        ];

        for (time, want) in cases {
            let got = second_hand_point(&time);
            assert!(
                roughly_equal_point(got, want),
                "at {} seconds wanted {want:?}, got {got:?}",
                time.seconds
            );
        }
    }
    // ANCHOR_END: point_test
}
