use fj_math::{Point, Scalar};
use itertools::Itertools;

use crate::{
    geometry::Geometry,
    queries::AllHalfEdgesWithSurface,
    storage::Handle,
    topology::{HalfEdge, Shell},
    validation::{ValidationCheck, ValidationConfig},
};

/// # [`Shell`] contains [`Curve`] with contradicting geometry definitions
///
/// Curve geometry is defined locally, in the 2D coordinates of a surface. A
/// curve can be on multiple surfaces, where those intersect, and these local
/// definitions exist for all surfaces that a curve is on.
///
/// This means that multiple redundant definitions might exist for each curve.
/// This validation check makes sure that these definitions match.
///
/// ## Implementation Note
///
/// That multiple redundant definitions exist, is undesirable in the first
/// place. However, we can't just use one global definition in 3D, as we need
/// the local 2D definitions to approximate and triangulate curves, and we
/// currently don't have the tools to project a global definition into a local
/// context.
///
/// Eventually, it should be possible to define the geometry of a curve once,
/// either locally or globally, and then convert that single definition into
/// (other) local contexts, as needed. There currently is no issue to track that
/// specifically, but there is the following issue, which is a prerequisite for
/// making the required tooling practical:
///
/// <https://github.com/hannobraun/fornjot/issues/2118>
///
/// [`Curve`]: crate::topology::Curve
#[derive(Clone, Debug, thiserror::Error)]
#[error("Curve coordinate system mismatch: {:#?}", self)]
pub struct CurveGeometryMismatch {
    /// One of the half-edges, whose curves have mismatching geometry
    pub half_edge_a: Handle<HalfEdge>,

    /// The other of the half-edges, whose curves have mismatching geometry
    pub half_edge_b: Handle<HalfEdge>,

    /// The point on the curves, where they don't match
    pub point_curve: Point<1>,

    /// The same point in 3D coordinates, according to `half_edge_a`'s curve
    pub point_a: Point<3>,

    /// The same point in 3D coordinates, according to `half_edge_b`'s curve
    pub point_b: Point<3>,

    /// The distance between those 3D coordinates
    pub distance: Scalar,
}

impl ValidationCheck<Shell> for CurveGeometryMismatch {
    fn check<'r>(
        object: &'r Shell,
        geometry: &'r Geometry,
        config: &'r ValidationConfig,
    ) -> impl Iterator<Item = Self> + 'r {
        let edges_and_surfaces =
            object.all_half_edges_with_surface().collect::<Vec<_>>();

        edges_and_surfaces
            .clone()
            .into_iter()
            .cartesian_product(edges_and_surfaces)
            .filter_map(|((edge_a, surface_a), (edge_b, surface_b))| {
                // We only care about edges referring to the same curve.
                if edge_a.curve().id() != edge_b.curve().id() {
                    return None;
                }

                // No need to check an edge against itself.
                if edge_a.id() == edge_b.id() {
                    return None;
                }

                let surface_a = geometry.of_surface(&surface_a);
                let surface_b = geometry.of_surface(&surface_b);

                // Let's check 4 points. Given that the most complex curves we
                // have right now are circles, 3 would be enough to check for
                // coincidence. But the first and last might be identical, so
                // let's add an extra one.
                let [a, d] = geometry.of_half_edge(&edge_a).boundary.inner;
                let b = a + (d - a) * 1. / 3.;
                let c = a + (d - a) * 2. / 3.;

                let mut errors: Vec<Self> = Vec::new();

                for point_curve in [a, b, c, d] {
                    let a_surface = geometry
                        .of_half_edge(&edge_a)
                        .path
                        .point_from_path_coords(point_curve);
                    let b_surface = geometry
                        .of_half_edge(&edge_b)
                        .path
                        .point_from_path_coords(point_curve);

                    let a_global =
                        surface_a.point_from_surface_coords(a_surface);
                    let b_global =
                        surface_b.point_from_surface_coords(b_surface);

                    let distance = (a_global - b_global).magnitude();

                    if distance > config.identical_max_distance {
                        errors.push(Self {
                            half_edge_a: edge_a.clone(),
                            half_edge_b: edge_b.clone(),
                            point_curve,
                            point_a: a_global,
                            point_b: b_global,
                            distance,
                        });
                    }
                }

                Some(errors)
            })
            .flatten()
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        operations::{
            build::BuildShell,
            geometry::UpdateHalfEdgeGeometry,
            insert::Insert,
            update::{UpdateCycle, UpdateFace, UpdateRegion, UpdateShell},
        },
        topology::{HalfEdge, Shell},
        validation::{checks::CurveGeometryMismatch, ValidationCheck},
        Core,
    };

    #[test]
    fn curve_geometry_mismatch() -> anyhow::Result<()> {
        let mut core = Core::new();

        let valid = Shell::tetrahedron(
            [[0., 0., 0.], [0., 1., 0.], [1., 0., 0.], [0., 0., 1.]],
            &mut core,
        );
        CurveGeometryMismatch::check_and_return_first_error(
            &valid.shell,
            &core.layers.geometry,
        )?;

        let invalid = valid.shell.update_face(
            &valid.abc.face,
            |face, core| {
                [face.update_region(
                    |region, core| {
                        region.update_exterior(
                            |cycle, core| {
                                cycle.update_half_edge(
                                    cycle.half_edges().nth_circular(0),
                                    |half_edge, core| {
                                        let mut geometry = *core
                                            .layers
                                            .geometry
                                            .of_half_edge(half_edge);
                                        geometry.path = geometry.path.reverse();
                                        geometry.boundary =
                                            geometry.boundary.reverse();

                                        [HalfEdge::new(
                                            half_edge.curve().clone(),
                                            half_edge.start_vertex().clone(),
                                        )
                                        .insert(core)
                                        .set_geometry(
                                            geometry,
                                            &mut core.layers.geometry,
                                        )]
                                    },
                                    core,
                                )
                            },
                            core,
                        )
                    },
                    core,
                )]
            },
            &mut core,
        );
        assert!(CurveGeometryMismatch::check_and_return_first_error(
            &invalid,
            &core.layers.geometry,
        )
        .is_err());

        Ok(())
    }
}
