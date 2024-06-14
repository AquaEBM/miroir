use core::{
    array,
    ops::{Add, Deref, Mul},
};
extern crate alloc;
use alloc::{boxed::Box, collections::TryReserveError, rc::Rc, sync::Arc, vec::Vec};
use num_traits::{float::FloatCore, AsPrimitive};
use std::time;

pub use glium as gl;
pub use glium_shapes as gl_shapes;

use cgmath as cg;
use gl::glutin;

use gl::backend::glutin::DisplayCreationError;

use glutin::{dpi, event_loop, window};
use nalgebra::{self as na, RealField, SVector, Scalar, SimdComplexField, Unit};
use reflect::*;

mod camera;
mod renderable;
mod sim_render_data;

pub use renderable::*;

use camera::{Camera, CameraController, Projection};
use sim_render_data::SimulationRenderData;

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
        Self {
            pos: v.map(AsPrimitive::as_).into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SimulationRay<S, const D: usize> {
    pub ray: Ray<S, D>,
    reflection_cap: Option<usize>,
}

impl<const D: usize, S: PartialEq> PartialEq for SimulationRay<S, D> {
    fn eq(&self, other: &Self) -> bool {
        self.ray == other.ray && self.reflection_cap == other.reflection_cap
    }
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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub struct SimulationParams<S> {
    epsilon: S,
}

impl<S: FloatCore + 'static> Default for SimulationParams<S>
where
    f64: AsPrimitive<S>,
{
    fn default() -> Self {
        Self {
            epsilon: S::epsilon() * 64.0.as_(),
        }
    }
}

pub struct SimulationWindow {
    events_loop: glutin::event_loop::EventLoop<()>,
    display: gl::Display,
}

impl SimulationWindow {
    #[inline]
    #[must_use]
    pub fn new<'a, T: glutin::ContextCurrentState>(
        wb: window::WindowBuilder,
        cb: glutin::ContextBuilder<'a, T>,
    ) -> Result<Self, DisplayCreationError> {
        let events_loop = event_loop::EventLoop::default();
        gl::Display::new(wb, cb, &events_loop).map(|display| Self {
            events_loop,
            display,
        })
    }

    #[inline]
    pub fn run<const D: usize, M, R>(self, mirror: &M, rays: R, params: SimulationParams<M::Scalar>)
    where
        M: Mirror<D, Scalar: RealField> + OpenGLRenderable + ?Sized,
        R: IntoIterator<Item = SimulationRay<M::Scalar, D>>,
        Vertex<D>: gl::Vertex + From<SVector<M::Scalar, D>>,
    {
        let Self {
            events_loop,
            display,
        } = self;

        let app = SimulationRenderData::from_simulation(mirror, rays, &display, params);

        app.run(display, events_loop);
    }
}

impl Default for SimulationWindow {
    #[inline]
    fn default() -> Self {
        Self::new(
            window::WindowBuilder::new()
                .with_inner_size(dpi::LogicalSize::new(1280, 720))
                .with_title("Reflect"),
            glutin::ContextBuilder::new()
                .with_vsync(true)
                .with_multisampling(1 << 4),
        )
        .expect("failed to build display")
    }
}
