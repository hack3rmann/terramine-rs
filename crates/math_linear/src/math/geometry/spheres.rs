use crate::{
    math::geometry::{Containment, Intersect, space_2d::Line},
    prelude::*,
};

#[derive(Clone, Copy, Debug)]
struct Circle {
    centre: vec2,
    radius: f32,
}

impl Circle {
    pub fn new(centre: vec2, radius: f32) -> Self {
        assert!(radius > 0.0, "Negative radiuses ({radius}) are unsensible!");
        Self { centre, radius }
    }

    /// Checks if given point is surrounded by circle.
    /// *Note*: boundary included.
    pub fn contains(&self, point: Float2) -> bool {
        match self.containment(point, 1e-6) {
            Containment::Contains | Containment::OnBoundary => true,
            Containment::Outside => false,
        }
    }

    /// Checks if given point is within a circle
    pub fn containment(&self, point: vec2, eps: f32) -> Containment {
        let distance = (self.centre - point).len();

        if distance <= self.radius - eps {
            Containment::Contains
        } else if distance >= self.radius + eps {
            Containment::Outside
        } else {
            Containment::OnBoundary
        }
    }
}

impl Intersect<Line> for Circle {
    fn intersect(&self, line: &Line) -> Option<Vec<vec2>> {
        let dir_sqr = line.direction.sqr();
        let diff = line.direction.dot(self.centre - line.origin);
        let sum = line.origin.sqr() + self.centre.sqr() - self.radius * self.radius;
        let discriminant = diff * diff - dir_sqr * sum;

        use std::cmp::Ordering;
        match discriminant.partial_cmp(&0.0)? {
            Ordering::Less => None,
            Ordering::Equal => Some(vec![line.point_along(diff / dir_sqr)]),
            Ordering::Greater => {
                let discr_sqrt = discriminant.sqrt();
                Some(vec![
                    line.point_along((diff + discr_sqrt) / dir_sqr),
                    line.point_along((diff - discr_sqrt) / dir_sqr),
                ])
            }
        }
    }
}

impl Intersect<Circle> for Circle {
    fn intersect(&self, other: &Circle) -> Option<Vec<vec2>> {
        let centre_dist = (self.centre - other.centre).len();
        let line_normal = (self.centre - other.centre) / centre_dist;
        let line = Line {
            origin: line_normal * (self.radius.powi(2) - other.radius.powi(2))
                / (2.0 * centre_dist),
            direction: line_normal.rotate_clockwise(),
        };

        self.intersect(&line)
    }
}

impl Intersect<Circle> for Line {
    fn intersect(&self, obj: &Circle) -> Option<Vec<vec2>> {
        obj.intersect(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circle_intersections() {
        let circle1 = Circle::new(vecf!(-3, 0), 5.0);
        let circle2 = Circle::new(vecf!(3, 0), 5.0);

        assert_eq!(
            circle1.intersect(&circle2).unwrap(),
            [vecf!(0, 4), vecf!(0, -4)],
        )
    }
}
