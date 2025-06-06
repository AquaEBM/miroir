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

pub trait Hyperplane {
    type Vector: Vector;

    fn reflect(&self, v: &mut Self::Vector);
}

impl<H: Hyperplane, I: Hyperplane<Vector = H::Vector>> Hyperplane for Either<H, I> {
    type Vector = H::Vector;

    fn reflect(&self, v: &mut Self::Vector) {
        match self {
            Either::Left(m) => m.reflect(v),
            Either::Right(m) => m.reflect(v),
        }
    }
}

pub type Scalar<T> = <<T as Hyperplane>::Vector as Vector>::Scalar;

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
    pub fn reflect_dir(&mut self, dir: &(impl Hyperplane<Vector = V> + ?Sized)) {
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
    pub fn closest_intersection<H: Hyperplane<Vector = V>>(
        &self,
        mirror: &(impl Mirror<H> + ?Sized),
        eps: &V::Scalar,
    ) -> Option<(Scalar<H>, H)> {
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

pub struct Intersection<H: Hyperplane> {
    dist: Scalar<H>,
    dir: H,
}

impl<H: Hyperplane> From<Intersection<H>> for (Scalar<H>, H) {
    fn from(Intersection { dist, dir }: Intersection<H>) -> Self {
        (dist, dir)
    }
}

impl<H: Hyperplane> Intersection<H> {
    #[inline]
    pub fn map<I: Hyperplane>(
        self,
        fdist: impl FnOnce(Scalar<H>) -> Scalar<I>,
        fdir: impl FnOnce(H) -> I,
    ) -> Intersection<I> {
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
    pub fn closest<H: Hyperplane<Vector: Vector<Scalar = S>>>(
        &self,
        tangents: impl IntoIterator<Item = (Scalar<H>, H)>,
    ) -> Option<Intersection<H>>
    where
        Scalar<H>: PartialOrd,
    {
        tangents
            .into_iter()
            .filter(|(d, _)| d > self.eps)
            .min_by(|(d1, _), (d2, _)| d1.partial_cmp(d2).unwrap())
            .map(|(dist, dir)| Intersection { dist, dir })
    }
}

pub trait Mirror<H: Hyperplane> {
    fn closest_intersection(
        &self,
        ray: &Ray<H::Vector>,
        ctx: SimulationCtx<Scalar<H>>,
    ) -> Option<Intersection<H>>;
}

impl<H: Hyperplane, I: Hyperplane<Vector = H::Vector>, M: Mirror<H>, N: Mirror<I>>
    Mirror<Either<H, I>> for Either<M, N>
{
    fn closest_intersection(
        &self,
        ray: &Ray<H::Vector>,
        ctx: SimulationCtx<Scalar<H>>,
    ) -> Option<Intersection<Either<H, I>>> {

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

impl<H: Hyperplane, I: Hyperplane<Vector = H::Vector>, M: Mirror<H>, N: Mirror<I>>
    Mirror<Either<H, I>> for (M, N)
where
    Scalar<H>: PartialOrd,
{
    fn closest_intersection(
        &self,
        ray: &Ray<H::Vector>,
        ctx: SimulationCtx<Scalar<H>>,
    ) -> Option<Intersection<Either<H, I>>> {
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

impl<H: Hyperplane, T: Mirror<H>> Mirror<H> for [T]
where
    Scalar<H>: PartialOrd,
{
    #[inline]
    fn closest_intersection(
        &self,
        ray: &Ray<H::Vector>,
        ctx: SimulationCtx<Scalar<H>>,
    ) -> Option<Intersection<H>> {
        ctx.closest(
            self.iter()
                .filter_map(|m| m.closest_intersection(ray, ctx.clone()))
                .map(Into::into),
        )
    }
}

impl<H: Hyperplane, T: Mirror<H>, const N: usize> Mirror<H> for [T; N]
where
    Scalar<H>: PartialOrd,
{
    #[inline]
    fn closest_intersection(
        &self,
        ray: &Ray<H::Vector>,
        ctx: SimulationCtx<Scalar<H>>,
    ) -> Option<Intersection<H>> {
        self.as_slice().closest_intersection(ray, ctx)
    }
}

#[cfg(feature = "alloc")]
impl<H: Hyperplane, T: Mirror<H>> Mirror<H> for Vec<T>
where
    Scalar<H>: PartialOrd,
{
    #[inline]
    fn closest_intersection(
        &self,
        ray: &Ray<H::Vector>,
        ctx: SimulationCtx<Scalar<H>>,
    ) -> Option<Intersection<H>> {
        self.as_slice().closest_intersection(ray, ctx)
    }
}

#[cfg(feature = "alloc")]
impl<H: Hyperplane, T: Mirror<H> + ?Sized> Mirror<H> for Box<T> {
    #[inline]
    fn closest_intersection(
        &self,
        ray: &Ray<H::Vector>,
        ctx: SimulationCtx<Scalar<H>>,
    ) -> Option<Intersection<H>> {
        self.as_ref().closest_intersection(ray, ctx)
    }
}

#[cfg(feature = "alloc")]
impl<H: Hyperplane, T: Mirror<H> + ?Sized> Mirror<H> for Arc<T> {
    #[inline]
    fn closest_intersection(
        &self,
        ray: &Ray<H::Vector>,
        ctx: SimulationCtx<Scalar<H>>,
    ) -> Option<Intersection<H>> {
        self.as_ref().closest_intersection(ray, ctx)
    }
}

#[cfg(feature = "alloc")]
impl<H: Hyperplane, T: Mirror<H> + ?Sized> Mirror<H> for Rc<T> {
    #[inline]
    fn closest_intersection(
        &self,
        ray: &Ray<H::Vector>,
        ctx: SimulationCtx<Scalar<H>>,
    ) -> Option<Intersection<H>> {
        self.as_ref().closest_intersection(ray, ctx)
    }
}

impl<H: Hyperplane, T: Mirror<H> + ?Sized> Mirror<H> for &T {
    #[inline]
    fn closest_intersection(
        &self,
        ray: &Ray<H::Vector>,
        ctx: SimulationCtx<Scalar<H>>,
    ) -> Option<Intersection<H>> {
        #[allow(suspicious_double_ref_op)]
        (*self).closest_intersection(ray, ctx)
    }
}

impl<H: Hyperplane, T: Mirror<H> + ?Sized> Mirror<H> for &mut T {
    #[inline]
    fn closest_intersection(
        &self,
        ray: &Ray<H::Vector>,
        ctx: SimulationCtx<Scalar<H>>,
    ) -> Option<Intersection<H>> {
        (*self as &T).closest_intersection(ray, ctx)
    }
}
