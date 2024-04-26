#![cfg_attr(feature = "release", windows_subsystem = "windows")]
#![allow(incomplete_features, unused_braces)]
#![feature(
    get_mut_unchecked, exhaustive_patterns, associated_type_defaults, never_type, const_trait_impl,
    decl_macro
)]

#[allow(unused_imports)]
#[macro_use(vecf, veci, vecu, vecs)]
pub extern crate math_linear;

pub mod app;
pub mod prelude;

pub use app::utils::*;



fn main() -> anyhow::Result<()> {
    app::App::drive()
}