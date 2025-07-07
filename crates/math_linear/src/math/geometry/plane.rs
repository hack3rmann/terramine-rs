use crate::prelude::*;

/// Represents a plane
#[derive(Clone, Copy, Debug)]
pub struct Plane {
    pub normal: vec3,
    pub distance: f32,
}

impl Plane {
    /// Constructs plane from origin and normal
    pub fn new(origin: vec3, mut normal: vec3) -> Self {
        normal = normal.normalized();
        Plane {
            normal,
            distance: origin.dot(normal),
        }
    }

    /// Checks if gitven vector is in positive side of plane
    pub fn is_in_positive_side(&self, vec: vec3) -> bool {
        self.signed_distance(vec) >= 0.0
    }

    /// Gives signed distance to this plane
    pub fn signed_distance(&self, vec: vec3) -> f32 {
        vec.dot(self.normal) - self.distance
    }
}
