use charts_rs;

pub fn plot_radar_one<const N: usize>(
    datas: [f64; N],
    thetas: [String; N],
    chart_name: String,
) -> Vec<u8> {
    // thetas.reverse();
    let width = 800;
    let height = 600;
    let mut chart = charts_rs::RadarChart::new(
        vec![charts_rs::Series::new(
            "".to_string(),
            datas.map(|i| i as f32).to_vec(),
        )],
        thetas
            .iter()
            .map(|i| {
                let mut a = charts_rs::RadarIndicator::default();
                a.max = 180.0;
                a.name = i.clone();
                a
            })
            .collect(),
    );
    chart.series_colors = vec![
        charts_rs::Color::from("#67F9D8"),
        charts_rs::Color::from("#FFE434"),
        charts_rs::Color::from("#56A3F1"),
        charts_rs::Color::from("#FF917C"),
        charts_rs::Color::from("#67f976"),
        charts_rs::Color::from("#e434ff"),
    ];
    chart.title_text = chart_name;
    chart.title_align = charts_rs::Align::Center;
    chart.legend_show = Some(false);
    chart.width = width as f32;
    chart.height = height as f32;
    chart.x_axis_font_color = charts_rs::Color::from("#739ee7");
    chart.x_axis_name_gap = 3.0;
    chart.x_axis_stroke_color = charts_rs::Color::black();
    chart.series_symbol = Some(charts_rs::Symbol::Circle(18.0, None));
    charts_rs::svg_to_png(
        &chart
            .svg()
            .expect("Failed to turn the data to spider chart"),
    )
    .unwrap()
}

pub fn plot_radar_multiple(
    datas: Vec<Vec<f64>>,
    thetas: Vec<String>,
    markers: Vec<String>,
    chart_name: String,
) -> Vec<u8> {
    let width = 1500;
    let height = 1200;
    let mut chart = charts_rs::RadarChart::new(
        datas
            .iter()
            .zip(markers)
            .map(|(i, m)| charts_rs::Series::new(m, i.clone().iter().map(|i| *i as f32).collect()))
            .collect(),
        thetas
            .iter()
            .map(|i| {
                let mut a = charts_rs::RadarIndicator::default();
                a.max = 180.0;
                a.name = i.clone();
                a
            })
            .collect(),
    );
    chart.series_colors = vec![
        charts_rs::Color::from("#67F9D8"),
        charts_rs::Color::from("#FFE434"),
        charts_rs::Color::from("#56A3F1"),
        charts_rs::Color::from("#FF917C"),
        charts_rs::Color::from("#67f976"),
        charts_rs::Color::from("#e434ff"),
    ];
    chart.title_text = chart_name;
    chart.title_align = charts_rs::Align::Center;
    chart.legend_show = Some(true);
    chart.x_axis_font_color = charts_rs::Color::from("#739ee7");
    chart.x_axis_name_gap = 3.0;
    chart.x_axis_stroke_color = charts_rs::Color::black();
    chart.series_symbol = Some(charts_rs::Symbol::Circle(18.0, None));
    chart.width = width as f32;
    chart.height = height as f32;
    charts_rs::svg_to_png(
        &chart
            .svg()
            .expect("Failed to turn the data to spider chart"),
    )
    .unwrap()
}
