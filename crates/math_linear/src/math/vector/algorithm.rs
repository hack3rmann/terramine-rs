pub mod planar {
    use crate::prelude::*;

    /// Chooses most bottom-left vertex then sorts by angle.
    /// If slice is empty the [`None`] is returned.
    pub fn get_bottom_left(vertices: &[vec2]) -> Option<vec2> {
        use std::cmp::Ordering::*;
        vertices
            .iter()
            .min_by(|lhs, rhs| match lhs.y.partial_cmp(&rhs.y).unwrap() {
                Equal => lhs.x.partial_cmp(&rhs.x).unwrap(),
                other => other,
            })
            .cloned()
    }

    pub fn sort_radially(vertices: &mut [vec2]) -> Option<()> {
        let bottom_left = get_bottom_left(vertices)?;

        vertices.sort_by(|&p1, &p2| {
            // NOTE(hack3ramnn): rhs.cmp(lhs) due to cross product sign.
            use std::cmp::Ordering::*;
            let (c_sin1, c_sin2) = (bottom_left.cross(p1), bottom_left.cross(p2));
            match c_sin2.partial_cmp(&c_sin1).unwrap_or(Equal) {
                Equal => {
                    let (len1, len2) = ((p1 - bottom_left).len(), (p2 - bottom_left).len());
                    len1.partial_cmp(&len2).unwrap()
                }
                other => other,
            }
        });

        Some(())
    }

    /// Computes convex boundary of given vertex set on a plane.
    ///
    /// # Safety
    ///
    /// - vertex slice has to be sorted in angle order and first vertex
    ///   should be most left-down.
    /// - vertex slice should contain at least 4 elements.
    pub unsafe fn compute_convex_boundary_unchecked(vertices: &[vec2]) -> Vec<vec2> {
        let mut stack = Vec::with_capacity(vertices.len());

        for _ in 0..vertices.len() {
            todo!()
        }

        stack.shrink_to_fit();
        stack
    }

    /// Computes convex boundary of given vertex list on a plane.
    /// ? Note: firstly sorts list by an angle.
    pub fn compute_convex_boundary(vertices: &[vec2]) -> Vec<vec2> {
        if vertices.len() <= 3 {
            return Vec::from(vertices);
        }

        let mut vertices = Vec::from(vertices);
        sort_radially(&mut vertices);

        // Safety: safe due to check and sort above. See [`compute_convex_boundary_unchecked(&[vec])`]
        unsafe { compute_convex_boundary_unchecked(&vertices) }
    }

    fn circle_tangent_points(centre: vec2, radius: f32, point: vec2) -> (vec2, vec2) {
        let dist = (centre - point).len();
        let to_circle_dir = (centre - point) / dist;

        let proj = dist - radius * radius / dist;
        let height = ((dist - proj) * proj).sqrt();

        (
            to_circle_dir * proj + to_circle_dir.rotate_clockwise() * height,
            to_circle_dir * proj - to_circle_dir.rotate_clockwise() * height,
        )
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::prelude::vecf;

        const LL: vec2 = vecf!(-1, -1);
        const LH: vec2 = vecf!(-1, 1);
        const HH: vec2 = vecf!(1, 1);
        const HL: vec2 = vecf!(1, -1);

        #[test]
        fn test_min_point() {
            let vertices = [LH, HH, LL, HL];
            let min_vertex = get_bottom_left(&vertices).unwrap();
            assert_eq!(min_vertex, LL);
        }

        #[test]
        fn print_radial_sort() {
            let range = -3..=3;
            let size = range.clone().count();
            let mut vertices = Vec::with_capacity(size * size);
            for x in range.clone() {
                for y in range.clone() {
                    vertices.push(vecf!(x, y));
                }
            }

            sort_radially(&mut vertices);

            println!("{vertices:?}");
        }

        #[test]
        fn test_radial_sort() {
            let mut vertices = [
                vecf!(-2, 1),
                vecf!(-2, 0),
                vecf!(-2, -1),
                vecf!(-1, 2),
                vecf!(0, 2),
                vecf!(1, 2),
                vecf!(2, 1),
                vecf!(2, 0),
                vecf!(2, -1),
                vecf!(-1, -2),
                vecf!(0, -2),
                vecf!(1, -2),
            ];

            let left_bottom = get_bottom_left(&vertices).unwrap();
            println!("{left_bottom}");

            sort_radially(&mut vertices);

            println!("{vertices:?}");
        }
    }
}
