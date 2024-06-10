use core::array;
use nalgebra::{ComplexField, Vector2};

use super::*;

#[derive(Clone, Copy, PartialEq, Debug)]
/// All points at a certain distance (`radius`) from a certain vector (`center`)
/// where the distance here is the standard euclidean distance
// TODO: We can do other distances, can we?
pub struct Sphere<S: ComplexField, const D: usize> {
    pub center: SVector<S, D>,
    pub radius: S::RealField,
}

impl<S: ComplexField, const D: usize> Sphere<S, D> {
    #[inline]
    pub fn new(center: impl Into<SVector<S, D>>, radius: impl Into<S::RealField>) -> Self {
        Self {
            center: center.into(),
            radius: radius.into(),
        }
    }

    pub fn intersections(&self, ray: &Ray<S, D>) -> Option<[S; 2]> {
        // substituting `V` for `P + t * D` in the sphere equation:
        // `||V - C||^2 = r^2` results in a quadratic equation in `t`.

        let v = ray.origin.clone() - self.center.clone();
        let r = self.radius.clone();

        let b = v.dotc(&ray.dir).real();
        let c = r.clone().mul_add(-r, v.norm_squared());

        let delta = b.clone().mul_add(b.clone(), -c);

        S::from_real(delta).try_sqrt().map(move |root| {
            let neg_b = S::from_real(-b);

            [neg_b.clone() - root.clone(), neg_b + root]
        })
    }

    #[rustfmt::skip]
    pub fn tangents_at_intersections(
        &self,
        ray: &Ray<S, D>,
    ) -> Option<[(S, Unit<SVector<S, D>>); 2]> {
        self.intersections(ray).map(|ds| ds.map(|d| (
            d.clone(),
            // SAFETY: p := ray.at(d) is in the sphere,
            // so ||p - self.center|| = |self.radius|
            Unit::new_unchecked((ray.at(d) - self.center.clone()).unscale(self.radius.clone().abs())),
        )))
    }
}

impl<S: ComplexField, const D: usize> Mirror<D> for Sphere<S, D> {
    type Scalar = S;
    fn add_tangents(&self, ctx: &mut SimulationCtx<Self::Scalar, D>) {
        if let Some(tangents) = self.tangents_at_intersections(ctx.ray()) {
            for (d, n) in tangents {
                ctx.add_tangent(Plane {
                    intersection: Intersection::Distance(d),
                    direction: HyperPlane::Normal(n),
                });
            }
        }
    }
}

// Use glium_shapes::sphere::Sphere for the 3D implementation
impl OpenGLRenderable for Sphere<Float, 3> {
    fn append_render_data(&self, display: &gl::Display, list: &mut List<Box<dyn RenderData>>) {
        let r = self.radius as f32;
        let [x, y, z] = self.center.map(|s| s as f32).into();

        let sphere = gl_shapes::sphere::SphereBuilder::new()
            .scale(r, r, r)
            .translate(x, y, z)
            .with_divisions(60, 60)
            .build(display)
            .unwrap();

        list.push(Box::new(sphere))
    }
}

struct Circle {
    vertices: gl::VertexBuffer<Vertex2D>,
}

impl Circle {
    fn new<const N: usize>(center: [f32; 2], radius: f32, display: &gl::Display) -> Self {
        let c = SVector::from(center);

        use core::f32::consts::TAU;

        let points: [_; N] = array::from_fn(|i| {
            let w = i as f32 / N as f32 * TAU;
            let p = Vector2::new(w.cos(), w.sin());
            (p * radius + c).into()
        });

        let vertices = gl::VertexBuffer::immutable(display, points.as_slice()).unwrap();

        Self { vertices }
    }
}

impl RenderData for Circle {
    fn vertices(&self) -> gl::vertex::VerticesSource {
        (&self.vertices).into()
    }

    fn indices(&self) -> gl::index::IndicesSource {
        gl::index::IndicesSource::NoIndices {
            primitives: gl::index::PrimitiveType::LineLoop,
        }
    }
}

// in 2d, the list of vertices of a circle is easy to calculate
impl OpenGLRenderable for Sphere<Float, 2> {
    fn append_render_data(&self, display: &gl::Display, list: &mut List<Box<dyn RenderData>>) {
        list.push(Box::new(Circle::new::<360>(
            self.center.map(|s| s as f32).into(),
            self.radius as f32,
            display,
        )))
    }
}
