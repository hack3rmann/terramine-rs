#![cfg_attr(feature = "release", windows_subsystem = "windows")]

#[allow(unused_imports)]
#[macro_use(vecf, veci, vecu, vecs)]
pub extern crate math_linear;

mod app;

use app::App;

fn main() {
    app::utils::werror::set_panic_hook();
    app::utils::profiler::initialize();
    app::utils::runtime::initialize();

    App::new().run();
}