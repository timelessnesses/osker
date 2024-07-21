use std::io::Cursor;

use plotters::{
    self,
    backend::BitMapBackend,
    chart::ChartBuilder,
    drawing::IntoDrawingArea,
    element::{PathElement, Polygon, Text},
    prelude::{RGBAColor, WHITE, BLUE, BLACK, Rectangle  },
    style::{Color, IntoFont}
};

use image::{RgbImage, Rgb};

pub fn plot_radar_one<const N: usize>(datas: [f64; N], thetas: [String; N], chart_name: String) -> Vec<u8> {
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

        // Find the maximum value in the dataset for normalization
        let max_value = datas.iter().copied().fold(0.0, f64::max);

        let coords = angles.iter().zip(datas.iter())
            .map(|(a, v)| ((v / max_value) * a.cos(), (v / max_value) * a.sin()))
            .collect::<Vec<(f64, f64)>>();

        chart.draw_series(std::iter::once(Polygon::new(
            coords.clone(),
            BLUE,
        ))).expect("Failed to draw polygon");

        chart.draw_series(std::iter::once(PathElement::new(
            (0..=360).map(|angle| {
                let rad = angle as f64 * core::f64::consts::PI / 180.0;
                (rad.cos(), rad.sin())
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

pub fn plot_radar_multiple<const N: usize, const A: usize>(
    datas: [[f64; N]; A], 
    thetas: [String; N], 
    markers: [String; A], 
    chart_name: String
) -> Vec<u8> {
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

        // Define colors with transparency
        let colors = vec![
            RGBAColor(255, 0, 0, 0.3),   // RED with transparency
            RGBAColor(0, 255, 0, 0.3),   // GREEN with transparency
            RGBAColor(0, 0, 255, 0.3),   // BLUE with transparency
            RGBAColor(0, 255, 255, 0.3), // CYAN with transparency
            RGBAColor(255, 20, 147, 0.3), // PINK with transparency
            RGBAColor(255, 255, 0, 0.3), // YELLOW with transparency
            RGBAColor(0, 0, 0, 0.3),     // BLACK with transparency
            RGBAColor(255, 255, 255, 0.3) // WHITE with transparency
        ];

        // Find the maximum value in the dataset for normalization
        let max_value = datas.iter()
            .flat_map(|data| data.iter().copied())
            .fold(0.0, f64::max);

        for (data, color) in datas.iter().zip(colors.iter().cycle()) {
            let coords = angles.iter().zip(data.iter())
                .map(|(a, v)| ((v / max_value) * a.cos(), (v / max_value) * a.sin()))
                .collect::<Vec<(f64, f64)>>();

            chart.draw_series(std::iter::once(Polygon::new(
                coords.clone(),
                color.filled(),
            ))).expect("Failed to draw polygon");
        }

        chart.draw_series(std::iter::once(PathElement::new(
            (0..=360).map(|angle| {
                let rad = angle as f64 * core::f64::consts::PI / 180.0;
                (rad.cos(), rad.sin())
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

        // Draw legend
        let legend_height = 40; // Match the rectangle height
        let legend_width = 40;
        let legend_margin = 10;
        let legend_x = 200;
        let legend_y = 200;

        for (i, (marker, color)) in markers.iter().zip(colors.iter().cycle()).enumerate() {
            let y = legend_y - i as i32 * (legend_height + legend_margin);
            drawing.draw(&Rectangle::new(
                [(legend_x, y), (legend_x + legend_width, y + legend_height)],
                color.filled(),
            )).unwrap();
            drawing.draw(&Text::new(
                marker.clone(),
                (legend_x + legend_width + 10, y + legend_height / 2),
                ("sans-serif", 60).into_font().color(&BLACK),
            )).unwrap();
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
