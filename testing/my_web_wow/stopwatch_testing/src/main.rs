mod stopwatch;
use stopwatch::Stopwatch;

use std::{thread, time::Duration};

fn main() {
    let mut sw = Stopwatch::new();
    println!("Starting stopwatch and counting 3 seconds...");
    sw.start();

    for i in 1..=3 {
        thread::sleep(Duration::from_secs(1));
        println!(
            "{} second{} elapsed: {:.3} s",
            i,
            if i == 1 { "" } else { "s" },
            sw.elapsed_seconds()
        );
    }

    sw.stop();
    println!("Final elapsed time: {:.3} s", sw.elapsed_seconds());
}
