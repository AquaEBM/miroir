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

impl<D: Direction + ?Sized, T: ?Sized + Point<D>> Point<D> for &mut T {
    #[inline]
    fn translate(&mut self, dir: &D, t: &D::Scalar) {
        (*self).translate(dir, t)
    }
}

pub trait Reflect<R: ?Sized> {
    fn reflect(&self, v: &mut R);
}

impl<D: ?Sized, R1: Reflect<D>, R2: Reflect<D>> Reflect<D> for Either<R1, R2> {
    fn reflect(&self, v: &mut D) {
        match self {
            Either::Left(m) => m.reflect(v),
            Either::Right(m) => m.reflect(v),
        }
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

pub trait Mirror<P: ?Sized, D: Direction + ?Sized> {
    type Reflector<'a>
    where
        Self: 'a;

    fn closest_intersection(
        &self,
        pos: &P,
        dir: &D,
        ctx: SimulationCtx<'_, D::Scalar>,
    ) -> Option<Intersection<D::Scalar, Self::Reflector<'_>>>;
}

pub trait MirrorExt<P: ?Sized, D: Direction + ?Sized>: Mirror<P, D> {
    fn closest_with_tolerance(
        &self,
        eps: &D::Scalar,
        pos: &P,
        dir: &D,
    ) -> Option<Intersection<D::Scalar, Self::Reflector<'_>>>;
}

impl<P: ?Sized, D: Direction + ?Sized, M: Mirror<P, D> + ?Sized> MirrorExt<P, D> for M {
    #[inline]
    fn closest_with_tolerance(
        &self,
        eps: &D::Scalar,
        pos: &P,
        dir: &D,
    ) -> Option<Intersection<D::Scalar, Self::Reflector<'_>>> {
        self.closest_intersection(pos, dir, SimulationCtx::new(eps))
    }
}

impl<P: ?Sized, D: Direction + ?Sized, M1: Mirror<P, D>, M2: Mirror<P, D>> Mirror<P, D>
    for Either<M1, M2>
{
    type Reflector<'a>
        = Either<M1::Reflector<'a>, M2::Reflector<'a>>
    where
        Self: 'a;

    fn closest_intersection(
        &self,
        pos: &P,
        dir: &D,
        ctx: SimulationCtx<'_, D::Scalar>,
    ) -> Option<Intersection<D::Scalar, Self::Reflector<'_>>> {
        match self.as_ref() {
            Either::Left(m) => m
                .closest_intersection(pos, dir, ctx)
                .map(|i| i.map(id, Either::Left)),
            Either::Right(m) => m
                .closest_intersection(pos, dir, ctx)
                .map(|i| i.map(id, Either::Right)),
        }
    }
}

impl<P: ?Sized, D: Direction + ?Sized, M: Mirror<P, D>, N: Mirror<P, D>> Mirror<P, D> for (M, N)
where
    D::Scalar: PartialOrd,
{
    type Reflector<'a>
        = Either<M::Reflector<'a>, N::Reflector<'a>>
    where
        Self: 'a;

    fn closest_intersection(
        &self,
        pos: &P,
        dir: &D,
        ctx: SimulationCtx<'_, D::Scalar>,
    ) -> Option<Intersection<D::Scalar, Self::Reflector<'_>>> {
        let (l, r) = self;

        iter::chain(
            l.closest_intersection(pos, dir, ctx.clone())
                .map(|i| i.map(id, Either::Left)),
            r.closest_intersection(pos, dir, ctx.clone())
                .map(|i| i.map(id, Either::Right)),
        )
        .min_by(|i1, i2| i1.dist.partial_cmp(&i2.dist).unwrap())
    }
}

impl<P: ?Sized, D: Direction + ?Sized, T: Mirror<P, D>> Mirror<P, D> for [T]
where
    D::Scalar: PartialOrd,
{
    type Reflector<'a>
        = T::Reflector<'a>
    where
        Self: 'a;

    #[inline]
    fn closest_intersection(
        &self,
        pos: &P,
        dir: &D,
        ctx: SimulationCtx<'_, D::Scalar>,
    ) -> Option<Intersection<D::Scalar, Self::Reflector<'_>>> {
        ctx.closest(
            self.iter()
                .filter_map(|m| m.closest_intersection(pos, dir, ctx.clone()))
                .map(Into::into),
        )
    }
}

impl<P: ?Sized, D: Direction + ?Sized, T: Mirror<P, D>, const N: usize> Mirror<P, D> for [T; N]
where
    D::Scalar: PartialOrd,
{
    type Reflector<'a>
        = T::Reflector<'a>
    where
        Self: 'a;

    #[inline]
    fn closest_intersection(
        &self,
        pos: &P,
        dir: &D,
        ctx: SimulationCtx<'_, D::Scalar>,
    ) -> Option<Intersection<D::Scalar, Self::Reflector<'_>>> {
        self.as_slice().closest_intersection(pos, dir, ctx)
    }
}

#[cfg(feature = "alloc")]
impl<P: ?Sized, D: Direction + ?Sized, T: Mirror<P, D>> Mirror<P, D> for alloc::vec::Vec<T>
where
    D::Scalar: PartialOrd,
{
    type Reflector<'a>
        = T::Reflector<'a>
    where
        Self: 'a;

    #[inline]
    fn closest_intersection(
        &self,
        pos: &P,
        dir: &D,
        ctx: SimulationCtx<'_, D::Scalar>,
    ) -> Option<Intersection<D::Scalar, Self::Reflector<'_>>> {
        self.as_slice().closest_intersection(pos, dir, ctx)
    }
}

#[cfg(feature = "alloc")]
impl<P: ?Sized, D: Direction + ?Sized, T: Mirror<P, D>> Mirror<P, D> for alloc::boxed::Box<T> {
    type Reflector<'a>
        = T::Reflector<'a>
    where
        Self: 'a;

    #[inline]
    fn closest_intersection(
        &self,
        pos: &P,
        dir: &D,
        ctx: SimulationCtx<'_, D::Scalar>,
    ) -> Option<Intersection<D::Scalar, Self::Reflector<'_>>> {
        self.as_ref().closest_intersection(pos, dir, ctx)
    }
}

#[cfg(feature = "alloc")]
impl<P: ?Sized, D: Direction + ?Sized, T: Mirror<P, D>> Mirror<P, D> for alloc::sync::Arc<T> {
    type Reflector<'a>
        = T::Reflector<'a>
    where
        Self: 'a;

    #[inline]
    fn closest_intersection(
        &self,
        pos: &P,
        dir: &D,
        ctx: SimulationCtx<'_, D::Scalar>,
    ) -> Option<Intersection<D::Scalar, Self::Reflector<'_>>> {
        self.as_ref().closest_intersection(pos, dir, ctx)
    }
}

#[cfg(feature = "alloc")]
impl<P: ?Sized, D: Direction + ?Sized, T: Mirror<P, D>> Mirror<P, D> for alloc::rc::Rc<T> {
    type Reflector<'a>
        = T::Reflector<'a>
    where
        Self: 'a;

    #[inline]
    fn closest_intersection(
        &self,
        pos: &P,
        dir: &D,
        ctx: SimulationCtx<'_, D::Scalar>,
    ) -> Option<Intersection<D::Scalar, Self::Reflector<'_>>> {
        self.as_ref().closest_intersection(pos, dir, ctx)
    }
}

impl<P: ?Sized, D: Direction + ?Sized, T: Mirror<P, D>> Mirror<P, D> for &T {
    type Reflector<'a>
        = T::Reflector<'a>
    where
        Self: 'a;

    #[inline]
    fn closest_intersection(
        &self,
        pos: &P,
        dir: &D,
        ctx: SimulationCtx<'_, D::Scalar>,
    ) -> Option<Intersection<D::Scalar, Self::Reflector<'_>>> {
        (*self).closest_intersection(pos, dir, ctx)
    }
}

impl<P: ?Sized, D: Direction + ?Sized, T: Mirror<P, D>> Mirror<P, D> for &mut T {
    type Reflector<'a>
        = T::Reflector<'a>
    where
        Self: 'a;

    #[inline]
    fn closest_intersection(
        &self,
        pos: &P,
        dir: &D,
        ctx: SimulationCtx<'_, D::Scalar>,
    ) -> Option<Intersection<D::Scalar, Self::Reflector<'_>>> {
        (*self as &T).closest_intersection(pos, dir, ctx)
    }
}
