#![cfg_attr(feature = "release", windows_subsystem = "windows")]
#![allow(incomplete_features, unused_braces)]
#![feature(
    get_mut_unchecked, exhaustive_patterns, associated_type_defaults, never_type, const_trait_impl,
    specialization, const_fn_floating_point_arithmetic, const_option_ext, let_chains, inline_const,
    decl_macro, inline_const_pat, trait_upcasting,
)]

#[allow(unused_imports)]
#[macro_use(vecf, veci, vecu, vecs)]
pub extern crate math_linear;

pub mod app;
pub mod prelude;

pub use app::utils::*;



fn main() -> anyhow::Result<!> {
    app::App::drive()
}