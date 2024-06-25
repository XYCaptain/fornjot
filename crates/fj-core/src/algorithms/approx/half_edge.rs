//! Half-edge approximation
//!
//! See [`HalfEdgeApprox`].

use std::iter;

use fj_math::Point;

use crate::{
    geometry::{CurveBoundary, Geometry},
    storage::Handle,
    topology::{HalfEdge, Surface},
};

use super::{
    curve::{approx_curve_with_cache, CurveApproxCache},
    vertex::{approx_vertex, VertexApproxCache},
    ApproxPoint, Tolerance,
};

/// Approximate the provided half-edge
pub fn approx_half_edge(
    half_edge: &Handle<HalfEdge>,
    surface: &Handle<Surface>,
    boundary: CurveBoundary<Point<1>>,
    tolerance: impl Into<Tolerance>,
    vertex_cache: &mut VertexApproxCache,
    curve_cache: &mut CurveApproxCache,
    geometry: &Geometry,
) -> HalfEdgeApprox {
    let tolerance = tolerance.into();

    let [start_position_curve, _] = boundary.inner;

    let start = approx_vertex(
        half_edge.start_vertex().clone(),
        half_edge.curve(),
        surface,
        start_position_curve,
        vertex_cache,
        geometry,
    );

    let rest = approx_curve_with_cache(
        half_edge.curve(),
        surface,
        boundary,
        tolerance,
        curve_cache,
        geometry,
    );

    let points = iter::once(start)
        .chain(rest.points)
        .map(|point| {
            let point_surface = geometry
                .of_curve(half_edge.curve())
                .unwrap()
                .local_on(surface)
                .unwrap()
                .path
                .point_from_path_coords(point.local_form);

            ApproxPoint::new(point_surface, point.global_form)
        })
        .collect();

    HalfEdgeApprox { points }
}

/// An approximation of a [`HalfEdge`]
///
/// The approximation of a half-edge is its first vertex, combined with the
/// approximation of its curve. The second vertex is left out, as half-edge
/// approximations are usually used to build cycle approximations, and this way,
/// the caller doesn't have to deal with duplicate vertices.
#[derive(Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct HalfEdgeApprox {
    /// The points that approximate the half-edge
    pub points: Vec<ApproxPoint<2>>,
}
