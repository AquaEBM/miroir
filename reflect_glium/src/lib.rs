use core::ops::Deref;
extern crate alloc;
use alloc::{sync::Arc, rc::Rc};
use std::time;

use cgmath as cg;
pub use glium as gl;
pub use glium_shapes as gl_shapes;

use gl::glutin;
use reflect::{nalgebra, util::List};

mod camera;
mod ray_render_data;
mod sim_render_data;

use camera::{Camera, CameraController, Projection};
use ray_render_data::RayRenderData;
use sim_render_data::SimRenderData;

#[derive(Copy, Clone, Debug)]
pub struct Vertex<const N: usize> {
    pub position: [f32; N],
}

pub type Vertex2D = Vertex<2>;
glium::implement_vertex!(Vertex2D, position);

pub type Vertex3D = Vertex<3>;
glium::implement_vertex!(Vertex3D, position);

impl<const D: usize> From<nalgebra::SVector<f32, D>> for Vertex<D> {
    fn from(v: nalgebra::SVector<f32, D>) -> Self {
        Self { position: v.into() }
    }
}

impl<const D: usize> From<nalgebra::SVector<f64, D>> for Vertex<D> {
    fn from(v: nalgebra::SVector<f64, D>) -> Self {
        Self {
            position: v.map(|s| s as f32).into(),
        }
    }
}

fn default_display_event_loop() -> (glutin::event_loop::EventLoop<()>, gl::Display) {

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
            &el
    ).expect("failed to build display");

    (
        el,
        display,
    )
}

pub fn run_2d<T: reflect::mirror::Mirror<2> + OpenGLRenderable>(
    sim: reflect::Simulation<T, 2>,
    reflection_limit: usize,
) {
    let (el, display) = default_display_event_loop();

    let drawable_simulation = SimRenderData::<2>::from_simulation(&sim, reflection_limit, &display);

    drawable_simulation.run(display, el);
}

pub fn run_3d<T: reflect::mirror::Mirror<3> + OpenGLRenderable>(
    sim: reflect::Simulation<T, 3>,
    reflection_limit: usize,
) {
    let (el, display) = default_display_event_loop();

    let drawable_simulation = SimRenderData::<3>::from_simulation(&sim, reflection_limit, &display);

    drawable_simulation.run(display, el);
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

pub trait OpenGLRenderable {
    fn append_render_data(&self, display: &gl::Display, list: List<Box<dyn RenderData>>);
}

impl<T: OpenGLRenderable> OpenGLRenderable for [T] {
    fn append_render_data(&self, display: &gl::Display, mut list: List<Box<dyn RenderData>>) {
        self.iter()
            .for_each(|a| a.append_render_data(display, list.reborrow()))
    }
}

// It's clear that all these impls use the `Deref` trait, but writing a blanket impl over all types implementing `Deref`
// makes the trait unusable downstream

impl<T: OpenGLRenderable + ?Sized> OpenGLRenderable for Box<T> {
    fn append_render_data(&self, display: &gl::Display, list: List<Box<dyn RenderData>>) {
        self.deref().append_render_data(display, list)
    }
}

impl<T: OpenGLRenderable + ?Sized> OpenGLRenderable for Arc<T> {
    fn append_render_data(&self, display: &gl::Display, list: List<Box<dyn RenderData>>) {
        self.deref().append_render_data(display, list)
    }
}

impl<T: OpenGLRenderable + ?Sized> OpenGLRenderable for Rc<T> {
    fn append_render_data(&self, display: &glium::Display, list: List<Box<dyn RenderData>>) {
        self.deref().append_render_data(display, list)
    }
}

impl<T: OpenGLRenderable> OpenGLRenderable for Vec<T> {
    fn append_render_data(&self, display: &glium::Display, list: List<Box<dyn RenderData>>) {
        self.deref().append_render_data(display, list)
    }
}

impl<'a, T: OpenGLRenderable + ?Sized> OpenGLRenderable for &'a T {
    fn append_render_data(&self, display: &glium::Display, list: List<Box<dyn RenderData>>) {
        (*self).append_render_data(display, list)
    }
}

impl<'a, T: OpenGLRenderable + ?Sized> OpenGLRenderable for &'a mut T {
    fn append_render_data(&self, display: &glium::Display, list: List<Box<dyn RenderData>>) {
        self.deref().append_render_data(display, list)
    }
}

pub struct Circle {
    pub vertices: gl::VertexBuffer<Vertex2D>,
}

impl Circle {
    pub fn new(center: [f32; 2], radius: f32, display: &gl::Display) -> Self {
        const NUM_POINTS: usize = 360;

        let c = nalgebra::SVector::from(center);

        use core::f32::consts::TAU;

        let points: Vec<Vertex2D> = (0..NUM_POINTS)
            .map(|i| {
                let pos: [f32; 2] = (i as f32 / NUM_POINTS as f32 * TAU).sin_cos().into();
                (nalgebra::SVector::from(pos) * radius + c).into()
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