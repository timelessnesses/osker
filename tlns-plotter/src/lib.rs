use std::io::Read;

use plotly;
use cuid2;

pub fn plot<const N: usize>(data: [String; N], thetas: [String; N]) -> std::io::Bytes<std::io::BufReader<std::fs::File>> {
    let sc = plotly::ScatterPolar::new(thetas.to_vec(), data.to_vec());
    let mut pl = plotly::Plot::new();
    pl.add_trace(sc);
    let file = "./".to_string() + &get_random_id_string() + ".png";
    pl.write_image(&file, plotly::ImageFormat::PNG, 1280, 720, 1.0);
    let file = std::fs::OpenOptions::new().read(true).open(file).expect("Failed to open file");
    std::io::BufReader::new(file).bytes()
}

fn get_random_id_string() -> String {
    cuid2::create_id()
}