use std::f64::consts::PI;

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

// ANCHOR: radians
pub fn seconds_in_radians(time: &Time) -> f64 {
    PI / (30.0 / f64::from(time.seconds))
}

pub fn minutes_in_radians(time: &Time) -> f64 {
    seconds_in_radians(time) / 60.0 + PI / (30.0 / f64::from(time.minutes))
}

pub fn hours_in_radians(time: &Time) -> f64 {
    minutes_in_radians(time) / 12.0 + PI / (6.0 / f64::from(time.hours % 12))
}
// ANCHOR_END: radians

// ANCHOR: points
fn angle_to_point(angle: f64) -> Point {
    Point {
        x: angle.sin(),
        y: angle.cos(),
    }
}

pub fn second_hand_point(time: &Time) -> Point {
    angle_to_point(seconds_in_radians(time))
}

pub fn minute_hand_point(time: &Time) -> Point {
    angle_to_point(minutes_in_radians(time))
}

pub fn hour_hand_point(time: &Time) -> Point {
    angle_to_point(hours_in_radians(time))
}
// ANCHOR_END: points

#[cfg(test)]
mod tests {
    use super::*;

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

    // ANCHOR: minutes_test
    #[test]
    fn converts_minutes_to_an_angle_in_radians() {
        let cases = [
            (simple_time(0, 30, 0), PI),
            (simple_time(0, 0, 7), 7.0 * (PI / (30.0 * 60.0))),
            (simple_time(0, 45, 0), (PI / 2.0) * 3.0),
        ];

        for (time, want) in cases {
            let got = minutes_in_radians(&time);
            assert!(
                roughly_equal(got, want),
                "at {}m {}s wanted {want} radians, got {got}",
                time.minutes,
                time.seconds
            );
        }
    }

    #[test]
    fn converts_hours_to_an_angle_in_radians() {
        let cases = [
            (simple_time(6, 0, 0), PI),
            (simple_time(0, 0, 0), 0.0),
            (simple_time(21, 0, 0), (PI / 2.0) * 3.0),
            (simple_time(0, 1, 30), PI / ((6.0 * 60.0 * 60.0) / 90.0)),
        ];

        for (time, want) in cases {
            let got = hours_in_radians(&time);
            assert!(
                roughly_equal(got, want),
                "at {}h {}m {}s wanted {want} radians, got {got}",
                time.hours,
                time.minutes,
                time.seconds
            );
        }
    }
    // ANCHOR_END: minutes_test

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

    // ANCHOR: hand_point_tests
    #[test]
    fn finds_the_unit_vector_for_the_minute_hand() {
        let cases = [
            (simple_time(0, 30, 0), Point { x: 0.0, y: -1.0 }),
            (simple_time(0, 45, 0), Point { x: -1.0, y: 0.0 }),
        ];

        for (time, want) in cases {
            let got = minute_hand_point(&time);
            assert!(
                roughly_equal_point(got, want),
                "at {} minutes wanted {want:?}, got {got:?}",
                time.minutes
            );
        }
    }

    #[test]
    fn finds_the_unit_vector_for_the_hour_hand() {
        let cases = [
            (simple_time(6, 0, 0), Point { x: 0.0, y: -1.0 }),
            (simple_time(21, 0, 0), Point { x: -1.0, y: 0.0 }),
        ];

        for (time, want) in cases {
            let got = hour_hand_point(&time);
            assert!(
                roughly_equal_point(got, want),
                "at {} hours wanted {want:?}, got {got:?}",
                time.hours
            );
        }
    }
    // ANCHOR_END: hand_point_tests
}
