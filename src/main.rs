#![cfg_attr(feature = "release", windows_subsystem = "windows")]
#![allow(dead_code)]

#[allow(unused_imports)]
#[macro_use(vecf, veci, vecu, vecs)]
pub extern crate math_linear;

mod app;

use app::App;

fn main() {
    app::utils::profiler::initialyze();
    app::utils::runtime::initialyze();

    App::new().run();
}