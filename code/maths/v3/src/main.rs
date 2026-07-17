// ANCHOR: main
use std::time::{SystemTime, UNIX_EPOCH};

use maths_v3::{Time, svg_writer};

fn main() -> std::io::Result<()> {
    let seconds_since_epoch = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock is set before 1970")
        .as_secs();

    let time = Time {
        hours: ((seconds_since_epoch / 3600) % 24) as u32,
        minutes: ((seconds_since_epoch / 60) % 60) as u32,
        seconds: (seconds_since_epoch % 60) as u32,
    };

    svg_writer(&mut std::io::stdout().lock(), &time)
}
// ANCHOR_END: main
