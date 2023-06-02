#![cfg_attr(feature = "release", windows_subsystem = "windows")]
#![allow(incomplete_features)]
#![feature(
    generators, generator_trait, get_mut_unchecked, exhaustive_patterns,
    associated_type_defaults, never_type, const_trait_impl, specialization,
    const_fn_floating_point_arithmetic, const_option_ext, let_chains, inline_const,
    decl_macro, rustc_attrs, local_key_cell_methods,
)]



#[allow(unused_imports)]
#[macro_use(vecf, veci, vecu, vecs)]
pub extern crate math_linear;



pub mod app;
pub mod prelude;



pub use app::utils::*;

use app::App;
use runtime::RUNTIME;



fn main() -> anyhow::Result<()> {
    RUNTIME.block_on(App::new())?.run()
}