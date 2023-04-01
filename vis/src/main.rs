extern crate plotly;
use plotly::common::Mode;
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
    for (_, time) in &mut data {
        *time -= min_time;
    }

    let audio_requested_series: Vec<u64> = data
        .iter()
        .filter_map(|(event, time)| (event == "audio_requested").then(|| *time))
        .collect();
    let mut max_gap = 0;
    for i in 1..audio_requested_series.len() {
        max_gap = std::cmp::max(max_gap, audio_requested_series[i] - audio_requested_series[i - 1]);
    }
    dbg!(max_gap);

    let trace = |event, y| {
        Scatter::new(
            data.iter()
                .filter_map(|(e, time)| (e == event).then(|| *time as f32))
                .collect(),
            data.iter()
                .filter_map(|(e, _)| (e == event).then(|| y))
                .collect(),
        )
        .name(event)
        .mode(Mode::Markers)
    };

    let traces: Vec<Box<dyn Trace + 'static>> = vec![

        trace("refresher_lock", 0.0),
        trace("refresher_start", 1.0),
        trace("frame_ready", 2.0),
        trace("refresher_end", 3.0),

        trace("audio_lock", 5.0),
        trace("audio_start", 6.0),
        trace("audio_end", 7.0),

        trace("window_start", 10.0),
        trace("window_lock", 11.0),
        trace("window_locked", 12.0),
        trace("window_end", 13.0),

        trace("frame_repeated", 15.0),
        trace("frame_skipped", 16.0),
    ];

    let mut plot = Plot::new();
    plot.add_traces(traces);
    plot.show();

    Ok(())
}
