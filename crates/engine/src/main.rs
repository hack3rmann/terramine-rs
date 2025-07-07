#![cfg_attr(feature = "release", windows_subsystem = "windows")]

#[allow(unused_imports)]
#[macro_use(vecf, veci, vecu, vecs)]
pub extern crate math_linear;

pub mod app;
pub mod prelude;

use app::{App, utils::runtime::RUNTIME};

pub use app::utils::*;

fn main() {
    // FIXME(hack3rmann): support unix
    // app::utils::werror::set_panic_hook();

    RUNTIME.block_on(App::new()).run().unwrap();
}
