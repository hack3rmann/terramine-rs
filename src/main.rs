#![cfg_attr(feature = "release", windows_subsystem = "windows")]

mod app;

use app::App;

fn main() {
	App::new().run();
}