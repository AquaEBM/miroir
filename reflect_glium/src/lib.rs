use core::{
    array,
    ops::{Add, Deref, Mul},
};
extern crate alloc;
use alloc::{boxed::Box, collections::TryReserveError, rc::Rc, sync::Arc, vec::Vec};
use num_traits::AsPrimitive;
use std::time;

pub use glium as gl;
pub use glium_shapes as gl_shapes;

use cgmath as cg;
use gl::glutin;
use nalgebra::{self as na, ComplexField, SVector, Scalar, SimdComplexField, Unit};
use reflect::*;

mod app;
mod camera;
mod renderable;

pub use renderable::*;

use app::App;
use camera::{Camera, CameraController, Projection};

#[derive(Copy, Clone, Debug)]
pub struct Vertex<const N: usize> {
    pub pos: [f32; N],
}

impl<const D: usize> Default for Vertex<D> {
    fn default() -> Self {
        Self { pos: [0.; D] }
    }
}

impl<const D: usize> Add for Vertex<D> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            pos: array::from_fn(|i| self.pos[i] + rhs.pos[i]),
        }
    }
}

impl<const D: usize> Mul<f32> for Vertex<D> {
    type Output = Self;

    fn mul(self, s: f32) -> Self::Output {
        Self {
            pos: self.pos.map(|c| c * s),
        }
    }
}

impl<const D: usize> Mul<Vertex<D>> for f32 {
    type Output = Vertex<D>;

    fn mul(self, rhs: Vertex<D>) -> Self::Output {
        Vertex {
            pos: rhs.pos.map(|c| c * self),
        }
    }
}

pub type Vertex2D = Vertex<2>;
gl::implement_vertex!(Vertex2D, pos);

pub type Vertex3D = Vertex<3>;
gl::implement_vertex!(Vertex3D, pos);

impl<S, const D: usize> From<na::SVector<S, D>> for Vertex<D>
where
    S: Scalar + AsPrimitive<f32>,
{
    fn from(v: na::SVector<S, D>) -> Self {
        Self { pos: v.map(AsPrimitive::as_).into() }
    }
}

pub struct SimulationRay<S, const D: usize> {
    pub ray: Ray<S, D>,
    reflection_cap: Option<usize>,
}

impl<S, const D: usize> SimulationRay<S, D> {
    #[inline]
    #[must_use]
    pub fn new_unit_dir(origin: impl Into<SVector<S, D>>, dir: Unit<SVector<S, D>>) -> Self {
        Self {
            ray: Ray::new_unit_dir(origin, dir),
            reflection_cap: None,
        }
    }

    #[inline]
    #[must_use]
    pub fn new_unchecked_dir(
        origin: impl Into<SVector<S, D>>,
        dir: impl Into<SVector<S, D>>,
    ) -> Self {
        Self {
            ray: Ray::new_unchecked_dir(origin, dir),
            reflection_cap: None,
        }
    }

    #[inline]
    #[must_use]
    pub fn max_reflections(&self) -> Option<&usize> {
        self.reflection_cap.as_ref()
    }

    #[inline]
    #[must_use]
    pub fn with_reflection_cap(mut self, max: usize) -> Self {
        self.reflection_cap = Some(max);
        self
    }
}

impl<S: SimdComplexField, const D: usize> SimulationRay<S, D> {
    #[inline]
    #[must_use]
    pub fn new(origin: impl Into<SVector<S, D>>, dir: impl Into<SVector<S, D>>) -> Self {
        Self {
            ray: Ray::new(origin, dir),
            reflection_cap: None,
        }
    }
}

pub fn run_simulation<const D: usize, M, R>(
    mirror: &M,
    rays: R,
    default_eps: <M::Scalar as ComplexField>::RealField,
) where
    M: Mirror<D> + OpenGLRenderable + ?Sized,
    R: IntoIterator<Item = SimulationRay<M::Scalar, D>>,
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
