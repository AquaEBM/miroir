#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;
#[cfg(feature = "alloc")]
use alloc::{boxed::Box, rc::Rc, sync::Arc, vec::Vec};
use core::{
    fmt::Debug,
    ops::{Add, Deref},
};

pub use nalgebra;

use nalgebra::{zero, ComplexField, SMatrix, SVector, SimdComplexField, Unit};

/// A hyperplane, stored as a basis of `D-1` vectors
///
/// `D` must be non-zero, or unexpected panics could happen.
#[derive(Clone, Debug, PartialEq)]
pub struct HyperplaneBasis<S, const D: usize> {
    /// See [`Self::try_new`] for info on the layout of this field.
    vectors: [SVector<S, D>; D],
}

impl<S: ComplexField, const D: usize> HyperplaneBasis<S, D> {
    /// The first element of the `vectors` can be completely arbitrary and will be ignored
    ///
    /// The remaining `D - 1` vectors are a free family spanning this hyperplane
    ///
    /// Returns `None` if the family isn't free.
    ///
    /// Note that an expression like `[T ; D - 1]` requires `#[feature(const_generic_exprs)]`.
    #[inline]
    #[must_use]
    pub fn try_new(vectors: [SVector<S, D>; D]) -> Option<(Self, HyperplaneBasisOrtho<S, D>)> {
        let mut orthonormalized = vectors.clone();
        (SVector::orthonormalize(&mut orthonormalized[1..]) == D - 1).then_some((
            Self { vectors },
            HyperplaneBasisOrtho {
                plane: Self {
                    vectors: orthonormalized,
                },
            },
        ))
    }

    /// Returns a vector `[t_1, ..., t_d]` whose coordinates represent
    /// the `intersection` between `ray` and the affine hyperplane
    /// starting at `v0`, and directed by `self`.
    ///
    /// If it exists, the following holds:
    ///
    /// `intersection = ray.origin + t_1 * ray.direction` and,
    ///
    /// let `[v_2, ..., v_d]` be the basis of `self` (returned by `self.basis()`)
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
}

impl<S, const D: usize> HyperplaneBasis<S, D> {
    /// A reference to the unused first vector in
    /// the array that `self` was constructed with.
    ///
    /// # Panics
    ///
    /// if `D == 0`
    #[inline]
    #[must_use]
    pub const fn v0(&self) -> &SVector<S, D> {
        &self.vectors[0]
    }

    /// A mutable reference to the unused first vector in
    /// the array that `self` was constructed with.
    ///
    /// # Panics
    ///
    /// if `D == 0`
    #[inline]
    pub fn v0_mut(&mut self) -> &mut SVector<S, D> {
        &mut self.vectors[0]
    }

    /// A reference to the basis of the plane's direction hyperplane.
    ///
    /// The returned slice is garanteed to be of length `D - 1`.
    ///
    /// # Panics
    ///
    /// if `D == 0`
    #[inline]
    #[must_use]
    pub fn basis(&self) -> &[SVector<S, D>] {
        self.vectors.split_first().unwrap().1
    }

    #[inline]
    #[must_use]
    pub const fn vectors_raw(&self) -> &[SVector<S, D>; D] {
        &self.vectors
    }
}

/// A hyperplane, like [`HyperplaneBasis`], but the basis stored is garanteed
/// to be orthonormal, efficiently enabling projections and symmetries.
///
/// `D` must be non-zero, or unexpected panics could happen
#[derive(Clone, Debug, PartialEq)]
pub struct HyperplaneBasisOrtho<S, const D: usize> {
    /// See [`HyperPlaneBasis::new`] for info on the layout of this field
    plane: HyperplaneBasis<S, D>,
}

impl<S, const D: usize> Deref for HyperplaneBasisOrtho<S, D> {
    type Target = HyperplaneBasis<S, D>;

    fn deref(&self) -> &Self::Target {
        &self.plane
    }
}

impl<S, const D: usize> HyperplaneBasisOrtho<S, D> {
    /// A mutable reference to the unused first vector in
    /// the array that `self` was constructed with.
    ///
    /// # Panics
    ///
    /// if `D == 0`
    pub fn v0_mut(&mut self) -> &mut SVector<S, D> {
        self.plane.v0_mut()
    }
}

impl<S: SimdComplexField, const D: usize> HyperplaneBasisOrtho<S, D> {
    /// Returns the orthogonal projection of `v` w.r.t. `self`
    #[inline]
    #[must_use]
    pub fn project(&self, v: &SVector<S, D>) -> SVector<S, D> {
        self.basis().iter().map(|e| e * v.dot(e)).sum()
    }

    /// Returns the point in the affine hyperplane starting at `v0`, and directed by `self`,
    /// with the smallest distance to `v0`.
    #[inline]
    #[must_use]
    pub fn closest_point_to_plane(&self, v0: &SVector<S, D>, p: &SVector<S, D>) -> SVector<S, D> {
        let v = p - v0;
        v0 + self.project(&v)
    }
}

/// Different ways of representing a hyperplane
#[derive(Clone, Debug)]
pub enum Hyperplane<S, const D: usize> {
    Plane(HyperplaneBasisOrtho<S, D>),
    Normal(Unit<SVector<S, D>>),
}

// Unit<Vector<T>>: PartiaEq has an extra (useless?) requirement of T: Scalar
impl<S: PartialEq, const D: usize> PartialEq for Hyperplane<S, D> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Plane(l0), Self::Plane(r0)) => l0 == r0,
            (Self::Normal(l0), Self::Normal(r0)) => l0.as_ref() == r0.as_ref(),
            _ => false,
        }
    }
}

impl<S: SimdComplexField, const D: usize> Hyperplane<S, D> {
    #[inline]
    #[must_use]
    /// Reflect a vector w.r.t this hyperplane
    pub fn reflect(&self, v: &SVector<S, D>) -> SVector<S, D> {
        #[inline]
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

    #[inline]
    #[must_use]
    /// Reflect a unit vector w.r.t. this hyperplane
    pub fn reflect_unit(&self, v: &Unit<SVector<S, D>>) -> Unit<SVector<S, D>> {
        Unit::new_normalize(self.reflect(v.as_ref()))
    }

    /// Theorectically, since orthogonal reflection preserves norms,
    /// there is no need to renormalize the vector. However, some
    /// precision is lost when performing this optimisation. So, this function
    /// must be used with caution.
    #[inline]
    #[must_use]
    pub fn reflect_unit_optimised(&self, v: &Unit<SVector<S, D>>) -> Unit<SVector<S, D>> {
        Unit::new_unchecked(self.reflect(v.as_ref()))
    }
}

/// A ray, represented as a line
#[derive(Clone, Debug)]
pub struct Ray<S, const D: usize> {
    /// The starting point of the line
    pub origin: SVector<S, D>,
    /// the direction of the line
    pub dir: Unit<SVector<S, D>>,
}

// Unit<Vector<T>>: PartialEq has an extra (useless?) requirement of T: Scalar
impl<S: PartialEq, const D: usize> PartialEq for Ray<S, D> {
    fn eq(&self, other: &Self) -> bool {
        self.origin == other.origin && self.dir.as_ref() == other.dir.as_ref()
    }
}

impl<S: ComplexField, const D: usize> Ray<S, D> {
    /// # Panics
    ///
    /// If `dir` is the zero vector.
    #[inline]
    #[must_use]
    pub fn new(origin: impl Into<SVector<S, D>>, dir: impl Into<SVector<S, D>>) -> Self {
        Self::try_new(origin, dir).expect("direction must be non-zero")
    }

    /// Returns `None` if `dir` is the zero vector.
    #[inline]
    #[must_use]
    pub fn try_new(
        origin: impl Into<SVector<S, D>>,
        dir: impl Into<SVector<S, D>>,
    ) -> Option<Self> {
        Unit::try_new(dir.into(), zero()).map(|unit| Self {
            origin: origin.into(),
            dir: unit,
        })
    }

    /// Returns the smallest (positive) `t` such that `P :=`[`self.at(t)`](Self::at)
    /// (approximately) intersects with `mirror`, and the direction space
    /// of the tangent to `mirror` at `P`, if any.
    ///
    /// It is possible that [`ray.at(t)`](Self::at) lands slightly beyond `mirror`, due to roundoff errors,
    /// making the `ray` bump, roughly, into `P` again, in it's next reflection, sometimes
    /// resulting in it traveling "through" `mirror`, when it should "move away" from it.
    ///
    /// To mitigate this, one can require `ray` to travel a certain, usually very small, strictly
    /// positive, distance (`eps`) before being reflected again, discarding intersections whose
    /// distance to `ray` is less than `eps`, hopefully avoiding the previous point in it's path.
    ///
    /// `eps` must be large enough to accomodate the precision errors of `S`
    /// (e. g. lower values are more acceptable for `f64` than for `f32`) but small enough
    /// to make sure `ray` doesn't ignore an intersection it shouldn't.
    ///
    /// Note that, actually, `eps.abs()` will be used, preventing `ray` from traveling
    /// "negative" distances. An `eps` of `0` is generally useless, and often results
    /// in incorrect results. The behaviour for `eps = NAN` or `inf` is unspecified, but, usually,
    /// results in a return value of `None` regardless of anything else.
    #[inline]
    #[must_use]
    pub fn closest_intersection(
        &self,
        mirror: &(impl Mirror<D, Scalar = S> + ?Sized),
        eps: S::RealField,
    ) -> Option<(S, Hyperplane<S, D>)> {
        let mut ctx = SimulationCtx::new(self, eps);
        mirror.add_tangents(&mut ctx);
        ctx.reset_closest()
    }
}

impl<S, const D: usize> Ray<S, D> {
    #[inline]
    #[must_use]
    pub fn new_unit_dir(origin: impl Into<SVector<S, D>>, dir: Unit<SVector<S, D>>) -> Self {
        Self {
            origin: origin.into(),
            dir,
        }
    }

    #[inline]
    #[must_use]
    /// Does not normalize `dir`
    pub fn new_unchecked_dir(
        origin: impl Into<SVector<S, D>>,
        dir: impl Into<SVector<S, D>>,
    ) -> Self {
        Self {
            origin: origin.into(),
            dir: Unit::new_unchecked(dir.into()),
        }
    }
}

impl<S: SimdComplexField, const D: usize> Ray<S, D> {
    /// Reflect [`self.dir`](Self::dir) w.r.t. `dir`, using [`Hyperplane::reflect_unit`].
    #[inline]
    pub fn reflect_dir(&mut self, dir: &Hyperplane<S, D>) {
        self.dir = dir.reflect_unit(&self.dir);
    }

    /// Like [`Self::reflect_dir`] but uses [`Hyperplane::reflect_unit_optimised`] instead of
    /// [`Hyperplane::reflect_unit`] internally.
    #[inline]
    pub fn reflect_dir_optimised(&mut self, dir: &Hyperplane<S, D>) {
        self.dir = dir.reflect_unit_optimised(&self.dir);
    }

    /// Translates [`self.origin`](Self::origin) by [`self.dir`](Self::dir)`* t`.
    #[inline]
    pub fn advance(&mut self, t: S) {
        self.origin += self.dir.as_ref() * t;
    }

    /// Returns [`self.origin`](Self::origin), translated by [`self.dir`](Self::dir)`* t`.
    #[inline]
    #[must_use]
    pub fn at(&self, t: S) -> SVector<S, D> {
        &self.origin + self.dir.as_ref() * t
    }
}

pub struct SimulationCtx<'a, S: ComplexField, const D: usize> {
    ray: &'a Ray<S, D>,
    closest: Option<(S, Hyperplane<S, D>)>,
    // garanteed to be positive
    epsilon: S::RealField,
}

impl<'a, S: ComplexField, const D: usize> SimulationCtx<'a, S, D> {
    #[inline]
    #[must_use]
    fn new(ray: &'a Ray<S, D>, epsilon: S::RealField) -> Self {
        Self {
            ray,
            epsilon: epsilon.abs(),
            closest: None,
        }
    }

    /// Stores `dist`, and `tangent_direction` along with it,
    /// if it's positive and smaller than the `dist` stored internally.
    pub fn add_tangent(&mut self, dist: S, tangent_direction: Hyperplane<S, D>) {
        let d = dist.clone().real();

        if d >= self.epsilon
            && self
                .closest
                .as_ref()
                .map_or(true, |(t, _)| t.clone().real() > d)
        {
            self.closest = Some((dist, tangent_direction));
        }
    }

    #[inline]
    #[must_use]
    pub const fn ray(&self) -> &Ray<S, D> {
        self.ray
    }

    #[inline]
    fn reset_closest(&mut self) -> Option<(S, Hyperplane<S, D>)> {
        self.closest.take()
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
/// `D` is expected to be non-zero. The behavior is unspecified (unexpected panics) if `D == 0`.
///
/// `D` could have been an associated constant, the same way `Scalar` is an associated type,
/// but, lack of `#[feature(generic_const_exprs)]` makes this difficult.
pub trait Mirror<const D: usize> {
    type Scalar: ComplexField;
    /// Adds the tangents to this mirror, at the points of intersection
    /// between it and a given ray, in no particular order.
    ///
    /// The ray can be accessed through [`ctx.ray( )`](SimulationCtx::ray).
    ///
    /// Tangents can be added with [`ctx.add_tangent(...)`](SimulationCtx::add_tangent).
    ///
    /// Adds nothing if the ray doesn't intersect with the set that `self` represents.
    ///
    /// This method may add tangents intersecting with the ray at negative `t` values,
    /// making them "behind" the ray's origin, (`ray.at(t)` where `t < 0.0`), these will be
    /// properly discarded.
    ///
    /// This method is expected to be deterministic with respect to the ray,
    /// i. e. for every valid `ray`, calling this method any number of times, at any time,
    /// (without modifying `self` in between) should result in the exact same plane(s) being
    /// reported to `ctx`, regardless of any internal/external state. Effectively making this
    /// method behave like a mathemiatical function. Thus implementors of this trait are advised
    /// to not make this method read/mutate any state that can have an effect on the planes
    /// reported to `ctx`, or their number.
    fn add_tangents(&self, ctx: &mut SimulationCtx<Self::Scalar, D>);
}

use impl_trait_for_tuples::impl_for_tuples;

#[impl_for_tuples(1, 16)]
impl<S: ComplexField, const D: usize> Mirror<D> for T {
    for_tuples!( where #( T: Mirror<D, Scalar = S> )* );
    type Scalar = S;

    #[inline]
    fn add_tangents(&self, ctx: &mut SimulationCtx<Self::Scalar, D>) {
        for_tuples!( #( T.add_tangents(ctx); )* );
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
#[cfg(feature = "alloc")]
impl<const D: usize, T: Mirror<D> + ?Sized> Mirror<D> for Box<T> {
    type Scalar = T::Scalar;
    #[inline]
    fn add_tangents(&self, ctx: &mut SimulationCtx<Self::Scalar, D>) {
        self.deref().add_tangents(ctx);
    }
}

#[cfg(feature = "alloc")]
impl<const D: usize, T: Mirror<D> + ?Sized> Mirror<D> for Arc<T> {
    type Scalar = T::Scalar;
    #[inline]
    fn add_tangents(&self, ctx: &mut SimulationCtx<Self::Scalar, D>) {
        self.deref().add_tangents(ctx);
    }
}

#[cfg(feature = "alloc")]
impl<const D: usize, T: Mirror<D> + ?Sized> Mirror<D> for Rc<T> {
    type Scalar = T::Scalar;
    #[inline]
    fn add_tangents(&self, ctx: &mut SimulationCtx<Self::Scalar, D>) {
        self.deref().add_tangents(ctx);
    }
}

#[cfg(feature = "alloc")]
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

#[derive(Debug, Clone)]
pub struct RayPath<'a, const D: usize, M: Mirror<D> + ?Sized> {
    pub ray: Ray<M::Scalar, D>,
    /// Minimum travel distance between each reflection of the ray.
    ///
    /// See [`Ray::closest_intersection`] for more info.
    pub eps: <M::Scalar as ComplexField>::RealField,
    pub mirror: &'a M,
}

impl<'a, const D: usize, M: Mirror<D> + ?Sized> Iterator for RayPath<'a, D, M> {
    type Item = SVector<M::Scalar, D>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let ray = &mut self.ray;
        ray.closest_intersection(self.mirror, self.eps.clone())
            .map(|(dist, direction)| {
                ray.advance(dist);
                ray.reflect_dir(&direction);
                ray.origin.clone()
            })
    }
}

/// Checks if adding `new_pt` to `path` results in a ray doing a potential infinite loop.
/// `eps` is used for comparisons.
#[inline]
#[must_use]
pub fn loop_index<const D: usize, S: ComplexField>(
    path: &[SVector<S, D>],
    new_pt: &SVector<S, D>,
    eps: &S::RealField,
) -> Option<usize> {
    path.split_last().and_then(|(last_pt, points)| {
        let current_dir = Unit::new_normalize(new_pt - last_pt).into_inner();

        points.windows(2).enumerate().find_map(|(i, window)| {
            // ugly, but `slice::array_windows` is unstable
            let [this_pt, next_pt] = window else {
                // because window.len() is always 2
                unreachable!()
            };

            let impact_dir = Unit::new_normalize(next_pt - this_pt).into_inner();

            ((new_pt - next_pt).norm() <= *eps && (impact_dir - &current_dir).norm() <= *eps)
                .then_some(i)
        })
    })
}
