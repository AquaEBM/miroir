use core::{
    array,
    ops::{Add, Deref, Mul},
};
extern crate alloc;
use alloc::{boxed::Box, collections::TryReserveError, rc::Rc, sync::Arc, vec::Vec};
use num_traits::{float::FloatCore, AsPrimitive};
use std::time;

use gl::glutin;

use gl::backend::glutin::DisplayCreationError;

use glutin::{dpi, event_loop, window};
use miroir::*;
use nalgebra::{ComplexField, RealField, SVector, Scalar, Unit};

mod camera;
mod renderable;
mod sim_render_data;

pub use renderable::*;
use sim_render_data::SimulationRenderData;

pub use glium as gl;
pub use glium_shapes as gl_shapes;
pub use miroir;
pub use miroir_shapes;

/// The main vertex type used when rendering simulations,
/// You are free to use whichever vertex type you wish, as long as their dimensions
/// correctly match those of the simulation .
#[derive(Copy, Clone, Debug)]
pub struct Vertex<const N: usize> {
    pub position: [f32; N],
}

impl<const D: usize> Default for Vertex<D> {
    fn default() -> Self {
        Self { position: [0.; D] }
    }
}

impl<const D: usize> Add for Vertex<D> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            position: array::from_fn(|i| self.position[i] + rhs.position[i]),
        }
    }
}

impl<const D: usize> Mul<f32> for Vertex<D> {
    type Output = Self;

    fn mul(self, s: f32) -> Self::Output {
        Self {
            position: self.position.map(|c| c * s),
        }
    }
}

impl<const D: usize> Mul<Vertex<D>> for f32 {
    type Output = Vertex<D>;

    fn mul(self, rhs: Vertex<D>) -> Self::Output {
        Vertex {
            position: rhs.position.map(|c| c * self),
        }
    }
}

pub type Vertex2D = Vertex<2>;
gl::implement_vertex!(Vertex2D, position);

pub type Vertex3D = Vertex<3>;
gl::implement_vertex!(Vertex3D, position);

impl<S, const D: usize> From<SVector<S, D>> for Vertex<D>
where
    S: Scalar + AsPrimitive<f32>,
{
    fn from(v: SVector<S, D>) -> Self {
        Self {
            position: v.map(AsPrimitive::as_).into(),
        }
    }
}

/// A wrapper around a [`Ray`](miroir::Ray) that contains extra data required by the simulation
/// visualizer/runner
#[derive(Debug, Clone)]
pub struct SimulationRay<S, const D: usize> {
    /// They ray used in the simulation.
    pub ray: Ray<S, D>,
    /// The maximum amount of reflections this ray will do. If this is `Some(n)` the ray
    /// will perform at most `n` reflections.
    pub reflection_cap: Option<usize>,
}

impl<const D: usize, S: PartialEq> PartialEq for SimulationRay<S, D> {
    fn eq(&self, other: &Self) -> bool {
        self.ray == other.ray && self.reflection_cap == other.reflection_cap
    }
}

impl<S, const D: usize> From<Ray<S, D>> for SimulationRay<S, D> {
    fn from(ray: Ray<S, D>) -> Self {
        Self::from_ray(ray)
    }
}

impl<S, const D: usize> SimulationRay<S, D> {
    #[inline]
    #[must_use]
    pub const fn from_ray(ray: Ray<S, D>) -> Self {
        Self {
            ray,
            reflection_cap: None,
        }
    }

    #[inline]
    #[must_use]
    pub fn new_unit_dir(origin: impl Into<SVector<S, D>>, dir: Unit<SVector<S, D>>) -> Self {
        Self::from_ray(Ray::new_unit_dir(origin, dir))
    }

    /// Does not normalize `dir`
    #[inline]
    #[must_use]
    pub fn new_unchecked_dir(
        origin: impl Into<SVector<S, D>>,
        dir: impl Into<SVector<S, D>>,
    ) -> Self {
        Self::from_ray(Ray::new_unchecked_dir(origin, dir))
    }

    #[inline]
    #[must_use]
    pub fn with_reflection_cap(mut self, max: usize) -> Self {
        self.reflection_cap = Some(max);
        self
    }
}

impl<S: ComplexField, const D: usize> SimulationRay<S, D> {
    #[inline]
    #[must_use]
    pub fn try_new(
        origin: impl Into<SVector<S, D>>,
        dir: impl Into<SVector<S, D>>,
    ) -> Option<Self> {
        Ray::try_new(origin, dir).map(Self::from_ray)
    }

    #[inline]
    #[must_use]
    pub fn new(origin: impl Into<SVector<S, D>>, dir: impl Into<SVector<S, D>>) -> Self {
        Self::from_ray(Ray::new(origin, dir))
    }
}

/// A set of global parameters for a simulation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SimulationParams<S> {
    /// See [`Ray::closest_intersection`] for more info on the role of this field.
    ///
    /// Will also be used as the comparison epsilon when detecting loops.
    pub epsilon: S,
    /// Whether to detect if the ray's path ends up in an infinite loop,
    /// and halt the simulation accordingly. Default: `false`
    pub detect_loops: bool,
}

impl<S: FloatCore + 'static> Default for SimulationParams<S>
where
    f64: AsPrimitive<S>,
{
    fn default() -> Self {
        Self {
            epsilon: S::epsilon() * 64.0.as_(),
            detect_loops: false,
        }
    }
}

/// A handle for the window used to visualize simulations.
pub struct SimulationWindow {
    events_loop: glutin::event_loop::EventLoop<()>,
    display: gl::Display,
}

impl SimulationWindow {
    #[inline]
    /// Create a new window to visualize simulations in from a `winit`
    /// [`WindowBuilder`](window::WindowBuilder) and a [`glutin::ContextBuilder`].
    pub fn new<T: glutin::ContextCurrentState>(
        wb: window::WindowBuilder,
        cb: glutin::ContextBuilder<T>,
    ) -> Result<Self, DisplayCreationError> {
        let events_loop = event_loop::EventLoop::default();
        gl::Display::new(wb, cb, &events_loop).map(|display| Self {
            events_loop,
            display,
        })
    }

    #[inline]
    pub fn run<const D: usize, M>(
        self,
        mirror: &M,
        rays: impl IntoIterator<Item = SimulationRay<M::Scalar, D>>,
        params: SimulationParams<M::Scalar>,
    ) where
        M: Mirror<D, Scalar: RealField> + OpenGLRenderable + ?Sized,
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
                .with_title("Miroir"),
            glutin::ContextBuilder::new()
                .with_vsync(true)
                .with_multisampling(1 << 4),
        )
        .expect("failed to build display")
    }
}
