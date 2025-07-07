#![cfg_attr(feature = "release", windows_subsystem = "windows")]
#![feature(
    get_mut_unchecked,
    exhaustive_patterns,
    associated_type_defaults,
    never_type
)]
#![warn(static_mut_refs)]

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
