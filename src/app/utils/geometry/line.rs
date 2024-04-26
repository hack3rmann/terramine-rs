use glam::*;
use serde::{Serialize, Deserialize};



/// Represents mathematical line
#[derive(Clone, Copy, Debug)]
#[derive(Serialize, Deserialize)]
pub struct Line3d {
    pub origin: Vec3,
    pub direction: Vec3,
}

impl Line3d {
    /// Creates new Line.
    pub fn new(origin: Vec3, direction: Vec3) -> Self {
        assert!(direction.is_normalized(), "direction vector should be normalized");
        Self { origin, direction }
    }

    /// Creates line from 2 points.
    pub fn from_2_points(start: Vec3, end: Vec3) -> Self {
        Self { origin: start, direction: (end - start).normalize_or(Vec3::X) }
    }

    /// Gives point on the line by signed distance from line origin.
    pub fn point_along(&self, distance: f32) -> Vec3 {
        self.origin + distance * self.direction
    }
}

impl Default for Line3d {
    fn default() -> Self {
        Self { origin: Vec3::ZERO, direction: Vec3::X }
    }
}



/// Represents mathematical ray
#[derive(Clone, Copy, Debug)]
#[derive(Serialize, Deserialize)]
pub struct Line2d {
    pub origin: Vec2,
    pub direction: Vec2,
}

impl Line2d {
    /// Creates new ray.
    pub fn new(origin: Vec2, direction: Vec2) -> Self {
        assert!(direction.is_normalized(), "direction vector should be normalized");
        Self { origin, direction }
    }

    /// Creates ray from 2 points.
    pub fn from_2_points(start: Vec2, end: Vec2) -> Self {
        Self { origin: start, direction: (end - start).normalize_or(Vec2::X) }
    }

    /// Gives point on the ray by signed distance from ray origin.
    pub fn point_along(&self, distance: f32) -> Vec2 {
        self.origin + distance * self.direction
    }
}

impl Default for Line2d {
    fn default() -> Self {
        Self { origin: Vec2::ZERO, direction: Vec2::X }
    }
}