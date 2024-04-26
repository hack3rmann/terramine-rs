#![cfg_attr(feature = "release", windows_subsystem = "windows")]

#[allow(unused_imports)]
#[macro_use(vecf, veci, vecu, vecs)]
pub extern crate math_linear;

pub mod app;
pub mod prelude;

pub use app::utils::*;



fn main() -> anyhow::Result<()> {
    app::App::drive()
}