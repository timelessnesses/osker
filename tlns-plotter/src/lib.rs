use charming;

pub fn plot_radar_one<const N: usize>(
    datas: [f64; N],
    thetas: [String; N],
    chart_name: String,
) -> Vec<u8> {
    // thetas.reverse();
    let width = 1500;
    let height = 1200;
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
                    thetas
                        .iter()
                        .map(|i| {
                            charming::component::RadarIndicator::new()
                                .name(i)
                                .max(180)
                                .min(0)
                        })
                        .collect(),
                )
                .radius(480)
                .axis_name(
                    charming::component::RadarAxisName::new()
                        .color("#739ee7")
                        .padding((3, 5))
                        .font_size(40),
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
                .symbol(charming::element::Symbol::Circle)
                .symbol_size(18)
                .line_style(
                    charming::element::LineStyle::new()
                        .type_(charming::element::LineStyleType::Solid),
                ),
        ));
    let mut renderer = charming::ImageRenderer::new(width, height);
    renderer
        .render_format(charming::ImageFormat::Png, &chart)
        .expect("Failed to render plot")
}

pub fn plot_radar_multiple(
    datas: Vec<Vec<f64>>,
    thetas: Vec<String>,
    markers: Vec<String>,
    chart_name: String,
) -> Vec<u8> {
    let width = 1500;
    let height = 1400;
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
                    thetas
                        .iter()
                        .map(|i| {
                            charming::component::RadarIndicator::new()
                                .name(i)
                                .max(180)
                                .min(0)
                        })
                        .collect(),
                )
                .radius(480)
                .axis_name(
                    charming::component::RadarAxisName::new()
                        .color("#739ee7")
                        .padding((3, 5))
                        .font_size(40),
                )
                .axis_line(
                    charming::element::AxisLine::new()
                        .line_style(charming::element::AxisLineStyle::new().color((0.0, "black"))),
                ),
        )
        .series(
            charming::series::Radar::new()
                .data(
                    datas
                        .iter()
                        .zip(markers)
                        .map(|(i, a)| {
                            charming::datatype::DataPoint::Item(charming::datatype::DataPointItem::new(charming::datatype::CompositeValue::Array(
                                i.iter()
                                    .map(|i| {
                                        charming::datatype::CompositeValue::Number(
                                            charming::datatype::NumericValue::Float(*i),
                                        )
                                    })
                                    .collect(),
                            )).name(a))
                        })
                        .collect(),
                )
                .symbol(charming::element::Symbol::Circle)
                .symbol_size(18)
                .line_style(
                    charming::element::LineStyle::new()
                        .type_(charming::element::LineStyleType::Solid),
                )
        );
    let mut renderer = charming::ImageRenderer::new(width, height);
    renderer
        .render_format(charming::ImageFormat::Png, &chart)
        .expect("Failed to render plot")
}
