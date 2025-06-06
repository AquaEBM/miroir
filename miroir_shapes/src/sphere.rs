use super::*;

/// All points at a certain distance (`radius`) from a certain vector (`center`)
/// where the distance here is the standard euclidean distance
// TODO: We can do other distances, can we?
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Sphere<S: ComplexField, const D: usize> {
    pub center: SVector<S, D>,
    radius: S::RealField,
    radius_sq: S::RealField,
}

impl<S: ComplexField, const D: usize> Sphere<S, D> {
    #[inline]
    #[must_use]
    pub fn new(center: impl Into<SVector<S, D>>, radius: impl Into<S::RealField>) -> Self {
        let radius = radius.into();
        Self {
            center: center.into(),
            radius: radius.clone().abs(),
            radius_sq: radius.clone() * radius,
        }
    }

    #[inline]
    #[must_use]
    pub fn radius(&self) -> &S::RealField {
        &self.radius
    }

    #[inline]
    pub fn set_radius(&mut self, r: S::RealField) {
        self.radius = r.clone();
        self.radius_sq = r.clone() * r;
    }

    #[inline]
    #[must_use]
    pub fn intersections(&self, ray: &Ray<SVector<S, D>>) -> Option<[S; 2]> {
        // substituting `V` for `P + t * D` in the sphere equation:
        // `||V - C||^2 = r^2` results in a quadratic equation in `t`.

        let v = &ray.pos - &self.center;

        let b = v.dotc(&ray.dir).real();
        let c = v.norm_squared() - self.radius_sq.clone();

        let delta = b.clone().mul_add(b.clone(), -c);

        S::from_real(delta).try_sqrt().map(move |root| {
            let neg_b = S::from_real(-b);

            [neg_b.clone() - root.clone(), neg_b + root]
        })
    }

    #[inline]
    #[must_use]
    pub fn tangents_at_intersections(
        &self,
        ray: &Ray<SVector<S, D>>,
    ) -> Option<[(S, Unit<SVector<S, D>>); 2]> {
        self.intersections(ray).map(|ds| {
            ds.map(|d| {
                (
                    d.clone(),
                    // SAFETY: p := ray.at(d) is in the sphere,
                    // so ||p - self.center|| = |self.radius|
                    Unit::new_unchecked(
                        (ray.at(d) - self.center.clone()).unscale(self.radius.clone().abs()),
                    ),
                )
            })
        })
    }
}

impl<S: RealField, const D: usize> Mirror<Unit<SVector<S, D>>> for Sphere<S, D> {
    fn closest_intersection(
        &self,
        ray: &Ray<SVector<S, D>>,
        ctx: SimulationCtx<S>,
    ) -> Option<Intersection<Unit<SVector<S, D>>>> {
        ctx.closest(self.tangents_at_intersections(ray).into_iter().flatten())
    }
}

#[cfg(feature = "glium")]
// Use glium_shapes::sphere::Sphere for the 3D implementation
impl<S: RealField + AsPrimitive<f32>> OpenGLRenderable for Sphere<S, 3> {
    fn append_render_data(&self, display: &gl::Display, list: &mut List<Box<dyn RenderData>>) {
        let r = self.radius().as_();
        let [x, y, z] = self.center.map(|s| s.as_()).into();

        let sphere = gl_shapes::sphere::SphereBuilder::new()
            .scale(r, r, r)
            .translate(x, y, z)
            .with_divisions(60, 60)
            .build(display)
            .unwrap();

        list.push(Box::new(sphere))
    }
}

#[cfg(feature = "glium")]
struct Circle {
    vertices: gl::VertexBuffer<Vertex2D>,
}

#[cfg(feature = "glium")]
impl Circle {
    fn new<const N: usize>(center: [f32; 2], radius: f32, display: &gl::Display) -> Self {
        let c = SVector::from(center);

        use core::f32::consts::TAU;

        let points: [_; N] = core::array::from_fn(|i| {
            let w = i as f32 / N as f32 * TAU;
            let p = na::Vector2::new(w.cos(), w.sin());
            (p * radius + c).into()
        });

        let vertices = gl::VertexBuffer::immutable(display, points.as_slice()).unwrap();

        Self { vertices }
    }
}

#[cfg(feature = "glium")]
impl RenderData for Circle {
    fn vertices(&self) -> gl::vertex::VerticesSource<'_> {
        (&self.vertices).into()
    }

    fn indices(&self) -> gl::index::IndicesSource<'_> {
        gl::index::IndicesSource::NoIndices {
            primitives: gl::index::PrimitiveType::LineLoop,
        }
    }
}

#[cfg(feature = "glium")]
// in 2D, the list of vertices of a circle is easy to calculate
impl<S: RealField + AsPrimitive<f32>> OpenGLRenderable for Sphere<S, 2> {
    fn append_render_data(&self, display: &gl::Display, list: &mut List<Box<dyn RenderData>>) {
        list.push(Box::new(Circle::new::<360>(
            self.center.map(|s| s.as_()).into(),
            self.radius().as_(),
            display,
        )))
    }
}

#[cfg(feature = "numworks")]
impl<S: RealField + AsPrimitive<i16>> KandinskyRenderable for Sphere<S, 2> {
    fn draw(&self, color: Color) {
        draw_circle(
            self.center.to_point(),
            self.radius().as_().unsigned_abs(),
            color,
        );
    }
}