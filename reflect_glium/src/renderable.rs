use super::*;
use core::{array, ops::AddAssign};
use na::Vector2;
use nalgebra::RealField;

/// A trait encompassing a shape that can be rendered
///
/// Mirrors implementing [`OpenGLRenderable`] return objects for this trait enabling them to be rendered
/// on-screen in simulations.
pub trait RenderData {
    fn vertices(&self) -> gl::vertex::VerticesSource;
    fn indices(&self) -> gl::index::IndicesSource;
}

// glium_shapes 3d convenience blanket impl
impl RenderData for glium_shapes::sphere::Sphere {
    fn vertices(&self) -> gl::vertex::VerticesSource {
        self.into()
    }

    fn indices(&self) -> gl::index::IndicesSource {
        self.into()
    }
}

/// A wrapper around a `Vec<T>` that only allows pushing/appending/extending etc...
pub struct List<T>(Vec<T>);

impl<T> List<T> {
    #[inline]
    pub fn into_inner(self) -> Vec<T> {
        self.0
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.0.capacity()
    }

    #[inline]
    pub fn try_reserve(&mut self, additional: usize) -> Result<(), TryReserveError> {
        self.0.try_reserve(additional)
    }

    #[inline]
    pub fn try_reserve_exact(&mut self, additional: usize) -> Result<(), TryReserveError> {
        self.0.try_reserve_exact(additional)
    }

    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        self.0.reserve(additional);
    }

    #[inline]
    pub fn reserve_exact(&mut self, additional: usize) {
        self.0.reserve_exact(additional);
    }

    #[inline]
    pub fn push(&mut self, v: T) {
        self.0.push(v);
    }

    #[inline]
    pub fn append(&mut self, vec: &mut Vec<T>) {
        self.0.append(vec);
    }

    #[inline]
    pub fn extend_from_slice(&mut self, slice: &[T])
    where
        T: Clone,
    {
        self.0.extend_from_slice(slice);
    }
}

impl<T> Extend<T> for List<T> {
    #[inline]
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        self.0.extend(iter);
    }
}

impl<T> From<Vec<T>> for List<T> {
    #[inline]
    fn from(value: Vec<T>) -> Self {
        Self(value)
    }
}

#[impl_trait_for_tuples::impl_for_tuples(1, 16)]
pub trait OpenGLRenderable {
    fn append_render_data(&self, display: &gl::Display, list: &mut List<Box<dyn RenderData>>);
}

impl<T: OpenGLRenderable> OpenGLRenderable for [T] {
    fn append_render_data(&self, display: &gl::Display, list: &mut List<Box<dyn RenderData>>) {
        self.iter()
            .for_each(|a| a.append_render_data(display, list));
    }
}

impl<const N: usize, T: OpenGLRenderable> OpenGLRenderable for [T; N] {
    fn append_render_data(&self, display: &gl::Display, list: &mut List<Box<dyn RenderData>>) {
        self.as_slice().append_render_data(display, list);
    }
}

// It's clear that all these impls use the `Deref` trait, but writing a blanket impl over all
// types implementing `Deref` makes the trait unusable downstream

impl<T: OpenGLRenderable + ?Sized> OpenGLRenderable for Box<T> {
    fn append_render_data(&self, display: &gl::Display, list: &mut List<Box<dyn RenderData>>) {
        self.deref().append_render_data(display, list);
    }
}

impl<T: OpenGLRenderable + ?Sized> OpenGLRenderable for Arc<T> {
    fn append_render_data(&self, display: &gl::Display, list: &mut List<Box<dyn RenderData>>) {
        self.deref().append_render_data(display, list);
    }
}

impl<T: OpenGLRenderable + ?Sized> OpenGLRenderable for Rc<T> {
    fn append_render_data(&self, display: &gl::Display, list: &mut List<Box<dyn RenderData>>) {
        self.deref().append_render_data(display, list);
    }
}

impl<T: OpenGLRenderable> OpenGLRenderable for Vec<T> {
    fn append_render_data(&self, display: &gl::Display, list: &mut List<Box<dyn RenderData>>) {
        self.deref().append_render_data(display, list);
    }
}

impl<'a, T: OpenGLRenderable + ?Sized> OpenGLRenderable for &'a T {
    fn append_render_data(&self, display: &gl::Display, list: &mut List<Box<dyn RenderData>>) {
        (*self).append_render_data(display, list);
    }
}

impl<'a, T: OpenGLRenderable + ?Sized> OpenGLRenderable for &'a mut T {
    fn append_render_data(&self, display: &gl::Display, list: &mut List<Box<dyn RenderData>>) {
        self.deref().append_render_data(display, list);
    }
}

// TODO: implement for all `RealField`s

// Use glium_shapes::sphere::Sphere for the 3D implementation
impl<S: RealField + AsPrimitive<f32>> OpenGLRenderable for reflect_mirrors::Sphere<S, 3> {
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
impl<S: RealField + AsPrimitive<f32>> OpenGLRenderable for reflect_mirrors::Sphere<S, 2> {
    fn append_render_data(&self, display: &gl::Display, list: &mut List<Box<dyn RenderData>>) {
        list.push(Box::new(Circle::new::<360>(
            self.center.map(|s| s.as_()).into(),
            self.radius().as_(),
            display,
        )))
    }
}

struct SimplexRenderData<const D: usize> {
    vertices: gl::VertexBuffer<Vertex<D>>,
}

impl<const D: usize> RenderData for SimplexRenderData<D> {
    fn vertices(&self) -> gl::vertex::VerticesSource {
        (&self.vertices).into()
    }

    fn indices(&self) -> gl::index::IndicesSource {
        gl::index::IndicesSource::NoIndices {
            primitives: match D {
                0 => unreachable!("dimension must not be zero"),
                1 | 2 => gl::index::PrimitiveType::LinesList,
                _ => gl::index::PrimitiveType::TriangleStrip,
            },
        }
    }
}

impl<S, const D: usize> OpenGLRenderable for reflect_mirrors::Simplex<S, D>
where
    Vertex<D>: gl::Vertex + From<SVector<S, D>>,
    SVector<S, D>: AddAssign + Clone,
{
    fn append_render_data(&self, display: &gl::Display, list: &mut List<Box<dyn RenderData>>) {
        let vertices = self.vertices().map(Vertex::from);

        list.push(Box::new(SimplexRenderData {
            vertices: gl::VertexBuffer::new(display, vertices.as_slice()).unwrap(),
        }))
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

impl<S: RealField + AsPrimitive<f32>> OpenGLRenderable for reflect_mirrors::Cylinder<S> {
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
