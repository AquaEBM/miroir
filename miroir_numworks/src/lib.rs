#![no_std]

use core::ops::Deref;
use eadk::kandinsky::*;
use miroir::{
    nalgebra::{ComplexField, RealField, SVector, Unit},
    Mirror, Ray, RayPath,
};
use num_traits::{float::FloatCore, AsPrimitive};

#[cfg(feature = "alloc")]
extern crate alloc;
#[cfg(feature = "alloc")]
use alloc::{boxed::Box, rc::Rc, sync::Arc, vec::Vec};

pub use eadk;

/// A trait enabling [`Mirror`]s to be drawn on your Numworks Calculator's screen.
#[impl_trait_for_tuples::impl_for_tuples(16)]
pub trait KandinskyRenderable {
    fn draw(&self, color: Color);
}

impl<S: RealField + AsPrimitive<i16>> KandinskyRenderable for miroir_shapes::Sphere<S, 2> {
    fn draw(&self, color: Color) {
        let [x, y] = self.center.into();

        draw_circle(
            Point {
                x: x.as_(),
                y: y.as_(),
            },
            self.radius().clone().as_().unsigned_abs(),
            color,
        );
    }
}

impl<S: RealField + AsPrimitive<i16>> KandinskyRenderable for miroir_shapes::LineSegment<S> {
    fn draw(&self, color: Color) {
        let [start, end] = self.vertices();
        let [x0, y0] = start.into();
        let [x1, y1] = end.into();
        draw_line(
            Point {
                x: x0.as_(),
                y: y0.as_(),
            },
            Point {
                x: x1.as_(),
                y: y1.as_(),
            },
            color,
        );
    }
}

impl<T: KandinskyRenderable> KandinskyRenderable for [T] {
    fn draw(&self, color: Color) {
        for mirror in self {
            mirror.draw(color);
        }
    }
}

impl<const N: usize, T: KandinskyRenderable> KandinskyRenderable for [T; N] {
    fn draw(&self, color: Color) {
        self.as_slice().draw(color);
    }
}

// It's clear that all these impls use the `Deref` trait, but writing a blanket impl over all
// types implementing `Deref` makes the trait unusable downstream

#[cfg(feature = "alloc")]
impl<T: KandinskyRenderable + ?Sized> KandinskyRenderable for Box<T> {
    fn draw(&self, color: Color) {
        self.deref().draw(color);
    }
}

#[cfg(feature = "alloc")]
impl<T: KandinskyRenderable + ?Sized> KandinskyRenderable for Arc<T> {
    fn draw(&self, color: Color) {
        self.deref().draw(color);
    }
}

#[cfg(feature = "alloc")]
impl<T: KandinskyRenderable + ?Sized> KandinskyRenderable for Rc<T> {
    fn draw(&self, color: Color) {
        self.deref().draw(color);
    }
}

#[cfg(feature = "alloc")]
impl<T: KandinskyRenderable> KandinskyRenderable for Vec<T> {
    fn draw(&self, color: Color) {
        self.deref().draw(color);
    }
}

impl<'a, T: KandinskyRenderable + ?Sized> KandinskyRenderable for &'a T {
    fn draw(&self, color: Color) {
        (*self).draw(color);
    }
}

impl<'a, T: KandinskyRenderable + ?Sized> KandinskyRenderable for &'a mut T {
    fn draw(&self, color: Color) {
        self.deref().draw(color);
    }
}

/// A wrapper around a [`Ray`](miroir::Ray) that contains extra data required by the simulation
/// visualizer/runner
#[derive(Debug, Clone)]
pub struct SimulationRay<S, const D: usize> {
    /// They ray used in the simulation.
    pub ray: Ray<S, D>,
    /// The maximum amount of reflections this ray will do. If this is `Some(n)` the ray
    /// will perform at most `n` reflections. Default: `None`
    pub reflection_cap: Option<usize>,
    /// Color of the lines drawn on screen representing the ray's path.
    /// Default: [`Self::DEFAULT_COLOR`]
    pub color: Color,
}

impl<const D: usize, S: PartialEq> PartialEq for SimulationRay<S, D> {
    fn eq(&self, other: &Self) -> bool {
        self.ray == other.ray
            && self.reflection_cap == other.reflection_cap
            && self.color == other.color
    }
}

impl<S, const D: usize> SimulationRay<S, D> {
    pub const DEFAULT_COLOR: Color = Color::from_rgb([248, 180, 48]);

    #[inline]
    #[must_use]
    pub const fn from_ray(ray: Ray<S, D>) -> Self {
        Self {
            ray,
            reflection_cap: None,
            color: Self::DEFAULT_COLOR,
        }
    }

    #[inline]
    #[must_use]
    pub fn new_unit_dir(origin: impl Into<SVector<S, D>>, dir: Unit<SVector<S, D>>) -> Self {
        Self::from_ray(Ray::new_unit_dir(origin, dir))
    }

    #[inline]
    #[must_use]
    /// Does not normalize `dir`
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

    #[inline]
    #[must_use]
    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
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
    /// The [`Color`] passed to [`KandinskyRenderable::draw`] when requesting the mirrors
    /// to be drawn.
    pub mirror_color: Color,
    /// A pause time between each reflection, useful for easily viewing the ray's path.
    pub step_time_ms: u32,
}

impl<S: FloatCore + 'static> Default for SimulationParams<S>
where
    f64: AsPrimitive<S>,
{
    fn default() -> Self {
        Self {
            epsilon: S::epsilon() * 64.0.as_(),
            mirror_color: Color::from_rgb([255, 0, 0]),
            step_time_ms: 0,
        }
    }
}

pub fn run_simulation<M>(
    mirror: &M,
    rays: impl IntoIterator<Item = SimulationRay<M::Scalar, 2>>,
    params: SimulationParams<M::Scalar>,
) where
    M: Mirror<2, Scalar: RealField + AsPrimitive<i16>> + KandinskyRenderable + ?Sized,
    f64: AsPrimitive<M::Scalar>,
{
    mirror.draw(params.mirror_color);

    for SimulationRay {
        ray,
        reflection_cap,
        color,
    } in rays
    {
        let mut prev_pt = ray.origin;
        let mut path = RayPath {
            mirror,
            ray,
            eps: params.epsilon.clone(),
        };

        let connect_line = |prev: &mut SVector<_, 2>, to: SVector<_, 2>| {
            let [x0, y0]: [M::Scalar; 2] = (*prev).into();
            *prev = to;
            let [x1, y1] = to.into();
            draw_line(
                Point {
                    x: x0.as_(),
                    y: y0.as_(),
                },
                Point {
                    x: x1.as_(),
                    y: y1.as_(),
                },
                color,
            );
            eadk::time::sleep_ms(params.step_time_ms);
        };

        let diverges = if let Some(n) = reflection_cap {
            let mut count = 0;
            for Ray { origin, .. } in path.by_ref().take(n) {
                connect_line(&mut prev_pt, origin);
                count += 1;
            }
            count < n
        } else {
            for Ray { origin, .. } in path.by_ref() {
                connect_line(&mut prev_pt, origin);
            }
            true
        };

        if diverges {
            let new_pt = prev_pt + path.ray.dir.as_ref() * 1000.0.as_();
            connect_line(&mut prev_pt, new_pt);
        }
    }
}
