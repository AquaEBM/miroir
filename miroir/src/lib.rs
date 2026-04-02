#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;
pub use either;
use either::Either;

use core::{convert::identity as id, iter};

#[cfg(feature = "nalgebra")]
mod nalg;
#[cfg(feature = "nalgebra")]
pub use nalg::*;

pub trait Direction {
    type Scalar;
}

pub trait Point<D: Direction + ?Sized> {
    fn translate(&mut self, dir: &D, t: &D::Scalar);
}

pub trait Reflect<Dir: ?Sized> {
    fn reflect(&self, v: &mut Dir);
}

impl<D, R1: Reflect<D>, R2: Reflect<D>> Reflect<D> for Either<R1, R2> {
    fn reflect(&self, v: &mut D) {
        match self {
            Either::Left(m) => m.reflect(v),
            Either::Right(m) => m.reflect(v),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ray<P, D> {
    pub pos: P,
    pub dir: D,
}

impl<P, D> Ray<P, D> {
    #[inline]
    #[must_use]
    pub fn new(pos: impl Into<P>, dir: impl Into<D>) -> Self {
        Self {
            pos: pos.into(),
            dir: dir.into(),
        }
    }
}

impl<P, D> Ray<P, D> {
    #[inline]
    pub fn reflect_dir(&mut self, dir: &(impl Reflect<D> + ?Sized)) {
        dir.reflect(&mut self.dir);
    }
}

impl<P, D: Direction> Ray<P, D> {
    /// Returns the smallest (positive) `t` such that `P :=`[`self.at(t)`](Self::at)
    /// (approximately) intersects with `mirror`, and the direction space
    /// of the tangent to `mirror` at `P`, if any.
    ///
    /// It is possible that [`ray.at(t)`](Self::at) lands slightly "beyond" `mirror`, due to roundoff errors,
    /// making the `ray` bump into `P` again, in it's next reflection, sometimes
    /// resulting in it traveling "through" `mirror`, when it should "move away" from it.
    ///
    /// To mitigate this, one can require `ray` to travel a certain, usually very small,
    /// positive distance (`eps`) before being reflected again, discarding intersections whose
    /// distance to `ray` is less than `eps`, hopefully avoiding this phenomenon.
    ///
    /// `eps` must be positive and large enough to accomodate the precision errors of `S`
    /// (e. g. lower values are more acceptable for `f64` than for `f32`) but small enough
    /// to make sure `ray` doesn't ignore an intersection it shouldn't.
    #[inline]
    #[must_use]
    pub fn closest_intersection<M: Mirror<P, D> + ?Sized>(
        &self,
        mirror: &M,
        eps: &D::Scalar,
    ) -> Option<(D::Scalar, M::Reflector)> {
        let ctx = SimulationCtx::new(eps);
        mirror.closest_intersection(self, ctx).map(Into::into)
    }

    #[inline]
    pub fn advance(&mut self, t: &D::Scalar)
    where
        P: Point<D>,
    {
        self.pos.translate(&self.dir, t);
    }
}

#[derive(Debug)]
pub struct Intersection<S, D> {
    dist: S,
    dir: D,
}

impl<S, D> From<Intersection<S, D>> for (S, D) {
    fn from(Intersection { dist, dir }: Intersection<S, D>) -> Self {
        (dist, dir)
    }
}

impl<S, D> Intersection<S, D> {
    #[inline]
    pub fn map<S2, D2>(
        self,
        fdist: impl FnOnce(S) -> S2,
        fdir: impl FnOnce(D) -> D2,
    ) -> Intersection<S2, D2> {
        let Intersection { dist, dir } = self;
        Intersection {
            dist: fdist(dist),
            dir: fdir(dir),
        }
    }
}

#[derive(Debug)]
pub struct SimulationCtx<'a, S> {
    eps: &'a S,
}

impl<S> Clone for SimulationCtx<'_, S> {
    fn clone(&self) -> Self {
        Self { eps: self.eps }
    }
}

impl<'a, S> SimulationCtx<'a, S> {
    #[inline]
    #[must_use]
    fn new(eps: &'a S) -> Self {
        Self { eps }
    }

    #[inline]
    #[must_use]
    pub fn closest<R>(
        &self,
        tangents: impl IntoIterator<Item = (S, R)>,
    ) -> Option<Intersection<S, R>>
    where
        S: PartialOrd,
    {
        tangents
            .into_iter()
            .filter(|(d, _)| d > self.eps)
            .min_by(|(d1, _), (d2, _)| d1.partial_cmp(d2).unwrap())
            .map(|(dist, dir)| Intersection { dist, dir })
    }
}

pub trait Mirror<P, D: Direction> {
    type Reflector;
    fn closest_intersection(
        &self,
        ray: &Ray<P, D>,
        ctx: SimulationCtx<'_, D::Scalar>,
    ) -> Option<Intersection<D::Scalar, Self::Reflector>>;
}

impl<P, D: Direction, M1: Mirror<P, D>, M2: Mirror<P, D>> Mirror<P, D> for Either<M1, M2> {
    type Reflector = Either<M1::Reflector, M2::Reflector>;
    fn closest_intersection(
        &self,
        ray: &Ray<P, D>,
        ctx: SimulationCtx<'_, D::Scalar>,
    ) -> Option<Intersection<D::Scalar, Self::Reflector>> {
        match self.as_ref() {
            Either::Left(m) => m
                .closest_intersection(ray, ctx)
                .map(|i| i.map(id, Either::Left)),
            Either::Right(m) => m
                .closest_intersection(ray, ctx)
                .map(|i| i.map(id, Either::Right)),
        }
    }
}

impl<P, D: Direction, M1: Mirror<P, D>, M2: Mirror<P, D>> Mirror<P, D> for (M1, M2)
where
    D::Scalar: PartialOrd,
{
    type Reflector = Either<M1::Reflector, M2::Reflector>;
    fn closest_intersection(
        &self,
        ray: &Ray<P, D>,
        ctx: SimulationCtx<'_, D::Scalar>,
    ) -> Option<Intersection<D::Scalar, Self::Reflector>> {
        let (l, r) = self;

        iter::chain(
            l.closest_intersection(ray, ctx.clone())
                .map(|i| i.map(id, Either::Left)),
            r.closest_intersection(ray, ctx.clone())
                .map(|i| i.map(id, Either::Right)),
        )
        .min_by(|i1, i2| i1.dist.partial_cmp(&i2.dist).unwrap())
    }
}

impl<P, D: Direction, T: Mirror<P, D>> Mirror<P, D> for [T]
where
    D::Scalar: PartialOrd,
{
    type Reflector = T::Reflector;
    #[inline]
    fn closest_intersection(
        &self,
        ray: &Ray<P, D>,
        ctx: SimulationCtx<'_, D::Scalar>,
    ) -> Option<Intersection<D::Scalar, Self::Reflector>> {
        ctx.closest(
            self.iter()
                .filter_map(|m| m.closest_intersection(ray, ctx.clone()))
                .map(Into::into),
        )
    }
}

impl<P, D: Direction, T: Mirror<P, D>, const N: usize> Mirror<P, D> for [T; N]
where
    D::Scalar: PartialOrd,
{
    type Reflector = T::Reflector;
    #[inline]
    fn closest_intersection(
        &self,
        ray: &Ray<P, D>,
        ctx: SimulationCtx<'_, D::Scalar>,
    ) -> Option<Intersection<D::Scalar, Self::Reflector>> {
        self.as_slice().closest_intersection(ray, ctx)
    }
}

#[cfg(feature = "alloc")]
impl<P, D: Direction, T: Mirror<P, D>> Mirror<P, D> for alloc::vec::Vec<T>
where
    D::Scalar: PartialOrd,
{
    type Reflector = T::Reflector;
    #[inline]
    fn closest_intersection(
        &self,
        ray: &Ray<P, D>,
        ctx: SimulationCtx<'_, D::Scalar>,
    ) -> Option<Intersection<D::Scalar, Self::Reflector>> {
        self.as_slice().closest_intersection(ray, ctx)
    }
}

#[cfg(feature = "alloc")]
impl<P, D: Direction, T: Mirror<P, D>> Mirror<P, D> for alloc::boxed::Box<T> {
    type Reflector = T::Reflector;
    #[inline]
    fn closest_intersection(
        &self,
        ray: &Ray<P, D>,
        ctx: SimulationCtx<'_, D::Scalar>,
    ) -> Option<Intersection<D::Scalar, Self::Reflector>> {
        self.as_ref().closest_intersection(ray, ctx)
    }
}

#[cfg(feature = "alloc")]
impl<P, D: Direction, T: Mirror<P, D>> Mirror<P, D> for alloc::sync::Arc<T> {
    type Reflector = T::Reflector;
    #[inline]
    fn closest_intersection(
        &self,
        ray: &Ray<P, D>,
        ctx: SimulationCtx<'_, D::Scalar>,
    ) -> Option<Intersection<D::Scalar, Self::Reflector>> {
        self.as_ref().closest_intersection(ray, ctx)
    }
}

#[cfg(feature = "alloc")]
impl<P, D: Direction, T: Mirror<P, D>> Mirror<P, D> for alloc::rc::Rc<T> {
    type Reflector = T::Reflector;
    #[inline]
    fn closest_intersection(
        &self,
        ray: &Ray<P, D>,
        ctx: SimulationCtx<'_, D::Scalar>,
    ) -> Option<Intersection<D::Scalar, Self::Reflector>> {
        self.as_ref().closest_intersection(ray, ctx)
    }
}

impl<P, D: Direction, T: Mirror<P, D>> Mirror<P, D> for &T {
    type Reflector = T::Reflector;
    #[inline]
    fn closest_intersection(
        &self,
        ray: &Ray<P, D>,
        ctx: SimulationCtx<'_, D::Scalar>,
    ) -> Option<Intersection<D::Scalar, Self::Reflector>> {
        (*self).closest_intersection(ray, ctx)
    }
}

impl<P, D: Direction, T: Mirror<P, D>> Mirror<P, D> for &mut T {
    type Reflector = T::Reflector;
    #[inline]
    fn closest_intersection(
        &self,
        ray: &Ray<P, D>,
        ctx: SimulationCtx<'_, D::Scalar>,
    ) -> Option<Intersection<D::Scalar, Self::Reflector>> {
        (*self as &T).closest_intersection(ray, ctx)
    }
}
