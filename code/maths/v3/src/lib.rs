use std::f64::consts::PI;
use std::io::Write;

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

pub fn seconds_in_radians(time: &Time) -> f64 {
    PI / (30.0 / f64::from(time.seconds))
}

pub fn minutes_in_radians(time: &Time) -> f64 {
    seconds_in_radians(time) / 60.0 + PI / (30.0 / f64::from(time.minutes))
}

pub fn hours_in_radians(time: &Time) -> f64 {
    minutes_in_radians(time) / 12.0 + PI / (6.0 / f64::from(time.hours % 12))
}

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

// ANCHOR: svg
const CLOCK_CENTRE: Point = Point { x: 150.0, y: 150.0 };
const SECOND_HAND_LENGTH: f64 = 90.0;
const MINUTE_HAND_LENGTH: f64 = 80.0;
const HOUR_HAND_LENGTH: f64 = 50.0;

const SVG_START: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="no"?>
<svg xmlns="http://www.w3.org/2000/svg"
     width="100%"
     height="100%"
     viewBox="0 0 300 300"
     version="2.0">"#;

const BEZEL: &str =
    r#"<circle cx="150" cy="150" r="100" style="fill:#fff;stroke:#000;stroke-width:5px;"/>"#;

const SVG_END: &str = "</svg>";

pub fn svg_writer(writer: &mut impl Write, time: &Time) -> std::io::Result<()> {
    writeln!(writer, "{SVG_START}")?;
    writeln!(writer, "{BEZEL}")?;
    write_hand(writer, hour_hand_point(time), HOUR_HAND_LENGTH, "#000", 7)?;
    write_hand(
        writer,
        minute_hand_point(time),
        MINUTE_HAND_LENGTH,
        "#000",
        7,
    )?;
    write_hand(
        writer,
        second_hand_point(time),
        SECOND_HAND_LENGTH,
        "#f00",
        3,
    )?;
    writeln!(writer, "{SVG_END}")
}

fn write_hand(
    writer: &mut impl Write,
    unit_vector: Point,
    length: f64,
    colour: &str,
    width: u32,
) -> std::io::Result<()> {
    let tip = make_hand(unit_vector, length);
    writeln!(
        writer,
        r#"<line x1="150" y1="150" x2="{:.3}" y2="{:.3}" style="fill:none;stroke:{colour};stroke-width:{width}px;"/>"#,
        tip.x, tip.y
    )
}

fn make_hand(unit_vector: Point, length: f64) -> Point {
    let scaled = Point {
        x: unit_vector.x * length,
        y: unit_vector.y * length,
    };
    let flipped = Point {
        x: scaled.x,
        y: -scaled.y,
    };
    Point {
        x: flipped.x + CLOCK_CENTRE.x,
        y: flipped.y + CLOCK_CENTRE.y,
    }
}
// ANCHOR_END: svg

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

    // ANCHOR: xml_helpers
    #[derive(Debug)]
    struct Line {
        x1: f64,
        y1: f64,
        x2: f64,
        y2: f64,
    }

    fn svg_lines(svg: &str) -> Vec<Line> {
        let document = roxmltree::Document::parse(svg).expect("SVG output should be valid XML");

        document
            .descendants()
            .filter(|node| node.has_tag_name("line"))
            .map(|node| Line {
                x1: float_attribute(node, "x1"),
                y1: float_attribute(node, "y1"),
                x2: float_attribute(node, "x2"),
                y2: float_attribute(node, "y2"),
            })
            .collect()
    }

    fn float_attribute(node: roxmltree::Node, name: &str) -> f64 {
        node.attribute(name)
            .unwrap_or_else(|| panic!("line should have a {name} attribute"))
            .parse()
            .unwrap_or_else(|_| panic!("{name} should be a number"))
    }

    fn contains_line(lines: &[Line], want: &Line) -> bool {
        lines.iter().any(|line| {
            roughly_equal(line.x1, want.x1)
                && roughly_equal(line.y1, want.y1)
                && roughly_equal(line.x2, want.x2)
                && roughly_equal(line.y2, want.y2)
        })
    }

    fn render(time: &Time) -> String {
        let mut buffer = Vec::new();
        svg_writer(&mut buffer, time).unwrap();
        String::from_utf8(buffer).unwrap()
    }
    // ANCHOR_END: xml_helpers

    // ANCHOR: acceptance
    #[test]
    fn draws_the_second_hand_at_midnight() {
        let svg = render(&simple_time(0, 0, 0));

        let want = Line {
            x1: 150.0,
            y1: 150.0,
            x2: 150.0,
            y2: 60.0,
        };

        assert!(
            contains_line(&svg_lines(&svg), &want),
            "expected to find the second hand {want:?} in the SVG output {svg}"
        );
    }

    #[test]
    fn draws_the_second_hand_at_30_seconds() {
        let svg = render(&simple_time(0, 0, 30));

        let want = Line {
            x1: 150.0,
            y1: 150.0,
            x2: 150.0,
            y2: 240.0,
        };

        assert!(
            contains_line(&svg_lines(&svg), &want),
            "expected to find the second hand {want:?} in the SVG output {svg}"
        );
    }

    #[test]
    fn draws_the_minute_hand_at_midnight() {
        let svg = render(&simple_time(0, 0, 0));

        let want = Line {
            x1: 150.0,
            y1: 150.0,
            x2: 150.0,
            y2: 70.0,
        };

        assert!(
            contains_line(&svg_lines(&svg), &want),
            "expected to find the minute hand {want:?} in the SVG output {svg}"
        );
    }

    #[test]
    fn draws_the_hour_hand_at_six_oclock() {
        let svg = render(&simple_time(6, 0, 0));

        let want = Line {
            x1: 150.0,
            y1: 150.0,
            x2: 150.0,
            y2: 200.0,
        };

        assert!(
            contains_line(&svg_lines(&svg), &want),
            "expected to find the hour hand {want:?} in the SVG output {svg}"
        );
    }
    // ANCHOR_END: acceptance
}
