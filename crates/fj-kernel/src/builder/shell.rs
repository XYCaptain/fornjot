use fj_math::Scalar;

use crate::{
    algorithms::transform::TransformObject,
    objects::{
        Curve, Cycle, Face, HalfEdge, Objects, Shell, Surface, SurfaceVertex,
        Vertex,
    },
    partial::HasPartial,
    storage::Handle,
};

/// API for building a [`Shell`]
///
/// Also see [`Shell::builder`].
pub struct ShellBuilder<'a> {
    /// The stores that the created objects are put in
    pub objects: &'a Objects,
}

impl<'a> ShellBuilder<'a> {
    /// Create a cube from the length of its edges
    pub fn build_cube_from_edge_length(
        self,
        edge_length: impl Into<Scalar>,
    ) -> Shell {
        let edge_length = edge_length.into();

        // Let's define some short-hands. We're going to need them a lot.
        const Z: Scalar = Scalar::ZERO;
        let h = edge_length / 2.;

        let bottom = {
            let surface = self
                .objects
                .surfaces
                .xy_plane()
                .translate([Z, Z, -h], self.objects);

            Face::builder(self.objects, surface)
                .with_exterior_polygon_from_points([
                    [-h, -h],
                    [h, -h],
                    [h, h],
                    [-h, h],
                ])
                .build()
        };

        let (sides, top_edges) = {
            let surfaces = bottom
                .exterior()
                .half_edges()
                .map(|half_edge| {
                    let [a, b] = half_edge
                        .vertices()
                        .clone()
                        .map(|vertex| vertex.global_form().position());
                    let c = a + [Z, Z, edge_length];

                    self.objects
                        .surfaces
                        .insert(Surface::plane_from_points([a, b, c]))
                })
                .collect::<Vec<_>>();

            let bottoms = bottom
                .exterior()
                .half_edges()
                .zip(&surfaces)
                .map(|(half_edge, surface)| {
                    Handle::<HalfEdge>::partial()
                        .with_surface(Some(surface.clone()))
                        .with_global_form(Some(half_edge.global_form().clone()))
                        .as_line_segment_from_points([[Z, Z], [edge_length, Z]])
                        .build(self.objects)
                })
                .collect::<Vec<_>>();

            let sides_up = bottoms
                .clone()
                .into_iter()
                .zip(&surfaces)
                .map(|(bottom, surface)| {
                    let [_, from] = bottom.vertices();

                    let from = from.surface_form().clone();
                    let to = Handle::<SurfaceVertex>::partial()
                        .with_position(Some(from.position() + [Z, edge_length]))
                        .with_surface(Some(surface.clone()));

                    Handle::<HalfEdge>::partial()
                        .with_vertices(Some([
                            Handle::<Vertex>::partial()
                                .with_surface_form(Some(from)),
                            Handle::<Vertex>::partial()
                                .with_surface_form(Some(to)),
                        ]))
                        .as_line_segment()
                        .build(self.objects)
                })
                .collect::<Vec<_>>();

            let sides_down = {
                let mut sides_up_prev = sides_up.clone();
                sides_up_prev.rotate_right(1);

                bottoms
                    .clone()
                    .into_iter()
                    .zip(sides_up_prev)
                    .zip(&surfaces)
                    .map(|((bottom, side_up_prev), surface)| {
                        let [_, from] = side_up_prev.vertices();
                        let [to, _] = bottom.vertices();

                        let to = to.surface_form().clone();
                        let from = Handle::<SurfaceVertex>::partial()
                            .with_position(Some(
                                to.position() + [Z, edge_length],
                            ))
                            .with_surface(Some(surface.clone()))
                            .with_global_form(Some(from.global_form().clone()));

                        let curve = Handle::<Curve>::partial()
                            .with_global_form(Some(
                                side_up_prev.curve().global_form().clone(),
                            ));

                        Handle::<HalfEdge>::partial()
                            .with_curve(Some(curve))
                            .with_vertices(Some([
                                Handle::<Vertex>::partial()
                                    .with_surface_form(Some(from)),
                                Handle::<Vertex>::partial()
                                    .with_surface_form(Some(to)),
                            ]))
                            .as_line_segment()
                            .build(self.objects)
                    })
                    .collect::<Vec<_>>()
            };

            let tops = sides_up
                .clone()
                .into_iter()
                .zip(sides_down.clone())
                .map(|(side_up, side_down)| {
                    let [_, from] = side_up.vertices();
                    let [to, _] = side_down.vertices();

                    let from = from.surface_form().clone();
                    let to = to.surface_form().clone();

                    let from = Handle::<Vertex>::partial()
                        .with_surface_form(Some(from));
                    let to =
                        Handle::<Vertex>::partial().with_surface_form(Some(to));

                    Handle::<HalfEdge>::partial()
                        .with_vertices(Some([from, to]))
                        .as_line_segment()
                        .build(self.objects)
                })
                .collect::<Vec<_>>();

            let sides = bottoms
                .into_iter()
                .zip(sides_up)
                .zip(tops.clone())
                .zip(sides_down)
                .zip(surfaces)
                .map(|((((bottom, side_up), top), side_down), surface)| {
                    let cycle = Cycle::partial()
                        .with_surface(Some(surface))
                        .with_half_edges([bottom, side_up, top, side_down])
                        .build(self.objects);

                    Face::from_exterior(cycle)
                });

            (sides, tops)
        };

        let top = {
            let surface = self
                .objects
                .surfaces
                .xy_plane()
                .translate([Z, Z, h], self.objects);

            let mut top_edges = top_edges;
            top_edges.reverse();

            let surface_vertices = {
                let mut edges = top_edges.iter();

                let a = edges.next().unwrap();
                let b = edges.next().unwrap();
                let c = edges.next().unwrap();
                let d = edges.next().unwrap();

                // Can be cleaned up, once `zip` is stable:
                // https://doc.rust-lang.org/std/primitive.array.html#method.zip
                let [a, b, c, d] =
                    [([-h, -h], a), ([-h, h], b), ([h, h], c), ([h, -h], d)]
                        .map(|(point, edge)| {
                            let vertex = edge.back();

                            Handle::<SurfaceVertex>::partial()
                                .with_position(Some(point))
                                .with_surface(Some(surface.clone()))
                                .with_global_form(Some(
                                    vertex.global_form().clone(),
                                ))
                                .build(self.objects)
                        });

                [a.clone(), b, c, d, a]
            };

            let mut edges = Vec::new();
            for (surface_vertices, edge) in
                surface_vertices.windows(2).zip(top_edges)
            {
                // This can't panic, as we passed `2` to `windows`. Can be
                // cleaned up, once `array_windows` is stable.
                let surface_vertices =
                    [surface_vertices[0].clone(), surface_vertices[1].clone()];

                // Can be cleaned up, once `zip` is stable:
                // https://doc.rust-lang.org/std/primitive.array.html#method.zip
                let [vertex_a, vertex_b] = edge.vertices().clone();
                let [surface_vertex_a, surface_vertex_b] = surface_vertices;
                let vertices = [
                    (vertex_a, surface_vertex_a),
                    (vertex_b, surface_vertex_b),
                ]
                .map(|(vertex, surface_form)| {
                    Handle::<Vertex>::partial()
                        .with_position(Some(vertex.position()))
                        .with_surface_form(Some(surface_form))
                });

                edges.push(
                    Handle::<HalfEdge>::partial()
                        .with_vertices(Some(vertices))
                        .with_global_form(Some(edge.global_form().clone()))
                        .as_line_segment()
                        .build(self.objects),
                );
            }

            Face::from_exterior(Cycle::new(surface, edges))
        };

        let mut faces = Vec::new();
        faces.push(bottom);
        faces.extend(sides);
        faces.push(top);

        Shell::new().with_faces(faces)
    }
}
