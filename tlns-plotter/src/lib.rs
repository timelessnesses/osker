use std::io::Cursor;

use charming;
use image::{Rgb, RgbImage};

pub fn plot_radar_one<const N: usize>(
    datas: [f64; N],
    thetas: [String; N],
    chart_name: String,
    weights: [f64; N],
) -> Vec<u8> {
    let width = 2000;
    let height = 2000;
    let chart = charming::Chart::new()
        .color(vec![
            charming::element::Color::Value("#67F9D8".to_string()),
            charming::element::Color::Value("#FFE434".to_string()),
            charming::element::Color::Value("#56A3F1".to_string()),
            charming::element::Color::Value("#FF917C".to_string()),
            charming::element::Color::Value("#67f976".to_string()),
            charming::element::Color::Value("#e434ff".to_string()),
        ])
        .title(
            charming::component::Title::new()
                .text(chart_name)
                .text_align(charming::element::TextAlign::Center),
        )
        .radar(
            charming::component::RadarCoordinate::new()
                .indicator(
                    weights
                        .iter()
                        .enumerate()
                        .map(|(i, w)| (thetas[i].as_str(), 0.0_f64, *w))
                        .collect(),
                )
                .radius(120)
                .axis_name(
                    charming::component::RadarAxisName::new()
                        .color("#fff")
                        .padding((3, 5)),
                ),
        )
        .series(charming::series::Series::Radar(
            charming::series::Radar::new()
                .radar_index(0)
                .data(vec![charming::datatype::DataPoint::Value(
                    charming::datatype::CompositeValue::Array(
                        datas
                            .iter()
                            .map(|i| {
                                charming::datatype::CompositeValue::Number(
                                    charming::datatype::NumericValue::Float(*i),
                                )
                            })
                            .collect(),
                    ),
                )])
                .symbol(charming::element::Symbol::None)
                .symbol_size(9)
                .line_style(charming::element::LineStyle::new().type_(charming::element::LineStyleType::Solid)),
        ));
    let mut renderer = charming::ImageRenderer::new(width, height);
    renderer
        .render_format(charming::ImageFormat::Png, &chart)
        .expect("Failed to render plot")
}

pub fn plot_radar_multiple<const N: usize, const A: usize>(
    datas: [[f64; N]; A],
    thetas: [String; N],
    markers: [String; A],
    chart_name: String,
) -> Vec<u8> {
    let width = 2000;
    let height = 2000;
    todo!()
}
