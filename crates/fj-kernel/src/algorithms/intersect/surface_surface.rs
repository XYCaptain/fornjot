use fj_math::{Line, Plane, Point, Scalar};

use crate::{
    objects::{Curve, GlobalCurve, Surface},
    path::{GlobalPath, SurfacePath},
    stores::Stores,
};

/// The intersection between two surfaces
#[derive(Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct SurfaceSurfaceIntersection {
    /// The intersection curves
    pub intersection_curves: [Curve; 2],
}

impl SurfaceSurfaceIntersection {
    /// Compute the intersection between two surfaces
    pub fn compute(surfaces: [&Surface; 2], stores: &Stores) -> Option<Self> {
        // Algorithm from Real-Time Collision Detection by Christer Ericson. See
        // section 5.4.4, Intersection of Two Planes.
        //
        // Adaptations were made to get the intersection curves in local
        // coordinates for each surface.

        let surfaces_and_planes = surfaces.map(|surface| {
            let plane = plane_from_surface(surface);
            (*surface, plane)
        });
        let [a, b] = surfaces_and_planes.map(|(_, plane)| plane);

        let (a_distance, a_normal) = a.constant_normal_form();
        let (b_distance, b_normal) = b.constant_normal_form();

        let direction = a_normal.cross(&b_normal);

        let denom = direction.dot(&direction);
        if denom == Scalar::ZERO {
            // Comparing `denom` against zero looks fishy. It's probably better
            // to compare it against an epsilon value, but I don't know how
            // large that epsilon should be.
            //
            // I'll just leave it like that, until we had the opportunity to
            // collect some experience with this code.
            // - @hannobraun
            return None;
        }

        let origin = (b_normal * a_distance - a_normal * b_distance)
            .cross(&direction)
            / denom;
        let origin = Point { coords: origin };

        let line = Line::from_origin_and_direction(origin, direction);

        let curves = surfaces_and_planes.map(|(surface, plane)| {
            let path = SurfacePath::Line(plane.project_line(&line));
            let global_form = GlobalCurve::new(stores);

            Curve::new(surface, path, global_form)
        });

        Some(Self {
            intersection_curves: curves,
        })
    }
}

fn plane_from_surface(surface: &Surface) -> Plane {
    let (line, path) = {
        let line = match surface.u() {
            GlobalPath::Line(line) => line,
            _ => todo!("Only plane-plane intersection is currently supported."),
        };

        (line, surface.v())
    };

    Plane::from_parametric(line.origin(), line.direction(), path)
}

#[cfg(test)]
mod tests {
    use fj_math::Transform;

    use crate::{
        algorithms::transform::TransformObject,
        objects::{Curve, Surface},
        partial::HasPartial,
        stores::Stores,
    };

    use super::SurfaceSurfaceIntersection;

    #[test]
    fn plane_plane() {
        let stores = Stores::new();

        let xy = Surface::xy_plane();
        let xz = Surface::xz_plane();

        // Coincident and parallel planes don't have an intersection curve.
        assert_eq!(
            SurfaceSurfaceIntersection::compute(
                [
                    &xy,
                    &xy.transform(
                        &Transform::translation([0., 0., 1.],),
                        &stores
                    )
                ],
                &stores
            ),
            None,
        );

        let expected_xy =
            Curve::partial().with_surface(xy).as_u_axis().build(&stores);
        let expected_xz =
            Curve::partial().with_surface(xz).as_u_axis().build(&stores);

        assert_eq!(
            SurfaceSurfaceIntersection::compute([&xy, &xz], &stores),
            Some(SurfaceSurfaceIntersection {
                intersection_curves: [expected_xy, expected_xz],
            })
        );
    }
}
