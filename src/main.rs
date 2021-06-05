use chrono::prelude::*;
use collecting_hashmap::CollectingHashMap;
use ordered_float::OrderedFloat;
use plotters::prelude::*;
use regex::Regex;
use serde::{Deserialize, Serialize};

use std::cmp::{max, min};
use std::collections::HashMap;
use std::convert::TryInto;
use std::env;
use std::fs::{create_dir_all, read_dir, File};
use std::io::Read;

#[derive(Serialize, Deserialize, Debug)]
struct ProfileJSONEntry {
	name: String,
	unit: Option<String>,
	value: f64,
	calls: i64,
}

#[derive(Debug)]
struct PlotData {
	value: OrderedFloat<f64>,
	calls: i64,
	time: DateTime<FixedOffset>,
}

fn main() {
	let args: Vec<String> = env::args().collect();
	let dir_path = args[1].clone();
	let paths = read_dir(dir_path).expect("Invalid or no directory to read provided.");

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
					"Error encountered while parsing:{} \nFile: {}\n
					Did you use a 514.1554 file and not fix it?",
					fullpath.display(),
					e
				);
				panic!()
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

	// This part is so we can divide all data by the amount of ticks spent in the round, to eliminate round-length bias
	let sendmaps_data = data_to_plot.get_all("SendMaps").unwrap(); // Guarenteed to exist in valid map profiling data
	let mut round_ticks_map = HashMap::new();

	// We also need to get the minimum round ticks for the plot y axis
	let mut min_round_ticks = i64::MAX;

	for data in sendmaps_data {
		round_ticks_map.insert(data.time, data.calls as f64); // Convert here for later division
		min_round_ticks = min(min_round_ticks, data.calls);
	}

	create_dir_all("output").expect("Unable to create `output/` dir.");
	for data in data_to_plot.iter() {
		let graph_name = data.0;
		let graph_path = format!("output/{}.png", sanitize_filename::sanitize(graph_name));

		// Get maximum value out of our possible logs for max y graph bound
		let mut max_data_value: OrderedFloat<f64> = OrderedFloat(0_f64);
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

		// Divide by the minimum round ticks we found earlier
		let max_data_value_adj = f64::from(max_data_value) / min_round_ticks as f64;

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
					// Autoscale the text on top so it looks nice
					num::clamp(
						85_i64.wrapping_sub(graph_name.len().try_into().unwrap()),
						25,
						50,
					) as i32,
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
				0_f64..(max_data_value_adj - (max_data_value_adj / 1.4_f64)), // Manually adjusted
			)
			.unwrap();

		// Draw grid
		chart
			.configure_mesh()
			.y_desc("Seconds/Ticks")
			.x_desc("Roundend in UNIX time")
			.draw()
			.expect("Draw failure");

		//Draw our actual data points
		chart
			.draw_series(data.1.iter().map(|data| {
				Circle::new(
					(
						data.time.timestamp(),
						get_tick_adjusted_value(data, &round_ticks_map),
					),
					3,
					RED.filled(),
				)
			}))
			.unwrap();

		// Draw connecting line
		chart
			.draw_series(LineSeries::new(
				data.1.iter().map(|data| {
					(
						data.time.timestamp(),
						get_tick_adjusted_value(data, &round_ticks_map),
					)
				}),
				&RED,
			))
			.unwrap();

		root_area.present().expect("Unable to write result to file, please make sure 'output' dir exists under current dir");
	}
	println!("Finished: results have been saved to `output/`")
}

/// Given data and a hashmap of time->total_ticks, adjusts the data by dividing it
/// This is to compensate for round length in plotting data
fn get_tick_adjusted_value(data: &PlotData, tick_map: &HashMap<DateTime<FixedOffset>, f64>) -> f64 {
	*data.value / (*(tick_map.get(&data.time).unwrap_or(&1.0)))
}
