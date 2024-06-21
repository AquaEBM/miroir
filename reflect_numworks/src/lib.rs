#![no_std]

use eadk::kandinsky::*;
use num_traits::{float::FloatCore, AsPrimitive};
use reflect::{
    nalgebra::{RealField, SVector, SimdComplexField, Unit},
    Mirror, Ray, RayPath,
};
use core::ops::Deref;

#[cfg(feature = "alloc")]
extern crate alloc;
#[cfg(feature = "alloc")]
use alloc::{boxed::Box, rc::Rc, sync::Arc, vec::Vec};

pub use eadk;

#[impl_trait_for_tuples::impl_for_tuples(16)]
pub trait KandinskyRenderable {
    fn draw(&self, color: Color);
}

impl<S: RealField + AsPrimitive<i16>> KandinskyRenderable for reflect_mirrors::Sphere<S, 2> {
    fn draw(&self, color: Color) {
        let [x, y] = self.center.into();
        draw_circle(x.as_(), y.as_(), self.radius().clone().as_() as u16, color)
    }
}

impl<S: RealField + AsPrimitive<i16>> KandinskyRenderable for reflect_mirrors::LineSegment<S> {
    fn draw(&self, color: Color) {
        let [start, end] = self.vertices();
        let [x0, y0] = start.into();
        let [x1, y1] = end.into();
        draw_line(x0.as_(), y0.as_(), x1.as_(), y1.as_(), color)
    }
}

impl<T: KandinskyRenderable> KandinskyRenderable for [T] {
    fn draw(&self, color: Color) {
        self.iter()
            .for_each(|a| a.draw(color));
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

#[derive(Debug, Clone)]
pub struct SimulationRay<S, const D: usize> {
    pub ray: Ray<S, D>,
    reflection_cap: Option<usize>,
    color: Color,
}

impl<const D: usize, S: PartialEq> PartialEq for SimulationRay<S, D> {
    fn eq(&self, other: &Self) -> bool {
        self.ray == other.ray && self.reflection_cap == other.reflection_cap
    }
}

impl<S, const D: usize> SimulationRay<S, D> {
    const DEFAULT_COLOR: Color = Color::from_rgb([255, 127, 0]);
    #[inline]
    #[must_use]
    pub fn new_unit_dir(origin: impl Into<SVector<S, D>>, dir: Unit<SVector<S, D>>) -> Self {
        Self {
            ray: Ray::new_unit_dir(origin, dir),
            reflection_cap: None,
            color: Self::DEFAULT_COLOR,
        }
    }

    /// # Safety
    ///
    /// `dir` must be a unit vector, or very close to one
    #[inline]
    #[must_use]
    pub unsafe fn new_unchecked_dir(
        origin: impl Into<SVector<S, D>>,
        dir: impl Into<SVector<S, D>>,
    ) -> Self {
        Self {
            ray: Ray::new_unchecked(origin, dir),
            reflection_cap: None,
            color: Self::DEFAULT_COLOR,
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
            color: Self::DEFAULT_COLOR,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SimulationParams<S> {
    pub epsilon: S,
    pub detect_loops: bool,
    pub mirror_color: Color,
    pub step_time_ms: u32,
}

impl<S: FloatCore + 'static> Default for SimulationParams<S>
where
    f64: AsPrimitive<S>,
{
    fn default() -> Self {
        Self {
            epsilon: S::epsilon() * 64.0.as_(),
            detect_loops: false,
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
        color
    } in rays
    {
        let mut prev_pt = ray.origin;
        let mut path = RayPath::new(mirror, ray, params.epsilon.clone());

        let connect_line = |prev: &mut SVector<_, 2>, to: SVector<_, 2>| {
            let [x0, y0]: [M::Scalar; 2] = (*prev).into();
            *prev = to;
            let [x1, y1] = to.into();
            draw_line(
                x0.as_(),
                y0.as_(),
                x1.as_(),
                y1.as_(),
                color,
            );
            eadk::time::sleep_ms(params.step_time_ms);
        };

        let diverges = if let Some(n) = reflection_cap {
            let mut count = 0;
            for pt in path.by_ref().take(n) {
                connect_line(&mut prev_pt, pt);
                count += 1;
            }
            count < n
        } else {
            for pt in path.by_ref() {
                connect_line(&mut prev_pt, pt)
            }
            true
        };

        if diverges {
            let new_pt = prev_pt + path.current_ray().dir.as_ref() * 1000.0.as_();
            connect_line(&mut prev_pt, new_pt);
        }
    }
}
