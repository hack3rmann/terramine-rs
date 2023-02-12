pub mod vector;
pub mod matrix;
pub mod ranges;

pub mod prelude {
    pub use super::{
        vector::{swizzle::*, *},
        matrix::*,
        ranges as rges,
    };
}