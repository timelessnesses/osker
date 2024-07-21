use std::io::Cursor;

use plotters::{
    self,
    backend::BitMapBackend,
    chart::ChartBuilder,
    drawing::IntoDrawingArea,
    element::{PathElement, Polygon, Text},
    style::full_palette::{BLACK, BLUE, WHITE},
    style::Color
};

use image::{RgbImage, Rgb};

pub fn plot<const N: usize>(datas: [f64; N], thetas: [String; N], chart_name: String) -> Vec<u8> {
    let width = 2000;
    let height = 2000;
    let mut buffer = vec![0; width * height * 3];
    {
        let image = BitMapBackend::with_buffer(&mut buffer, (width as u32, height as u32));
        let drawing = image.into_drawing_area();
        drawing.fill(&WHITE).expect("Failed to fill drawing area");

        let mut chart = ChartBuilder::on(&drawing)
            .caption(chart_name, ("sans-serif", 120))
            .build_cartesian_2d(-1.5..1.5, -1.5..1.5)
            .expect("Failed to build chart");

        let dimensions = N;
        let angles = (0..dimensions)
            .map(|i| 2.0 * core::f64::consts::PI * (i as f64) / (dimensions as f64))
            .collect::<Vec<f64>>();

        let coords = angles.iter().zip(datas.iter())
            .map(|(a, v)| (v * a.cos(), v * a.sin()))
            .collect::<Vec<(f64, f64)>>();

        chart.draw_series(std::iter::once(Polygon::new(
            coords.clone(),
            &BLUE,
        ))).expect("Failed to draw polygon");

        let max_value = datas.iter().copied().fold(0.0, f64::max);

        chart.draw_series(std::iter::once(PathElement::new(
            (0..=360).map(|angle| {
                let rad = angle as f64 * core::f64::consts::PI / 180.0;
                (max_value * rad.cos(), max_value * rad.sin())
            }).collect::<Vec<(f64, f64)>>(),
            BLACK.stroke_width(2),
        ))).expect("Failed to draw outer circle");

        for a in &angles {
            chart.draw_series(
                std::iter::once(
                    PathElement::new(
                        vec![(0.0, 0.0), (a.cos(), a.sin())],
                        &BLACK
                    )
                )
            ).expect("Failed to draw paths");
        }

        for (angle, theta) in angles.iter().zip(thetas.iter()) {
            chart.draw_series(
                std::iter::once(
                    Text::new(
                        theta.clone(),
                        (1.2 * angle.cos(), 1.2 * angle.sin()),
                        ("sans-serif", 150)
                    )
                )
            ).unwrap();
        }
    }
    let mut img = RgbImage::new(width as u32, height as u32);
    for (i, pixel) in buffer.chunks_exact(3).enumerate() {
        let x = (i % width) as u32;
        let y = (i / width) as u32;
        img.put_pixel(x, y, Rgb([pixel[0], pixel[1], pixel[2]]));
    }

    let mut png_data = Cursor::new(Vec::new());
    img.write_to(&mut png_data, image::ImageFormat::Png).unwrap();

    png_data.into_inner()
}