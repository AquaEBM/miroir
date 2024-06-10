use super::*;

/// An open, cylinder-shaped mirror,
pub struct Cylinder {
    start: SVector<Float, 3>,
    dist: SVector<Float, 3>,
    inv_norm_dist_squared: Float,
    radius: Float,
    radius_sq: Float,
}

impl Cylinder {
    /// Create a new cylinder from a line segment and a radius
    #[inline]
    #[must_use]
    pub fn new(
        segment_start: impl Into<SVector<Float, 3>>,
        segment_end: impl Into<SVector<Float, 3>>,
        radius: Float,
    ) -> Self {
        let start = segment_start.into();
        let end = segment_end.into();
        let dist = end - start;
        let dist_sq = dist.norm_squared();

        Self {
            start,
            dist,
            radius,
            radius_sq: radius * radius,
            inv_norm_dist_squared: dist_sq.recip(),
        }
    }

    #[inline]
    pub const fn segment_dist(&self) -> SVector<Float, 3> {
        self.dist
    }

    #[inline]
    pub fn line_segment(&self) -> [SVector<Float, 3>; 2] {
        [self.start, self.start + self.dist]
    }

    #[inline]
    pub const fn radius(&self) -> Float {
        self.radius
    }

    #[inline]
    pub fn set_radius(&mut self, radius: Float) -> bool {
        let r_abs = radius.abs();
        let ok = r_abs > Float::EPSILON * 16.0;

        if ok {
            self.radius = r_abs;
        }

        ok
    }
}

impl Mirror<3> for Cylinder {
    type Scalar = Float;
    fn add_tangents(&self, ctx: &mut SimulationCtx<Float, 3>) {
        let line_coord = |v| self.dist.dot(&v) * self.inv_norm_dist_squared;
        let p = |v| line_coord(v) * self.dist;

        let ray = ctx.ray().clone();

        let m = ray.origin - self.start;
        let d = ray.dir.into_inner();
        let pm = p(m);
        let pd = p(d);

        let a = (d - pd).norm_squared();
        let b = d.dot(&pm).mul_add(-2.0, pm.dot(&pd) + m.dot(&d));
        let c = (m - pm).norm_squared() - self.radius_sq;

        let delta = c.mul_add(-a, b * b);

        if delta >= 0. {
            let root_delta = delta.sqrt();
            let neg_b = -b;

            for t in [(neg_b - root_delta) / a, (neg_b + root_delta) / a] {
                let origin = ray.at(t);
                let coord = line_coord(origin - self.start);

                let line_pt = self.start + self.dist * coord;

                // Thanks clippy!
                if (0.0..=1.0).contains(&coord) {
                    // SAFETY: the vector `origin - v0` always has length `r = self.radius`
                    let normal = Unit::new_unchecked((origin - line_pt) / self.radius.abs());

                    ctx.add_tangent(Plane {
                        intersection: Intersection::Distance(t),
                        direction: HyperPlane::Normal(normal),
                    })
                }
            }
        }
    }
}

struct CylinderRenderData {
    vertices: gl::VertexBuffer<Vertex3D>,
}

impl RenderData for CylinderRenderData {
    fn vertices(&self) -> gl::vertex::VerticesSource {
        (&self.vertices).into()
    }

    fn indices(&self) -> gl::index::IndicesSource {
        gl::index::IndicesSource::NoIndices {
            primitives: gl::index::PrimitiveType::TriangleStrip,
        }
    }
}

impl OpenGLRenderable for Cylinder {
    fn append_render_data(&self, display: &gl::Display, list: &mut List<Box<dyn RenderData>>) {
        const NUM_POINTS: usize = 360;

        let d = self.segment_dist().map(|s| s as f32);

        let d_norm = d.normalize();

        let v = nalgebra::SVector::from([0.0, 0.0, 1.0]) + d_norm;

        // Rotation matrix to rotate the circle so it faces the axis formed by our line segment
        // specifically it is the orthogonal matrix that maps the unit `z` vector `a = [0, 0, 1]`
        // to a unit vector `b`, let `v = a + b`, and vT be the transpose of v, also,
        // let `O = (v * vT) / (vT * v), or v ⊗ v / <v, v>` (outer product over inner product)
        // Then, `R = 2 * O - Id`
        let id = nalgebra::SMatrix::identity();
        let o = nalgebra::SMatrix::from_fn(|i, j| v[i] * v[j]);
        let rot = 2.0 / v.norm_squared() * o - id;

        let r = self.radius() as f32;
        let start = self.line_segment()[0].map(|s| s as f32);

        use core::f32::consts::TAU;

        const NUM_VERTICES: usize = (NUM_POINTS + 1) * 2;

        let mut vertices: [_; NUM_VERTICES] = [Default::default(); NUM_VERTICES];

        vertices.chunks_exact_mut(2).enumerate().for_each(|(i, w)| {
            let [a, b] = w else { unreachable!() };

            let [x, y]: [f32; 2] = (i as f32 / NUM_POINTS as f32 * TAU).sin_cos().into();
            let vertex = [x * r, y * r, 0.];
            let k = rot * nalgebra::SVector::from(vertex) + start;
            (*a, *b) = (k.into(), (k + d).into())
        });

        let vertices = gl::VertexBuffer::immutable(display, vertices.as_slice()).unwrap();

        list.push(Box::new(CylinderRenderData { vertices }))
    }
}
