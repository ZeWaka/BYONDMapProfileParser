use chrono::prelude::*;
use collecting_hashmap::CollectingHashMap;
use ordered_float::OrderedFloat;
use plotters::prelude::*;
use regex::Regex;
use serde::{Deserialize, Serialize};

use std::cmp::max;
use std::cmp::min;
use std::fs::File;
use std::io::prelude::*;
use std::{env, fs};

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
	time: DateTime<FixedOffset>,
}

fn main() {
	let args: Vec<String> = env::args().collect();
	let dir_path = args[1].clone();
	let paths = fs::read_dir(dir_path).expect("Invalid or no directory to read provided.");

	// Reading files
	let mut data_to_plot: CollectingHashMap<String, PlotData> = CollectingHashMap::new();

	for path in paths {
		let fullpath = path.unwrap().path();
		let mut file = File::open(fullpath.clone()).expect("File could not be opened");
		let mut contents = String::new();
		file.read_to_string(&mut contents)
			.expect("Error reading file");

		// Deserialize
		let data: Option<Vec<ProfileJSONEntry>> = match serde_json::from_str(&contents) {
			Ok(x) => x,
			Err(e) => {
				println!(
					"Error encountered while parsing:{} \nFile: {}",
					fullpath.display(),
					e
				);
				None
			}
		};

		for entry in data.unwrap() {
			// Parse date from the file name, for later graphing
			let mut date_str = fullpath
				.with_extension("") // Remove extension
				.file_name()
				.unwrap()
				.to_str()
				.unwrap()
				.to_owned();

			// Comply with RFC3339
			let re = Regex::new(r" (\d{2})-(\d{2})-(\d{2})-ingame").unwrap();
			date_str = re.replace(&date_str, "T${1}:${2}:${3}-05:00").to_string();

			// Put our data into our storage
			data_to_plot.insert(
				entry.name,
				PlotData {
					value: OrderedFloat(entry.value),
					calls: entry.calls,
					time: date_str.parse::<DateTime<FixedOffset>>().unwrap(),
				},
			);
		}
	}

	println!("Drawing graphs...");
	fs::create_dir_all("output").expect("Unable to create `output/` dir.");
	for data in data_to_plot.iter() {
		let graph_name = data.0;
		let graph_path = format!("output/{}.png", sanitize_filename::sanitize(graph_name));

		// Get maximum value out of our possible logs for max y graph bound
		let mut max_data_value: OrderedFloat<f32> = OrderedFloat(0_f32);
		for data in data.1.iter() {
			max_data_value = max(max_data_value, data.value);
		}

		// Get min/max time out of our possible logs for min/max x graph bound
		let mut min_time = i64::MAX;
		let mut max_time = 0;
		for data in data.1.iter() {
			min_time = min(min_time, data.time.timestamp());
			max_time = max(max_time, data.time.timestamp());
		}

		// Setting up graph
		let root_area = BitMapBackend::new(&graph_path, (1000, 500)).into_drawing_area();

		// Color the background white
		root_area.fill(&WHITE).expect("Couldn't color the drawing.");

		// Create chart elements
		let mut chart = ChartBuilder::on(&root_area)
			.caption(
				graph_name,
				(
					FontFamily::Serif,
					num::clamp(85 - graph_name.len(), 20, 50) as i32,
				)
					.into_font(),
			)
			.margin(20)
			.margin_right(30)
			.x_label_area_size(50)
			.y_label_area_size(50)
			// Give some room on the edges of the bounds
			.build_cartesian_2d(
				(min_time - 3600)..(max_time + 3600),
				0_f32..(f32::from(max_data_value) + (f32::from(max_data_value) / 20.0)),
			)
			.unwrap();

		// Draw grid
		chart
			.configure_mesh()
			.y_desc("Seconds")
			.x_desc("Roundend in UNIX time")
			.draw()
			.expect("Draw failure");

		//Draw our actual data points
		chart
			.draw_series(
				data.1
					.iter()
					.map(|data| Circle::new((data.time.timestamp(), *data.value), 3, RED.filled())),
			)
			.unwrap();

		// Draw connecting line
		chart
			.draw_series(LineSeries::new(
				data.1
					.iter()
					.map(|data| (data.time.timestamp(), *data.value)),
				&RED,
			))
			.unwrap();

		root_area.present().expect("Unable to write result to file, please make sure 'output' dir exists under current dir");
	}
	println!("Finished: results have been saved to `output/`")
}
