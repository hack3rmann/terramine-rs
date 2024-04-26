use glam::*;
use serde::{Serialize, Deserialize};



/// Represents a plane
#[derive(Clone, Copy, Debug)]
#[derive(Serialize, Deserialize)]
pub struct Plane3d {
    pub normal: Vec3,
    pub distance: f32,
}

impl Plane3d {
    /// Constructs plane from origin and normal
    pub fn new(origin: Vec3, normal: Vec3) -> Self {
        assert!(normal.is_normalized(), "normal should be normalized");
        Plane3d { normal, distance: origin.dot(normal) }
    }

    /// Checks if gitven vector is in positive side of plane
    pub fn is_in_positive_side(&self, vec: Vec3) -> bool {
        self.signed_distance(vec) >= 0.0
    }

    /// Gives signed distance to this plane
    pub fn signed_distance(&self, vec: Vec3) -> f32 {
        vec.dot(self.normal) - self.distance
    }
}