use std::io::Write;

use tlns_plotter;

fn main() {
    std::fs::OpenOptions::new().write(true).create(true).open("test.png").unwrap().write(&tlns_plotter::plot([1.0,1.0,1.0], ["a".to_string(), "b".to_string(), "c".to_string()], "hi".to_string())).expect("Failed to write");
}