#![macro_use]

pub mod bounding_volumes;
pub mod geometry;
pub mod matrix;
pub mod ray;
pub mod space_index;
pub mod vector;

pub mod prelude {
    pub use super::{
        bounding_volumes::aabb::*,
        geometry::{angle::*, plane::*},
        matrix::*,
        ray::*,
        space_index as sdex,
        vector::{
            macros::{vecf, veci, vecs, vecu},
            *,
        },
    };

    #[allow(non_camel_case_types)]
    pub type vec1 = f32;

    #[allow(non_camel_case_types)]
    pub type vec2 = Float2;

    #[allow(non_camel_case_types)]
    pub type vec3 = Float3;

    #[allow(non_camel_case_types)]
    pub type vec4 = Float4;

    #[allow(non_camel_case_types)]
    pub type mat4 = FloatMatrix4x4;
}
