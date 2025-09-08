use core::{
    array,
    ops::{Add, Mul},
};
use num_traits::AsPrimitive;
use std::{time, collections::TryReserveError, rc::Rc, sync::Arc};

use gl::{backend::glutin::DisplayCreationError, glutin};

use glutin::{dpi, event_loop, window};
use miroir::*;
use na::SVector;

mod camera;
mod renderable;
mod sim_render_data;
use sim_render_data::SimulationRenderData;

pub use miroir;
pub use glium as gl;
pub use glium_shapes as gl_shapes;
pub use renderable::*;

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
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
    S: AsPrimitive<f32>,
{
    fn from(v: SVector<S, D>) -> Self {
        Self {
            position: array::from_fn(|i| v[i].as_()),
        }
    }
}

pub trait GLSimulationVertex: Add + Mul<f32> + gl::Vertex {
    const SHADER_SRC: &str;
}

impl GLSimulationVertex for Vertex2D {
    const SHADER_SRC: &str = r"#version 140

in vec2 position;
uniform mat4 perspective;
uniform mat4 view;

void main() {
    gl_Position = perspective * view * vec4(position, 0.0, 1.0);
}";
}

impl GLSimulationVertex for Vertex3D {
    const SHADER_SRC: &str = r"#version 140

in vec3 position;
uniform mat4 perspective;
uniform mat4 view;

void main() {
    gl_Position = perspective * view * vec4(position, 1.0);
}";
}

pub trait ToGLVertex {
    type Vertex: GLSimulationVertex;
    fn to_gl_vertex(&self) -> Self::Vertex;
}

impl<S, const D: usize> ToGLVertex for SVector<S, D>
where
    S: AsPrimitive<f32>,
    Vertex<D>: GLSimulationVertex,
{
    type Vertex = Vertex<D>;

    fn to_gl_vertex(&self) -> Self::Vertex {
        (*self).into()
    }
}

/// A set of global parameters for a simulation.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct RayParams<S> {
    /// See [`Ray::closest_intersection`] for more info on the role of this field.
    ///
    /// Will also be used as the comparison epsilon when detecting loops.
    pub epsilon: S,
    /// Whether to detect if the ray's path ends up in an infinite loop,
    /// and the epsilon value used for comparisons, and the color used to draw the section
    /// of the path that loops infinitely
    pub loop_detection: Option<(S, [f32 ; 4])>,
    pub reflection_cap: Option<usize>,
    pub path_color: [f32 ; 4],
}

impl<S: Copy + 'static> Default for RayParams<S>
where
    f64: AsPrimitive<S>,
{
    fn default() -> Self {
        Self {
            epsilon: 1e-6.as_(),
            loop_detection: None,
            reflection_cap: None,
            path_color: [1., 1., 1., 1.],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct SimulationParams {
    pub mirror_color: [f32 ; 4],
    pub bg_color: [f32 ; 4],
}

impl Default for SimulationParams {
    fn default() -> Self {
        Self {
            mirror_color: [0., 0., 1., 0.33],
            bg_color: [0., 0., 0., 1.],
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
    pub fn display<R: Reflector<Vector: Vector + VMulAdd + ToGLVertex + 'static>>(
        self,
        mirror: &(impl Mirror<R> + OpenGLRenderable + ?Sized),
        rays: impl IntoIterator<Item = (Ray<R::Vector>, RayParams<Scalar<R>>)>,
        params: SimulationParams,
    ) where
        Scalar<R>: Copy + 'static,
        f64: AsPrimitive<Scalar<R>>,
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
                .with_inner_size(dpi::LogicalSize::new(1067, 600))
                .with_title("Miroir"),
            glutin::ContextBuilder::new()
                .with_vsync(true)
                .with_multisampling(1 << 3),
        )
        .expect("failed to build display")
    }
}
