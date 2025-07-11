#![allow(dead_code)]

pub mod math;

pub mod prelude {
    pub use super::math::prelude::*;
}

pub use prelude::{vecf, veci, vecs, vecu};
