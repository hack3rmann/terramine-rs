#![cfg_attr(feature = "release", windows_subsystem = "windows")]

mod app;

use app::App;

fn main() {
	app::utils::profiler::initialyze();
	app::utils::runtime::initialyze();

	App::new().run();
}