#![no_std]

extern crate alloc;

use alloc::{boxed::Box, rc::Rc, sync::Arc, vec::Vec};
use core::{fmt::Debug, ops::{Add, Deref}};

pub use nalgebra;

use nalgebra::{ComplexField, SMatrix, SVector, SimdComplexField, Unit};

pub type Float = f64;

/// A ray
#[derive(Clone, Debug)]
pub struct Ray<S, const D: usize> {
    /// The starting point of the half-line
    pub origin: SVector<S, D>,
    /// the direction of the half-line
    pub dir: Unit<SVector<S, D>>,
}

// Unit<Vector<T>>: PartialEq has an extra (useless?) requirement of T: Scalar
impl<S: PartialEq, const D: usize> PartialEq for Ray<S, D> {
    fn eq(&self, other: &Self) -> bool {
        &self.origin == &other.origin && self.dir.as_ref() == other.dir.as_ref()
    }
}

impl<S, const D: usize> Ray<S, D> {
    #[inline]
    #[must_use]
    pub fn new_unit_dir(origin: impl Into<SVector<S, D>>, dir: Unit<SVector<S, D>>) -> Self {
        Self {
            origin: origin.into(),
            dir
        }
    }

    #[inline]
    #[must_use]
    pub fn new_unchecked_dir(origin: impl Into<SVector<S, D>>, dir: impl Into<SVector<S, D>>) -> Self {
        Self {
            origin: origin.into(),
            dir: Unit::new_unchecked(dir.into())
        }
    }
}

impl<S: SimdComplexField, const D: usize> Ray<S, D> {

    #[inline]
    #[must_use]
    pub fn new(origin: impl Into<SVector<S, D>>, dir: impl Into<SVector<S, D>>) -> Self {
        Self {
            origin: origin.into(),
            dir: Unit::new_normalize(dir.into()),
        }
    }

    /// Reflect the ray's direction with respect to the given hyperplane
    #[inline]
    pub fn reflect_dir(&mut self, dir_space: &HyperPlane<S, D>) {
        self.dir = dir_space.reflect_unit(&self.dir);
    }

    #[inline]
    pub fn reflect_dir_optimised(&mut self, dir_space: &HyperPlane<S, D>) {
        self.dir = dir_space.reflect_unit_optimised(&self.dir)
    }

    /// Move the ray's position forward (or backward if t < 0.0) by `t`
    #[inline]
    pub fn advance(&mut self, t: S) {
        self.origin += self.dir.as_ref() * t;
    }

    /// Get the point at distance `t` (can be negative) from the ray's origin
    #[inline]
    #[must_use]
    pub fn at(&self, t: S) -> SVector<S, D> {
        self.origin.clone() + self.dir.as_ref() * t
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct SimulationCtx<S: ComplexField, const D: usize> {
    pub(crate) ray: Ray<S, D>,
    pub(crate) closest: Option<(S, HyperPlane<S, D>)>,
    pub(crate) eps: S::RealField,
}

impl<S: ComplexField, const D: usize> SimulationCtx<S, D> {
    #[inline]
    const fn new(ray: Ray<S, D>, eps: S::RealField) -> Self {
        Self { ray, closest: None, eps }
    }

    #[inline]
    #[must_use]
    pub const fn ray(&self) -> &Ray<S, D> {
        &self.ray
    }
    /// # Panics
    ///
    /// if `tangent` is parallel to `self.ray()`.
    pub fn add_tangent(&mut self, tangent: Plane<S, D>) {

        let w = tangent
            .try_ray_intersection(self.ray())
            .expect("a mirror returned a plane parallel to the ray: aborting");

        let d = w.clone().real();

        d.clone().imaginary();

        if &d >= &self.eps && self.closest.as_ref().map_or(true, |(t, _)| t.clone().real() > d) {
            self.closest = Some((w, tangent.direction));
        }
    }
}

/// A hyperplane, represented with a basis of `D-1` vectors
#[derive(Clone, Debug, PartialEq)]
pub struct HyperPlaneBasis<S, const D: usize> {
    /// See [`AffineHyperPlane::new`] for info on the layout of this field
    vectors: [SVector<S, D>; D],
}

impl<S, const D: usize> HyperPlaneBasis<S, D> {
    /// A reference to the unused first vector in the array.
    #[inline]
    #[must_use]
    pub const fn v0(&self) -> &SVector<S, D> {
        &self.vectors[0]
    }

    /// A mutable reference to the unused first vector in the array.
    #[inline]
    pub fn v0_mut(&mut self) -> &mut SVector<S, D> {
        &mut self.vectors[0]
    }

    /// A reference to the basis of the plane's direction hyperplane.
    ///
    /// The returned slice is garanteed to be of length `D - 1`.
    #[inline]
    #[must_use]
    pub fn basis(&self) -> &[SVector<S, D>] {
        &self.vectors[1..]
    }

    #[inline]
    #[must_use]
    pub const fn vectors_raw(&self) -> &[SVector<S, D>; D] {
        &self.vectors
    }
}

impl<S: ComplexField, const D: usize> HyperPlaneBasis<S, D> {
    /// Returns a vector `[t_1, ..., t_d]` whose coordinates represent
    /// the `intersection` of the given `ray` and the affine hyperplane
    /// starting at `v0` whose direction space is `self`.
    ///
    /// If it exists, the following holds:
    ///
    /// `intersection = ray.origin + t_1 * ray.direction` and,
    ///
    /// let `[v_2, ..., v_d]` be the basis of `self`'s direction space (returned by `self.basis()`)
    ///
    /// `interserction = plane.origin + sum for k in [2 ; n] t_k * v_k`
    #[inline]
    #[must_use]
    pub fn intersection_coordinates(
        &self,
        ray: &Ray<S, D>,
        v0: &SVector<S, D>,
    ) -> Option<SVector<S, D>> {
        let mut a = SMatrix::<S, D, D>::from_columns(&self.vectors);
        a.set_column(0, ray.dir.as_ref());

        a.try_inverse_mut()
            // a now contains a^-1
            .then(|| {
                let mut v = a * (&ray.origin - v0);
                let first = &mut v[0];
                *first = -first.clone();
                v
            })
    }

    /// The first element of the `vectors` can be completely arbitrary and will be ignored
    ///
    /// The remaining `D - 1` vectors are a free family spanning this hyperplane
    ///
    /// Returns `None` if the family isn't free.
    ///
    /// Note that an expression like `[T ; D - 1]` is locked under `#[feature(const_generic_exprs)]`.
    #[inline]
    #[must_use]
    pub fn new(vectors: [SVector<S, D>; D]) -> Option<(Self, HyperPlaneBasisOrtho<S, D>)> {
        let mut orthonormalized = vectors.clone();
        (SVector::orthonormalize(&mut orthonormalized[1..]) == D - 1).then_some((
            Self { vectors },
            HyperPlaneBasisOrtho {
                vectors: orthonormalized,
            },
        ))
    }
}

/// A hyperplane, like [`HyperPlaneBasis`], but the basis stored is garanteed
/// to be orthonormal, efficiently enabling projections and symmetries.
#[derive(Clone, Debug, PartialEq)]
pub struct HyperPlaneBasisOrtho<S, const D: usize> {
    /// See [`HyperPlaneBasis::new`] for info on the layout of this field
    vectors: [SVector<S, D>; D],
}

impl<S, const D: usize> HyperPlaneBasisOrtho<S, D> {
    /// A reference to the unused first vector in the array.
    #[inline]
    #[must_use]
    pub const fn v0(&self) -> &SVector<S, D> {
        &self.vectors[0]
    }

    /// A mutable reference to the unused first vector in the array.
    #[inline]
    pub fn v0_mut(&mut self) -> &mut SVector<S, D> {
        &mut self.vectors[0]
    }

    /// A reference to an orthonormal basis of `self`
    ///
    /// The returned slice is garanteed to be of length `D - 1`.
    #[inline]
    #[must_use]
    pub fn basis(&self) -> &[SVector<S, D>] {
        &self.vectors[1..]
    }
}

impl<S: SimdComplexField, const D: usize> HyperPlaneBasisOrtho<S, D> {
    /// Returns the orthogonal projection of `v` w.r.t. `self`
    #[inline]
    #[must_use]
    pub fn project(&self, v: &SVector<S, D>) -> SVector<S, D> {
        self.basis().iter().map(|e| e * v.dot(e)).sum()
    }

    /// Returns the point with the smallest distance to the affine
    /// hyperplane stating at `v0`, and directed by `self`.
    #[inline]
    #[must_use]
    pub fn closest_point_to_plane(
        &self,
        v0: &SVector<S, D>,
        p: &SVector<S, D>,
    ) -> SVector<S, D> {
        let v = p - v0;
        v0 + self.project(&v)
    }
}

impl<S: ComplexField, const D: usize> HyperPlaneBasisOrtho<S, D> {
    /// Returns a vector `[t_1, ..., t_d]` whose coordinates represent
    /// the `intersection` of the given `ray` and the affine hyperplane
    /// starting at `v0`, and directed by `self`.
    ///
    /// If it exists, the following holds:
    ///
    /// `intersection = ray.origin + t_1 * ray.direction` and,
    ///
    /// let `[v_2, ..., v_d]` be the orthonormal basis of `self` returned by `self.basis()`
    ///
    /// `interserction = v0 + sum for k in [2 ; n] t_k * v_k`
    #[inline]
    #[must_use]
    pub fn intersection_coordinates(
        &self,
        ray: &Ray<S, D>,
        v0: &SVector<S, D>,
    ) -> Option<SVector<S, D>> {
        let mut a = SMatrix::<S, D, D>::from_columns(&self.vectors);
        a.set_column(0, ray.dir.as_ref());

        a.try_inverse_mut()
            // a now contains a^-1
            .then(|| {
                let mut v = a * (&ray.origin - v0);
                let first = &mut v[0];
                *first = -first.clone();
                v
            })
    }
}

/// Different ways of representing a hyperplane
#[derive(Clone, Debug)]
pub enum HyperPlane<S, const D: usize> {
    Plane(HyperPlaneBasisOrtho<S, D>),
    Normal(Unit<SVector<S, D>>),
}

// Unit<Vector<T>>: PartiaEq has an extra (useless?) requirement of T: Scalar
impl<S: PartialEq, const D: usize> PartialEq for HyperPlane<S, D> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Plane(l0), Self::Plane(r0)) => l0 == r0,
            (Self::Normal(l0), Self::Normal(r0)) => l0.as_ref() == r0.as_ref(),
            _ => false,
        }
    }
}

impl<S: SimdComplexField, const D: usize> HyperPlane<S, D> {
    /// Reflect a vector w.r.t this hyperplane
    #[inline]
    #[must_use]
    pub fn reflect(&self, v: &SVector<S, D>) -> SVector<S, D> {

        fn two<S: Clone + Add<Output = S>>(s: S) -> S {
            s.clone() + s
        }

        match self {
            Self::Plane(plane) => two(plane.project(v)) - v,
            Self::Normal(normal) => {
                let n = normal.as_ref();
                v - two(n * v.dot(n))
            }
        }
    }

    /// Reflect a unit vector w.r.t. this hyperplane
    #[inline]
    #[must_use]
    pub fn reflect_unit(&self, v: &Unit<SVector<S, D>>) -> Unit<SVector<S, D>> {
        // Theorectically, since orthogonal reflection preserves norms,
        // there is no need to renormalize the vector here. However, some
        // precision is lost when performing this optimisation
        Unit::new_normalize(self.reflect(v.as_ref()))
    }

    #[inline]
    #[must_use]
    pub fn reflect_unit_optimised(&self, v: &Unit<SVector<S, D>>) -> Unit<SVector<S, D>> {
        Unit::new_unchecked(self.reflect(v.as_ref()))
    }
}

impl<S: ComplexField, const D: usize> HyperPlane<S, D> {
    /// Return the distance `t` such that `ray.at(t)` intersects with the affine
    /// hyperplane starting at `v0`, and whose direction space is `self`.
    ///
    /// Returns `None` if `ray` is parallel to `self`
    #[inline]
    #[must_use]
    pub fn try_ray_intersection(&self, v0: &SVector<S, D>, ray: &Ray<S, D>) -> Option<S> {
        match self {
            Self::Plane(plane) => plane.intersection_coordinates(ray, v0).map(|v| v[0].clone()),
            Self::Normal(normal) => {
                let u = ray.dir.dot(normal);
                (!u.is_zero()).then(|| (v0 - &ray.origin).dot(normal) / u)
            }
        }
    }
}

/// Different ways of representing a starting point of an affine hyperplane in `D`-dimensional euclidean space
///
/// It may be provided directly or be at a certain distance from a ray.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Intersection<S, const D: usize> {
    /// If a [`Mirror`] returns `Intersection::Distance(t)` when calculating it's intersections with a `ray`, then `ray.at(t)` belongs to the returned tangent (hyper)plane.
    ///
    /// This is useful if `t` is easy to calculate.
    Distance(S),
    /// Note that the point in this vector doesn't necessarily intersect with the ray, but it serves as an offset point for the plane represented by [`Plane`]
    StartingPoint(SVector<S, D>),
}

#[derive(Clone, Debug, PartialEq)]
/// Different ways of representing an _affine_ hyperplane in `D`-dimensional euclidean space
pub struct Plane<S, const D: usize> {
    pub intersection: Intersection<S, D>,
    pub direction: HyperPlane<S, D>,
}

impl<S: ComplexField, const D: usize> Plane<S, D> {
    /// Return the distance `t` such that `ray.at(t)` intersects with this tangent plane
    ///
    /// Returns `None` if `ray` is parallel to `self`
    #[inline]
    #[must_use]
    pub fn try_ray_intersection(&self, ray: &Ray<S, D>) -> Option<S> {
        match &self.intersection {
            Intersection::Distance(t) => Some(t.clone()),
            Intersection::StartingPoint(p) => self.direction.try_ray_intersection(p, ray),
        }
    }
}

/// The core trait of this library.
///
/// This is the first trait to implement when creating a new
/// mirror shape, as it defines the core behavior of that mirror, as a reflective (hyper)surface.
///
/// The dimension parameter, `D`, defines the dimension of the space it
/// exists in (and thus, the size of the vectors used for it's calulcations).
///
/// Some mirrors, exist and are easy to implement in all dimensions. Hence why they implement
/// [`Mirror<D>`] for all `D`. Others can only implemented in _some_ dimensions, like 2, or 3.
///
/// Running simulations with mirrors in 0 dimensions will cause panics or return unspecified
/// results.
///
/// `D` could have been an associated constant, but, lack of
/// `#[feature(generic_const_exprs)]` makes this difficult
pub trait Mirror<const D: usize> {
    type Scalar: ComplexField;
    /// Adds the tangents to this mirror, at the
    /// points of intersection between it and the `ray`, in no particular order.
    ///
    /// Tangents can be added with [`ctx.add_tangent(...)`](SimulationCtx::add_tangent).
    ///
    /// The `ray` can be accessed through [`ctx.ray( )`](SimulationCtx::ray).
    ///
    /// The ray is expected to "bounce" off the plane closest to it.
    ///
    /// Here, "bounce" refers to the process of:
    ///     - Moving forward to the intersection.
    ///     - Then, orthogonally reflecting it's direction vector with
    ///       respect to the direction hyperplane.
    ///
    /// Adds nothing if the ray doesn't intersect with the mirror that `self` represents.
    ///
    /// This method may push intersection points that occur "behind" the ray's
    /// origin, (`ray.at(t)` where `t < 0.0`) simulations must discard these accordingly.
    ///
    /// This method is deterministic, i. e. not random: for some `ray`, it always has
    /// the same behavior for that `ray`, regardless of other circumstances/external state.
    fn add_tangents(&self, ctx: &mut SimulationCtx<Self::Scalar, D>);
}

use impl_trait_for_tuples::*;

#[impl_for_tuples(1, 32)]
impl<S: ComplexField, const D: usize> Mirror<D> for Tuple {
    type Scalar = S;
    for_tuples!( where #( Tuple: Mirror<D, Scalar = S> )* );

    fn add_tangents(&self, ctx: &mut SimulationCtx<Self::Scalar, D>) {
        for_tuples!( #( Tuple.add_tangents(ctx); )* );
    }
}

impl<const D: usize, T: Mirror<D>> Mirror<D> for [T] {
    type Scalar = T::Scalar;
    #[inline]
    fn add_tangents(&self, ctx: &mut SimulationCtx<Self::Scalar, D>) {
        self.iter().for_each(|mirror| mirror.add_tangents(ctx));
    }
}

impl<const N: usize, const D: usize, T: Mirror<D>> Mirror<D> for [T; N] {
    type Scalar = T::Scalar;
    #[inline]
    fn add_tangents(&self, ctx: &mut SimulationCtx<Self::Scalar, D>) {
        self.as_slice().add_tangents(ctx);
    }
}

// It's clear that all these impls use the `Deref` trait, but writing a blanket impl over all types implementing `Deref`
// makes it impossible to implement it for new types downstream.

impl<const D: usize, T: Mirror<D> + ?Sized> Mirror<D> for Box<T> {
    type Scalar = T::Scalar;
    #[inline]
    fn add_tangents(&self, ctx: &mut SimulationCtx<Self::Scalar, D>) {
        self.deref().add_tangents(ctx);
    }
}

impl<const D: usize, T: Mirror<D> + ?Sized> Mirror<D> for Arc<T> {
    type Scalar = T::Scalar;
    #[inline]
    fn add_tangents(&self, ctx: &mut SimulationCtx<Self::Scalar, D>) {
        self.deref().add_tangents(ctx);
    }
}

impl<const D: usize, T: Mirror<D> + ?Sized> Mirror<D> for Rc<T> {
    type Scalar = T::Scalar;
    #[inline]
    fn add_tangents(&self, ctx: &mut SimulationCtx<Self::Scalar, D>) {
        self.deref().add_tangents(ctx);
    }
}

impl<const D: usize, T: Mirror<D>> Mirror<D> for Vec<T> {
    type Scalar = T::Scalar;
    #[inline]
    fn add_tangents(&self, ctx: &mut SimulationCtx<Self::Scalar, D>) {
        self.as_slice().add_tangents(ctx);
    }
}

impl<'a, const D: usize, T: Mirror<D> + ?Sized> Mirror<D> for &'a T {
    type Scalar = T::Scalar;
    #[inline]
    fn add_tangents(&self, ctx: &mut SimulationCtx<Self::Scalar, D>) {
        (*self).add_tangents(ctx);
    }
}

impl<'a, const D: usize, T: Mirror<D> + ?Sized> Mirror<D> for &'a mut T {
    type Scalar = T::Scalar;
    #[inline]
    fn add_tangents(&self, ctx: &mut SimulationCtx<Self::Scalar, D>) {
        self.deref().add_tangents(ctx);
    }
}

pub struct RayPath<'a, const D: usize, M: Mirror<D> +?Sized> {
    pub(crate) ctx: SimulationCtx<M::Scalar, D>,
    pub(crate) mirror: &'a M,
}

impl<'a, const D: usize, M: Mirror<D> + ?Sized> RayPath<'a, D, M> {
    #[inline]
    pub const fn new(mirror: &'a M, ray: Ray<M::Scalar, D>, eps: <M::Scalar as ComplexField>::RealField) -> Self {
        Self {
            ctx: SimulationCtx::new(ray, eps),
            mirror,
        }
    }

    #[inline]
    #[must_use]
    pub const fn current_ray(&self) -> &Ray<M::Scalar, D> {
        self.ctx.ray()
    }
}

impl<'a, const D: usize, M: Mirror<D> + ?Sized> Iterator for RayPath<'a, D, M> {
    type Item = SVector<M::Scalar, D>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let ctx = &mut self.ctx;
        self.mirror.add_tangents(ctx);

        let ray = &mut ctx.ray;
        ctx.closest.take().map(|(dist, direction)| {
            ray.advance(dist);
            ray.reflect_dir(&direction);
            ray.origin.clone()
        })
    }
}

#[inline]
#[must_use]
pub fn loop_index<const D: usize>(
    path: &[SVector<Float, D>],
    pt: SVector<Float, D>,
    e: Float,
) -> Option<usize> {
    path.split_last().and_then(|(last_pt, points)| {
        points.windows(2).enumerate().find_map(|(i, window)| {
            // ugly, but `slice::array_windows` is unstable
            let [this_pt, next_pt] = window else {
                // because window.len() is always 2
                unreachable!()
            };
            ((last_pt - this_pt).norm() <= e && (pt - next_pt).norm() < e).then_some(i)
        })
    })
}
