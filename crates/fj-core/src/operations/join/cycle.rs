use std::ops::RangeInclusive;

use itertools::Itertools;

use crate::{
    geometry::{HalfEdgeGeom, LocalCurveGeom},
    operations::{
        build::BuildHalfEdge,
        geometry::UpdateHalfEdgeGeometry,
        insert::Insert,
        update::{UpdateCycle, UpdateHalfEdge},
    },
    storage::Handle,
    topology::{Cycle, HalfEdge, Surface},
    Core,
};

/// Join a [`Cycle`] to another
pub trait JoinCycle {
    /// Add new half-edges to the cycle that are joined to the provided ones
    ///
    /// This method creates a new half-edge for each half-edge that is provided,
    /// joins the new half-edge to the provided one, and adds the new half-edge
    /// to the cycle.
    ///
    /// The geometry for each new half-edge needs to be provided as well.
    ///
    /// Also requires the surface that the cycle is defined in.
    #[must_use]
    fn add_joined_edges<Es>(
        &self,
        edges: Es,
        surface: Handle<Surface>,
        core: &mut Core,
    ) -> Self
    where
        Es: IntoIterator<Item = (Handle<HalfEdge>, HalfEdgeGeom)>,
        Es::IntoIter: Clone + ExactSizeIterator;

    /// Join the cycle to another
    ///
    /// Joins the cycle to the other at the provided ranges. The ranges specify
    /// the indices of the edges that are joined together.
    ///
    /// A modulo operation is applied to all indices before use, so in a cycle
    /// of 3 edges, indices `0` and `3` refer to the same edge. This allows for
    /// specifying a range that crosses the "seam" of the cycle.
    ///
    /// # Panics
    ///
    /// Panics, if the ranges have different lengths.
    ///
    /// # Assumptions
    ///
    /// This method makes some assumptions that need to be met, if the operation
    /// is to result in a valid shape:
    ///
    /// - **The joined edges must be coincident.**
    /// - **The locally defined curve coordinate systems of the edges must
    ///   match.**
    ///
    /// If either of those assumptions are not met, this will result in a
    /// validation error down the line.
    ///
    /// # Implementation Note
    ///
    /// The use of the `RangeInclusive` type might be a bit limiting, as other
    /// range types might be more convenient in a given use case. This
    /// implementation was chosen for now, as it wasn't clear whether the
    /// additional complexity of using `RangeBounds` would be worth it.
    ///
    /// A solution based on `SliceIndex` could theoretically be used, but that
    /// trait is partially unstable. In addition, it's not clear how it could be
    /// used generically, as it could yield a range (which can be iterated over)
    /// or a single item (which can not). This is not a hard problem in
    /// principle (a single item could just be an iterator of length 1), but I
    /// don't see it how to address this in Rust in a reasonable way.
    ///
    /// Maybe a custom trait that is implemented for `usize` and all range types
    /// would be the best solution.
    #[must_use]
    fn join_to(
        &self,
        other: &Cycle,
        range: RangeInclusive<usize>,
        other_range: RangeInclusive<usize>,
        core: &mut Core,
    ) -> Self;
}

impl JoinCycle for Cycle {
    fn add_joined_edges<Es>(
        &self,
        edges: Es,
        surface: Handle<Surface>,
        core: &mut Core,
    ) -> Self
    where
        Es: IntoIterator<Item = (Handle<HalfEdge>, HalfEdgeGeom)>,
        Es::IntoIter: Clone + ExactSizeIterator,
    {
        let half_edges = edges
            .into_iter()
            .circular_tuple_windows()
            .map(|((prev_half_edge, _), (half_edge, geometry))| {
                let half_edge = HalfEdge::unjoined(core)
                    .update_curve(|_, _| half_edge.curve().clone(), core)
                    .update_start_vertex(
                        |_, _| prev_half_edge.start_vertex().clone(),
                        core,
                    )
                    .insert(core)
                    .set_geometry(geometry, &mut core.layers.geometry);

                core.layers.geometry.define_curve(
                    half_edge.curve().clone(),
                    surface.clone(),
                    LocalCurveGeom {
                        path: geometry.path,
                    },
                );

                half_edge
            })
            .collect::<Vec<_>>();
        self.add_half_edges(half_edges, core)
    }

    fn join_to(
        &self,
        other: &Cycle,
        range: RangeInclusive<usize>,
        range_other: RangeInclusive<usize>,
        core: &mut Core,
    ) -> Self {
        assert_eq!(
            range.end() - range.start(),
            range_other.end() - range_other.start(),
            "Ranges have different lengths",
        );

        range.zip(range_other).fold(
            self.clone(),
            |cycle, (index, index_other)| {
                let edge_other = other.half_edges().nth_circular(index_other);

                cycle
                    .update_half_edge(
                        self.half_edges().nth_circular(index),
                        |half_edge, core| {
                            [half_edge
                                .update_curve(
                                    |_, _| edge_other.curve().clone(),
                                    core,
                                )
                                .update_start_vertex(
                                    |_, _| {
                                        other
                                            .half_edges()
                                            .nth_circular(index_other + 1)
                                            .start_vertex()
                                            .clone()
                                    },
                                    core,
                                )
                                .insert(core)
                                .set_geometry(
                                    *core
                                        .layers
                                        .geometry
                                        .of_half_edge(half_edge),
                                    &mut core.layers.geometry,
                                )]
                        },
                        core,
                    )
                    .update_half_edge(
                        self.half_edges().nth_circular(index + 1),
                        |half_edge, core| {
                            [half_edge
                                .update_start_vertex(
                                    |_, _| edge_other.start_vertex().clone(),
                                    core,
                                )
                                .insert(core)
                                .set_geometry(
                                    *core
                                        .layers
                                        .geometry
                                        .of_half_edge(half_edge),
                                    &mut core.layers.geometry,
                                )]
                        },
                        core,
                    )
            },
        )
    }
}
