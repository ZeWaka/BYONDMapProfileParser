use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::prelude::*;
use std::{env, fs};

use plotters::prelude::*;

#[derive(Default, Serialize, Deserialize, Debug)]
struct ProfileJSONEntry {
    name: String,
    unit: Option<String>,
    value: f32,
    calls: i32,
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let dir_path = args[1].clone();
	println!("{}", dir_path);
    let paths = fs::read_dir(dir_path).expect("Invalid or no directory to read provided.");

    // Setting up graph
    let root_area = BitMapBackend::new("output.png", (700, 500)).into_drawing_area();

	// Color the background white
    root_area
        .fill(&WHITE)
        .expect("Couldn't color the drawing.");

    let mut chart = ChartBuilder::on(&root_area)
        .caption("BYOND Map Profiler", ("sans-serif", 50).into_font())
        .margin(5)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_cartesian_2d(0f32..50f32, 0f32..20000f32)
        .unwrap();

    chart.configure_mesh().draw().expect("Draw failure");

    chart
        .draw_series(LineSeries::new(
            (-50..=50).map(|x| x as f32 / 50.0).map(|x| (x, x * x)),
            &RED,
        )).unwrap()
        .label("y = x^2")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));

    chart
        .configure_series_labels()
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .draw()
		.unwrap();

    // Reading files
    for path in paths {
        let mut file = File::open(path.unwrap().path()).expect("File could not be read");
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .expect("Error reading file");
        let data: Vec<ProfileJSONEntry> = serde_json::from_str(&contents).expect("Error parsing file");

        for entry in data {
            println!("Value: {}", entry.value);
        }
    }
}
