#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;
#[cfg(feature = "alloc")]
use alloc::{boxed::Box, rc::Rc, sync::Arc, vec::Vec};
pub use either;
use either::Either;

use core::convert::identity as id;

#[cfg(feature = "nalgebra")]
mod nalg;
#[cfg(feature = "nalgebra")]
pub use nalg::*;

pub trait Vector {
    type Scalar;
}

pub trait VMulAdd: Vector {
    fn mul_add(&self, t: Self::Scalar, other: &Self) -> Self
    where
        Self: Sized;
}

pub trait Reflector {
    type Vector: Vector;

    fn reflect(&self, v: &mut Self::Vector);
}

impl<R1: Reflector, R2: Reflector<Vector = R1::Vector>> Reflector for Either<R1, R2> {
    type Vector = R1::Vector;

    fn reflect(&self, v: &mut Self::Vector) {
        match self {
            Either::Left(m) => m.reflect(v),
            Either::Right(m) => m.reflect(v),
        }
    }
}

pub type Scalar<T> = <<T as Reflector>::Vector as Vector>::Scalar;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ray<V> {
    pub pos: V,
    pub dir: V,
}

impl<V> Ray<V> {
    #[inline]
    #[must_use]
    pub fn new(pos: impl Into<V>, dir: impl Into<V>) -> Self {
        Self {
            pos: pos.into(),
            dir: dir.into(),
        }
    }
}

impl<V: Vector> Ray<V> {
    #[inline]
    pub fn reflect_dir(&mut self, dir: &(impl Reflector<Vector = V> + ?Sized)) {
        dir.reflect(&mut self.dir);
    }

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
    pub fn closest_intersection<R: Reflector<Vector = V>>(
        &self,
        mirror: &(impl Mirror<R> + ?Sized),
        eps: &V::Scalar,
    ) -> Option<(Scalar<R>, R)> {
        let ctx = SimulationCtx::new(eps);
        mirror.closest_intersection(self, ctx).map(Into::into)
    }
}

impl<V: VMulAdd> Ray<V> {
    #[inline]
    pub fn advance(&mut self, t: V::Scalar) {
        self.pos = self.at(t);
    }

    #[inline]
    pub fn at(&self, t: V::Scalar) -> V {
        self.dir.mul_add(t, &self.pos)
    }
}

pub struct Intersection<R: Reflector> {
    dist: Scalar<R>,
    dir: R,
}

impl<R: Reflector> From<Intersection<R>> for (Scalar<R>, R) {
    fn from(Intersection { dist, dir }: Intersection<R>) -> Self {
        (dist, dir)
    }
}

impl<R: Reflector> Intersection<R> {
    #[inline]
    pub fn map<R2: Reflector>(
        self,
        fdist: impl FnOnce(Scalar<R>) -> Scalar<R2>,
        fdir: impl FnOnce(R) -> R2,
    ) -> Intersection<R2> {
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
    pub fn closest<R: Reflector<Vector: Vector<Scalar = S>>>(
        &self,
        tangents: impl IntoIterator<Item = (Scalar<R>, R)>,
    ) -> Option<Intersection<R>>
    where
        Scalar<R>: PartialOrd,
    {
        tangents
            .into_iter()
            .filter(|(d, _)| d > self.eps)
            .min_by(|(d1, _), (d2, _)| d1.partial_cmp(d2).unwrap())
            .map(|(dist, dir)| Intersection { dist, dir })
    }
}

pub trait Mirror<R: Reflector> {
    fn closest_intersection(
        &self,
        ray: &Ray<R::Vector>,
        ctx: SimulationCtx<Scalar<R>>,
    ) -> Option<Intersection<R>>;
}

impl<R1: Reflector, R2: Reflector<Vector = R1::Vector>, M: Mirror<R1>, N: Mirror<R2>>
    Mirror<Either<R1, R2>> for Either<M, N>
{
    fn closest_intersection(
        &self,
        ray: &Ray<R1::Vector>,
        ctx: SimulationCtx<Scalar<R1>>,
    ) -> Option<Intersection<Either<R1, R2>>> {

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

impl<R1: Reflector, R2: Reflector<Vector = R1::Vector>, M: Mirror<R1>, N: Mirror<R2>>
    Mirror<Either<R1, R2>> for (M, N)
where
    Scalar<R1>: PartialOrd,
{
    fn closest_intersection(
        &self,
        ray: &Ray<R1::Vector>,
        ctx: SimulationCtx<Scalar<R1>>,
    ) -> Option<Intersection<Either<R1, R2>>> {
        let (l, r) = self;

        l.closest_intersection(ray, ctx.clone())
            .map(|i| i.map(id, Either::Left))
            .into_iter()
            .chain(
                r.closest_intersection(ray, ctx)
                    .map(|i| i.map(id, Either::Right)),
            )
            .min_by(|i1, i2| i1.dist.partial_cmp(&i2.dist).unwrap())
    }
}

impl<R: Reflector, T: Mirror<R>> Mirror<R> for [T]
where
    Scalar<R>: PartialOrd,
{
    #[inline]
    fn closest_intersection(
        &self,
        ray: &Ray<R::Vector>,
        ctx: SimulationCtx<Scalar<R>>,
    ) -> Option<Intersection<R>> {
        ctx.closest(
            self.iter()
                .filter_map(|m| m.closest_intersection(ray, ctx.clone()))
                .map(Into::into),
        )
    }
}

impl<R: Reflector, T: Mirror<R>, const N: usize> Mirror<R> for [T; N]
where
    Scalar<R>: PartialOrd,
{
    #[inline]
    fn closest_intersection(
        &self,
        ray: &Ray<R::Vector>,
        ctx: SimulationCtx<Scalar<R>>,
    ) -> Option<Intersection<R>> {
        self.as_slice().closest_intersection(ray, ctx)
    }
}

#[cfg(feature = "alloc")]
impl<R: Reflector, T: Mirror<R>> Mirror<R> for Vec<T>
where
    Scalar<R>: PartialOrd,
{
    #[inline]
    fn closest_intersection(
        &self,
        ray: &Ray<R::Vector>,
        ctx: SimulationCtx<Scalar<R>>,
    ) -> Option<Intersection<R>> {
        self.as_slice().closest_intersection(ray, ctx)
    }
}

#[cfg(feature = "alloc")]
impl<R: Reflector, T: Mirror<R> + ?Sized> Mirror<R> for Box<T> {
    #[inline]
    fn closest_intersection(
        &self,
        ray: &Ray<R::Vector>,
        ctx: SimulationCtx<Scalar<R>>,
    ) -> Option<Intersection<R>> {
        self.as_ref().closest_intersection(ray, ctx)
    }
}

#[cfg(feature = "alloc")]
impl<R: Reflector, T: Mirror<R> + ?Sized> Mirror<R> for Arc<T> {
    #[inline]
    fn closest_intersection(
        &self,
        ray: &Ray<R::Vector>,
        ctx: SimulationCtx<Scalar<R>>,
    ) -> Option<Intersection<R>> {
        self.as_ref().closest_intersection(ray, ctx)
    }
}

#[cfg(feature = "alloc")]
impl<R: Reflector, T: Mirror<R> + ?Sized> Mirror<R> for Rc<T> {
    #[inline]
    fn closest_intersection(
        &self,
        ray: &Ray<R::Vector>,
        ctx: SimulationCtx<Scalar<R>>,
    ) -> Option<Intersection<R>> {
        self.as_ref().closest_intersection(ray, ctx)
    }
}

impl<R: Reflector, T: Mirror<R> + ?Sized> Mirror<R> for &T {
    #[inline]
    fn closest_intersection(
        &self,
        ray: &Ray<R::Vector>,
        ctx: SimulationCtx<Scalar<R>>,
    ) -> Option<Intersection<R>> {
        #[allow(suspicious_double_ref_op)]
        (*self).closest_intersection(ray, ctx)
    }
}

impl<R: Reflector, T: Mirror<R> + ?Sized> Mirror<R> for &mut T {
    #[inline]
    fn closest_intersection(
        &self,
        ray: &Ray<R::Vector>,
        ctx: SimulationCtx<Scalar<R>>,
    ) -> Option<Intersection<R>> {
        (*self as &T).closest_intersection(ray, ctx)
    }
}
