#![no_std]

use core::ops::Deref;
use eadk::kandinsky::*;
use miroir::{
    either::Either,
    nalgebra::{RealField, SVector},
    Hyperplane, Mirror, Ray, Scalar, VMulAdd,
};
use num_traits::AsPrimitive;

#[cfg(feature = "alloc")]
extern crate alloc;
#[cfg(feature = "alloc")]
use alloc::{boxed::Box, rc::Rc, sync::Arc, vec::Vec};

pub use eadk;
pub use miroir;

pub trait ToPoint {
    fn to_point(&self) -> Point;
}

impl<S: miroir::nalgebra::Scalar + AsPrimitive<i16>> ToPoint for SVector<S, 2> {
    fn to_point(&self) -> Point {
        let [x, y] = (*self).into();
        Point {
            x: x.as_(),
            y: y.as_(),
        }
    }
}

/// A trait enabling [`Mirror`]s to be drawn on your Numworks Calculator's screen.
#[impl_trait_for_tuples::impl_for_tuples(16)]
pub trait KandinskyRenderable {
    fn draw(&self, color: Color);
}

impl<S: RealField + AsPrimitive<i16>> KandinskyRenderable for miroir_shapes::Sphere<S, 2> {
    fn draw(&self, color: Color) {
        draw_circle(
            self.center.to_point(),
            self.radius().as_().unsigned_abs(),
            color,
        );
    }
}

impl<S: RealField + AsPrimitive<i16>> KandinskyRenderable for miroir_shapes::LineSegment<S> {
    fn draw(&self, color: Color) {
        let [start, end] = self.vertices();
        draw_line(start.to_point(), end.to_point(), color);
    }
}

impl<T: KandinskyRenderable, U: KandinskyRenderable> KandinskyRenderable for Either<T, U> {
    fn draw(&self, color: Color) {
        match self {
            Either::Left(m) => m.draw(color),
            Either::Right(m) => m.draw(color),
        }
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

impl<T: KandinskyRenderable + ?Sized> KandinskyRenderable for &T {
    fn draw(&self, color: Color) {
        (*self).draw(color);
    }
}

impl<T: KandinskyRenderable + ?Sized> KandinskyRenderable for &mut T {
    fn draw(&self, color: Color) {
        self.deref().draw(color);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RayParams<S> {
    /// See [`Ray::closest_intersection`] for more info on the role of this field.
    ///
    /// Will also be used as the comparison epsilon when detecting loops.
    pub eps: S,
    /// A pause time between each reflection, useful for easily viewing the ray's path.
    pub step_time_ms: u32,
    /// The maximum amount of reflections this ray will do. If this is `Some(n)` the ray
    /// will perform at most `n` reflections. Default: `None`
    pub reflection_cap: Option<usize>,
    /// Color of the lines drawn on screen representing the ray's path.
    pub color: Color,
}

impl<S: Copy + 'static> Default for RayParams<S>
where
    f64: AsPrimitive<S>,
{
    fn default() -> Self {
        Self {
            reflection_cap: None,
            eps: 1e-6.as_(),
            color: Color::from_rgb([248, 180, 48]),
            step_time_ms: 0,
        }
    }
}

/// A set of global parameters for a simulation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SimulationParams {
    /// The [`Color`] passed to [`KandinskyRenderable::draw`] when requesting the mirrors
    /// to be drawn.
    pub mirror_color: Color,
}

impl Default for SimulationParams {
    fn default() -> Self {
        Self {
            mirror_color: Color::from_rgb([248, 180, 48]),
        }
    }
}

pub fn display_simulation<H: Hyperplane>(
    mirror: &(impl Mirror<H> + KandinskyRenderable + ?Sized),
    rays: impl IntoIterator<Item = (Ray<H::Vector>, RayParams<Scalar<H>>)>,
    params: SimulationParams,
) where
    H::Vector: VMulAdd + ToPoint,
    Scalar<H>: 'static + Copy,
    f64: AsPrimitive<Scalar<H>>,
{
    mirror.draw(params.mirror_color);

    for (mut ray, params) in rays {
        let mut prev_pt = ray.pos.to_point();
        let mut count = 0;
        let mut diverges = true;

        loop {
            if params.reflection_cap.is_some_and(|n| count == n) {
                diverges = false;
                break;
            }

            if let Some((dist, dir)) = ray.closest_intersection(mirror, &params.eps) {
                ray.advance(dist);
                let p1 = ray.pos.to_point();
                draw_line(prev_pt, p1, params.color);
                prev_pt = p1;
                eadk::time::sleep_ms(params.step_time_ms);
                ray.reflect_dir(&dir);
                count += 1;
            } else {
                break;
            }
        }

        if diverges {
            ray.advance(410.0.as_());
            draw_line(prev_pt, ray.pos.to_point(), params.color);
        }
    }
}
