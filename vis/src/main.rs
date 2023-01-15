extern crate plotly;
use plotly::common::Mode;
use plotly::layout::Axis;
use plotly::{Plot, Scatter, Trace};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let data_str = std::fs::read_to_string("../polones-desktop/data2").unwrap();
    let mut data = Vec::new();
    for line in data_str.split("\n").filter(|l| !l.is_empty()) {
        let mut items = line.split_whitespace();
        if let (Some(_), Some(event), Some(time)) = (items.next(), items.next(), items.next()) {
            data.push((
                event.to_owned(),
                time.parse::<u64>().unwrap() - 1673719210000000,
            ));
        } else {
            std::process::exit(1);
        }
    }

    let min_time = data.iter().map(|(_, time)| *time).min().unwrap();

    for (event, time) in &mut data {
        *time -= min_time;
    }

    let mut traces = Vec::<Box<dyn Trace + 'static>>::new();

    traces.push(
        Scatter::new(
            data.iter()
                .filter_map(|(event, time)| (event == "audio_requested").then(|| *time as f32))
                .collect(),
            data.iter()
                .filter_map(|(event, time)| (event == "audio_requested").then(|| 0.0))
                .collect(),
        )
        .name("audio_requested")
        .mode(Mode::Markers),
    );

    traces.push(
        Scatter::new(
            data.iter()
                .filter_map(|(event, time)| (event == "audio_ready").then(|| *time as f32))
                .collect(),
            data.iter()
                .filter_map(|(event, time)| (event == "audio_ready").then(|| 2.0))
                .collect(),
        )
        .name("audio_ready")
        .mode(Mode::Markers),
    );

    traces.push(
        Scatter::new(
            data.iter()
                .filter_map(|(event, time)| (event == "frame_ready").then(|| *time as f32))
                .collect(),
            data.iter()
                .filter_map(|(event, time)| (event == "frame_ready").then(|| 5.0))
                .collect(),
        )
        .name("frame_ready")
        .mode(Mode::Markers),
    );

    traces.push(
        Scatter::new(
            data.iter()
                .filter_map(|(event, time)| (event == "game_window_drawn").then(|| *time as f32))
                .collect(),
            data.iter()
                .filter_map(|(event, time)| (event == "game_window_drawn").then(|| 10.0))
                .collect(),
        )
        .name("game_window_drawn")
        .mode(Mode::Markers),
    );

    traces.push(
        Scatter::new(
            data.iter()
                .filter_map(|(event, time)| (event == "game_window_ready").then(|| *time as f32))
                .collect(),
            data.iter()
                .filter_map(|(event, time)| (event == "game_window_ready").then(|| 12.0))
                .collect(),
        )
        .name("game_window_ready")
        .mode(Mode::Markers),
    );

    traces.push(
        Scatter::new(
            data.iter()
                .filter_map(|(event, time)| (event == "frame_repeated").then(|| *time as f32))
                .collect(),
            data.iter()
                .filter_map(|(event, time)| (event == "frame_repeated").then(|| 15.0))
                .collect(),
        )
        .name("frame_repeated")
        .mode(Mode::Markers),
    );

    traces.push(
        Scatter::new(
            data.iter()
                .filter_map(|(event, time)| (event == "frame_skipped").then(|| *time as f32))
                .collect(),
            data.iter()
                .filter_map(|(event, time)| (event == "frame_skipped").then(|| 17.0))
                .collect(),
        )
        .name("frame_skipped")
        .mode(Mode::Markers),
    );

    let mut plot = Plot::new();
    // plot.set_layout(plotly::Layout::new().y_axis(Axis::new().range(vec![0, 100])));
    plot.add_traces(traces);
    plot.show();

    Ok(())
}
