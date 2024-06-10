use core::ops::Deref;
extern crate alloc;
use alloc::{boxed::Box, collections::TryReserveError, rc::Rc, sync::Arc, vec::Vec};
use std::time;

pub use glium as gl;
pub use glium_shapes as gl_shapes;

use cgmath as cg;
use gl::glutin;
use nalgebra::{self as na, ComplexField, SVector};
use reflect::*;

mod app;
mod camera;
mod renderable;

pub use renderable::*;

use app::App;
use camera::{Camera, CameraController, Projection};

#[derive(Copy, Clone, Debug)]
pub struct Vertex<const N: usize> {
    pub position: [f32; N],
}

impl<const D: usize> Default for Vertex<D> {
    fn default() -> Self {
        Self { position: [0.; D] }
    }
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

pub fn run_simulation<const D: usize, M, R>(
    mirror: &M,
    rays: R,
    default_eps: <M::Scalar as ComplexField>::RealField,
) where
    M: Mirror<D> + OpenGLRenderable + ?Sized,
    R: IntoIterator<Item = (Ray<M::Scalar, D>, Option<usize>)>,
    Vertex<D>: gl::Vertex,
    Vertex<D>: From<SVector<M::Scalar, D>>,
{
    const DEFAULT_WIDTH: u32 = 1280;
    const DEFAULT_HEIGHT: u32 = 720;

    use glutin::{dpi, event_loop, window, ContextBuilder};

    let el = event_loop::EventLoop::default();
    let display = gl::Display::new(
        window::WindowBuilder::new()
            .with_inner_size(dpi::LogicalSize::new(DEFAULT_WIDTH, DEFAULT_HEIGHT))
            .with_title("Reflect"),
        ContextBuilder::new()
            .with_vsync(true)
            .with_multisampling(1 << 4),
        &el,
    )
    .expect("failed to build display");

    let app = App::from_simulation(mirror, rays, &display, default_eps);

    app.run(display, el);
}
