use collecting_hashmap::CollectingHashMap;
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};
use std::cmp::max;
use std::fs::File;
use std::io::prelude::*;
use std::{env, fs};
use plotters::prelude::*;

#[deny(clippy::pedantic)]


#[derive(Serialize, Deserialize, Debug)]
struct ProfileJSONEntry {
	name: String,
	unit: Option<String>,
	value: f32,
	calls: i32,
}

#[derive(Debug)]
struct PlotData {
	value: OrderedFloat<f32>,
	calls: i32,
}

fn main() {
	let args: Vec<String> = env::args().collect();
	let dir_path = args[1].clone();
	let paths = fs::read_dir(dir_path).expect("Invalid or no directory to read provided.");

	// Reading files
	let mut data_to_plot: CollectingHashMap<String, PlotData> = CollectingHashMap::new();

	for path in paths {
		let mut file = File::open(path.unwrap().path()).expect("File could not be opened");
		let mut contents = String::new();
		file.read_to_string(&mut contents)
			.expect("Error reading file");

		// Deserialize
		let data: Vec<ProfileJSONEntry> =
			serde_json::from_str(&contents).expect("Error parsing file");

		for entry in data {
			data_to_plot.insert(
				entry.name,
				PlotData {
					value: OrderedFloat(entry.value),
					calls: entry.calls,
				},
			);
		}
	}

	for data in data_to_plot.iter() {
		let graph_name = data.0;
		let graph_path = format!("output/{}.png", sanitize_filename::sanitize(graph_name));
		println!("{}", graph_name);

		// Get maximum value out of our possible logs for max y graph bound
		let mut max_data_value: OrderedFloat<f32> = OrderedFloat(0f32);
		for data in data.1.iter() {
			max_data_value = max(max_data_value, data.value);
		}

		// Setting up graph
		let root_area = BitMapBackend::new(&graph_path, (700, 500)).into_drawing_area();

		// Color the background white
		root_area.fill(&WHITE).expect("Couldn't color the drawing.");

		// Create chart elements
		let mut chart = ChartBuilder::on(&root_area)
			.caption("BYOND Map Profiler", ("sans-serif", 50).into_font())
			.margin(5)
			.x_label_area_size(40)
			.y_label_area_size(40)
			.build_cartesian_2d(0f32..50f32, 0f32..10f32)
			.unwrap();

		// Draw grid
		chart.configure_mesh().draw().expect("Draw failure");

		// Draw our actual data
		chart
			.draw_series(LineSeries::new(
				(-50..=50).map(|x| x as f32 / 50.0).map(|x| (x, x * x)),
				&RED,
			))
			.unwrap()
			.label(graph_name)
			.legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));

		chart
			.configure_series_labels()
			.background_style(&WHITE.mix(0.8))
			.border_style(&BLACK)
			.draw()
			.unwrap();
	}
}
