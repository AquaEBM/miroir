use arrayvec::ArrayVec;

use super::*;

/// An open, cylinder-shaped mirror,
#[derive(Clone, Debug, PartialEq)]
pub struct Cylinder<S> {
    start: SVector<S, 3>,
    dist: SVector<S, 3>,
    inv_norm_dist_squared: S,
    radius: S,
    radius_sq: S,
}

impl<S: RealField> Cylinder<S> {
    /// Create a new cylinder from a line segment and a radius
    #[inline]
    #[must_use]
    pub fn new(
        segment_start: impl Into<SVector<S, 3>>,
        segment_end: impl Into<SVector<S, 3>>,
        radius: S,
    ) -> Self {
        let start = segment_start.into();
        let end = segment_end.into();
        let dist = end - &start;
        let dist_sq = dist.norm_squared();

        Self {
            start,
            dist,
            radius: radius.clone(),
            radius_sq: radius.clone() * radius,
            inv_norm_dist_squared: dist_sq.recip(),
        }
    }

    #[inline]
    #[must_use]
    pub const fn start(&self) -> &SVector<S, 3> {
        &self.start
    }

    #[inline]
    #[must_use]
    pub const fn segment_dist(&self) -> &SVector<S, 3> {
        &self.dist
    }

    #[inline]
    #[must_use]
    pub fn line_segment(&self) -> [SVector<S, 3>; 2] {
        [self.start.clone(), self.start.clone() + self.dist.clone()]
    }

    #[inline]
    #[must_use]
    pub const fn radius(&self) -> &S {
        &self.radius
    }

    #[inline]
    pub fn set_radius(&mut self, radius: S) {
        self.radius = radius.clone().abs();
        self.radius_sq = radius.clone() * radius;
    }

    /// Returns up to two pairs `(d, n)` (`d` may be negative),
    /// such that `P := `[`ray.at(t)`](Ray::at) instersects with `self`,
    /// and `n` is the normal vector to the direction space of the tangent
    /// to `self` at `P`, facing outwards of the cylinder.
    #[inline]
    #[must_use]
    pub fn tangents_at_intersections(
        &self,
        ray: &Ray<SVector<S, 3>>,
    ) -> ArrayVec<(S, Unit<SVector<S, 3>>), 2> {
        let line_coord = |v| self.dist.dot(&v) * self.inv_norm_dist_squared.clone();
        let p = |v| &self.dist * line_coord(v);

        let m = &ray.pos - &self.start;
        let d = &ray.dir;
        let pm = p(m.clone());
        let pd = p(d.clone());

        let a = (d - &pd).norm_squared();
        let dpm = d.dot(&pm);
        let b = (pm.dot(&pd) + m.dot(d)) - (dpm.clone() + dpm);
        let c = (&m - pm).norm_squared() - self.radius_sq.clone();

        let delta = c.mul_add(-a.clone(), b.clone() * b.clone());

        let mut out = ArrayVec::<_, 2>::new();

        if let Some(root) = delta.try_sqrt() {
            let neg_b = -b;
            let t1 = (neg_b.clone() - root.clone()) / a.clone();
            let t2 = (neg_b + root) / a;
            for t in [t1, t2] {
                let origin = ray.at(t.clone());
                let v = &origin - &self.start;
                let coord = line_coord(v);

                if (S::zero()..=S::one()).contains(&coord) {
                    let line_pt = &self.start + self.dist.clone() * coord;

                    out.push((
                        t,
                        Unit::new_unchecked((origin - line_pt).unscale(self.radius.clone())),
                    ));
                }
            }
        }

        out
    }
}

impl<S: RealField> Mirror<Unit<SVector<S, 3>>> for Cylinder<S> {
    fn closest_intersection(
        &self,
        ray: &Ray<SVector<S, 3>>,
        ctx: SimulationCtx<S>,
    ) -> Option<Intersection<Unit<SVector<S, 3>>>> {
        ctx.closest(self.tangents_at_intersections(ray))
    }
}

#[cfg(feature = "miroir_glium")]
struct CylinderRenderData {
    vertices: gl::VertexBuffer<Vertex3D>,
}

#[cfg(feature = "miroir_glium")]
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

#[cfg(feature = "miroir_glium")]
impl<S: RealField + AsPrimitive<f32>> OpenGLRenderable for Cylinder<S> {
    fn append_render_data(&self, display: &gl::Display, list: &mut List<Box<dyn RenderData>>) {
        const NUM_POINTS: usize = 360;

        let d = self.segment_dist().map(|v| v.as_());

        let d_norm = d.normalize();

        let v = nalgebra::SVector::from([0., 0., 1.]) + d_norm;

        // Rotation matrix to rotate the circle so it faces the axis formed by our line segment
        // specifically it is the orthogonal matrix that maps the unit `z` vector `a = [0, 0, 1]`
        // to a unit vector `b`, let `v = a + b`, and vT be the transpose of v, also,
        // let `O = (v * vT) / (vT * v), or v ⊗ v / <v, v>` (outer product over inner product)
        // Then, `R = 2 * O - Id`
        // TODO: use `nalgebra::Rotation`
        let id = nalgebra::SMatrix::identity();
        let o = nalgebra::SMatrix::from_fn(|i, j| v[i] * v[j]);
        let rot = 2.0 / v.norm_squared() * o - id;

        let r = self.radius().as_();
        let start = self.start().map(|s| s.as_());

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