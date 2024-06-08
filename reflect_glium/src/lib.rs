use core::ops::Deref;
extern crate alloc;
use alloc::{boxed::Box, collections::TryReserveError, rc::Rc, sync::Arc, vec::Vec};
use std::time;

pub use glium as gl;
pub use glium_shapes as gl_shapes;

use cgmath as cg;
use gl::glutin;
use nalgebra as na;
use reflect::*;

mod app;
mod camera;

use app::App;
use camera::{Camera, CameraController, Projection};

#[derive(Copy, Clone, Debug)]
pub struct Vertex<const N: usize> {
    pub position: [f32; N],
}

pub type Vertex2D = Vertex<2>;
gl::implement_vertex!(Vertex2D, position);

pub type Vertex3D = Vertex<3>;
gl::implement_vertex!(Vertex3D, position);

impl<const D: usize> From<na::SVector<f32, D>> for Vertex<D> {
    fn from(v: na::SVector<f32, D>) -> Self {
        Self { position: v.into() }
    }
}

impl<const D: usize> From<na::SVector<f64, D>> for Vertex<D> {
    fn from(v: na::SVector<f64, D>) -> Self {
        Self {
            position: v.map(|s| s as f32).into(),
        }
    }
}

pub fn run_simulation<
    const D: usize,
    M: Mirror<D> + OpenGLRenderable + ?Sized,
    R: IntoIterator<Item = Ray<D>>,
>(
    mirror: &M,
    rays: R,
    reflection_limit: Option<usize>,
) where
    Vertex<D>: gl::Vertex,
{
    const DEFAULT_WIDTH: u32 = 1280;
    const DEFAULT_HEIGHT: u32 = 720;

    let el = glutin::event_loop::EventLoop::new();
    let display = gl::Display::new(
        glutin::window::WindowBuilder::new()
            .with_inner_size(glutin::dpi::LogicalSize::new(DEFAULT_WIDTH, DEFAULT_HEIGHT))
            .with_title("Reflect"),
        glutin::ContextBuilder::new()
            .with_vsync(true)
            .with_multisampling(1 << 8),
        &el,
    )
    .expect("failed to build display");

    let app = App::from_simulation(mirror, rays, reflection_limit, &display);

    app.run(display, el);
}

/// A trait encompassing a shape that can be rendered
///
/// Mirrors implementing [`OpenGLRenderable`] return objects for this trait enabling them to be rendered
/// on-screen in simulations.
pub trait RenderData {
    fn vertices(&self) -> gl::vertex::VerticesSource;
    fn indices(&self) -> gl::index::IndicesSource;
}

// glium_shapes 3d convenience blanket impl
impl<T> RenderData for T
where
    for<'a> &'a T: Into<gl::vertex::VerticesSource<'a>>,
    for<'a> &'a T: Into<gl::index::IndicesSource<'a>>,
{
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
        self.0.reserve(additional)
    }

    #[inline]
    pub fn reserve_exact(&mut self, additional: usize) {
        self.0.reserve_exact(additional)
    }

    #[inline]
    pub fn push(&mut self, v: T) {
        self.0.push(v)
    }

    #[inline]
    pub fn append(&mut self, vec: &mut Vec<T>) {
        self.0.append(vec)
    }

    #[inline]
    pub fn extend_from_slice(&mut self, slice: &[T])
    where
        T: Clone,
    {
        self.0.extend_from_slice(slice)
    }
}

impl<T> Extend<T> for List<T> {
    #[inline]
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        self.0.extend(iter)
    }
}

impl<T> From<Vec<T>> for List<T> {
    #[inline]
    fn from(value: Vec<T>) -> Self {
        Self(value)
    }
}

#[impl_trait_for_tuples::impl_for_tuples(32)]
pub trait OpenGLRenderable {
    fn append_render_data(&self, display: &gl::Display, list: &mut List<Box<dyn RenderData>>);
}

impl<T: OpenGLRenderable> OpenGLRenderable for [T] {
    fn append_render_data(&self, display: &gl::Display, list: &mut List<Box<dyn RenderData>>) {
        self.iter()
            .for_each(|a| a.append_render_data(display, list))
    }
}

impl<const N: usize, T: OpenGLRenderable> OpenGLRenderable for [T; N] {
    fn append_render_data(&self, display: &gl::Display, list: &mut List<Box<dyn RenderData>>) {
        self.as_slice().append_render_data(display, list)
    }
}

// It's clear that all these impls use the `Deref` trait, but writing a blanket impl over all
// types implementing `Deref` makes the trait unusable downstream

impl<T: OpenGLRenderable + ?Sized> OpenGLRenderable for Box<T> {
    fn append_render_data(&self, display: &gl::Display, list: &mut List<Box<dyn RenderData>>) {
        self.deref().append_render_data(display, list)
    }
}

impl<T: OpenGLRenderable + ?Sized> OpenGLRenderable for Arc<T> {
    fn append_render_data(&self, display: &gl::Display, list: &mut List<Box<dyn RenderData>>) {
        self.deref().append_render_data(display, list)
    }
}

impl<T: OpenGLRenderable + ?Sized> OpenGLRenderable for Rc<T> {
    fn append_render_data(&self, display: &gl::Display, list: &mut List<Box<dyn RenderData>>) {
        self.deref().append_render_data(display, list)
    }
}

impl<T: OpenGLRenderable> OpenGLRenderable for Vec<T> {
    fn append_render_data(&self, display: &gl::Display, list: &mut List<Box<dyn RenderData>>) {
        self.deref().append_render_data(display, list)
    }
}

impl<'a, T: OpenGLRenderable + ?Sized> OpenGLRenderable for &'a T {
    fn append_render_data(&self, display: &gl::Display, list: &mut List<Box<dyn RenderData>>) {
        (*self).append_render_data(display, list)
    }
}

impl<'a, T: OpenGLRenderable + ?Sized> OpenGLRenderable for &'a mut T {
    fn append_render_data(&self, display: &gl::Display, list: &mut List<Box<dyn RenderData>>) {
        self.deref().append_render_data(display, list)
    }
}

pub struct Circle {
    pub vertices: gl::VertexBuffer<Vertex2D>,
}

impl Circle {
    pub fn new(center: [f32; 2], radius: f32, display: &gl::Display) -> Self {
        const NUM_POINTS: usize = 360;

        let c = na::SVector::from(center);

        use core::f32::consts::TAU;

        let points: Vec<Vertex2D> = (0..NUM_POINTS)
            .map(|i| {
                let pos: [f32; 2] = (i as f32 / NUM_POINTS as f32 * TAU).sin_cos().into();
                (na::SVector::from(pos) * radius + c).into()
            })
            .collect();

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

pub struct FilledCircle(Circle);

impl From<Circle> for FilledCircle {
    fn from(value: Circle) -> Self {
        Self(value)
    }
}

impl RenderData for FilledCircle {
    fn vertices(&self) -> gl::vertex::VerticesSource {
        self.0.vertices()
    }

    fn indices(&self) -> gl::index::IndicesSource {
        gl::index::IndicesSource::NoIndices {
            primitives: gl::index::PrimitiveType::TriangleFan,
        }
    }
}
