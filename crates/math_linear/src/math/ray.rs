use crate::prelude::*;

pub mod space_3d {
    use super::*;
    /// Represents mathematical line
    #[derive(Clone, Copy, Debug)]
    pub struct Line {
        pub origin: vec3,
        pub direction: vec3,
    }

    impl Line {
        /// Creates new Line.
        pub fn new(origin: vec3, direction: vec3) -> Self { Line { origin, direction } }

        /// Creates line from 2 points.
        pub fn from_2_points(start: vec3, end: vec3) -> Self {
            Line { origin: start, direction: (end - start).normalized() }
        }

        /// Gives point on the line by signed distance from line origin.
        pub fn point_along(&self, distance: f32) -> vec3 {
            self.origin + distance * self.direction
        }
    }
}

pub mod space_2d {
    use super::*;
    /// Represents mathematical ray
    #[derive(Clone, Copy, Debug)]
    pub struct Line {
        pub origin: vec2,
        pub direction: vec2,
    }

    impl Line {
        /// Creates new ray.
        pub fn new(origin: vec2, direction: vec2) -> Self {
            assert!(direction != vec2::zero(), "Direction vector should have non-zero length!");
            Self { origin, direction }
        }

        /// Creates ray from 2 points.
        pub fn from_2_points(start: vec2, end: vec2) -> Self {
            Self { origin: start, direction: (end - start).normalized() }
        }

        /// Gives point on the ray by signed distance from ray origin.
        pub fn point_along(&self, distance: f32) -> vec2 {
            self.origin + distance * self.direction
        }
    }
}