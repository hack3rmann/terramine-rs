#![macro_use]

pub mod vector;
pub mod matrix;
pub mod bounding_volumes;
pub mod ray;
pub mod space_index;
pub mod geometry;

pub mod prelude {
    pub use super::{
        vector::*,
        matrix::*,
        bounding_volumes::aabb::*,
        ray::*,
        space_index as sdex,
        geometry::{
            angle::*,
            plane::*,
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