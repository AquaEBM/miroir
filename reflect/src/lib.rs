#![no_std]

extern crate alloc;

use alloc::{boxed::Box, rc::Rc, sync::Arc, vec::Vec};
use core::ops::Deref;

pub use nalgebra;

use nalgebra::{SMatrix, SVector, Unit};

pub type Float = f64;

#[derive(Clone, PartialEq, Debug)]
pub struct SimulationCtx<const D: usize> {
    pub(crate) ray: Ray<D>,
    pub(crate) closest: Option<(Float, HyperPlane<D>)>,
}

impl<const D: usize> SimulationCtx<D> {
    #[inline]
    fn new(ray: Ray<D>) -> Self {
        Self { ray, closest: None }
    }

    #[inline]
    pub fn ray(&self) -> &Ray<D> {
        &self.ray
    }

    #[inline]
    pub fn add_tangent(&mut self, tangent: Plane<D>) {
        let d = tangent
            .try_ray_intersection(self.ray())
            .expect("a mirror returned a plane parallel to the ray: aborting");

        const E: Float = Float::EPSILON * 64.0;

        if d >= E && self.closest.as_ref().map(|(t, _)| *t > d).unwrap_or(true) {
            self.closest = Some((d, tangent.direction));
        }
    }

    #[inline]
    fn next_ray<M: Mirror<D> + ?Sized>(&mut self, mirror: &M) -> Option<&Ray<D>> {
        mirror.add_tangents(self);

        self.closest.take().map(move |(dist, dir_space)| {
            self.ray.advance(dist);
            self.ray.reflect_dir(&dir_space);
            &self.ray
        })
    }
}

/// A light ray, represented as a half-line.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ray<const D: usize> {
    /// The starting point of the half-line
    pub origin: SVector<Float, D>,
    /// the direction of the half-line
    pub direction: Unit<SVector<Float, D>>,
}

impl<const D: usize> Ray<D> {
    /// Reflect the ray's direction with respect to the given hyperplane
    #[inline]
    pub fn reflect_dir(&mut self, dir_space: &HyperPlane<D>) {
        self.direction = dir_space.reflect_unit(self.direction);
    }

    /// Move the ray's position forward (or backward if t < 0.0) by `t`
    #[inline]
    pub fn advance(&mut self, t: Float) {
        self.origin += t * self.direction.as_ref();
    }

    /// Get the point at distance `t` (can be negative) from the ray's origin
    #[inline]
    pub fn at(&self, t: Float) -> SVector<Float, D> {
        self.origin + self.direction.as_ref() * t
    }
}

/// A hyperplane
#[derive(Clone, Debug, PartialEq)]
pub struct HyperPlaneBasis<const D: usize> {
    /// See [`AffineHyperPlane::new`] for info on the layout of this field
    vectors: [SVector<Float, D>; D],
}

impl<const D: usize> HyperPlaneBasis<D> {
    /// The first element of the `vectors` can be completely arbitrary and will be ignored
    ///
    /// The remaining `D - 1` vectors are a free family spanning this hyperplane
    ///
    /// Returns `None` if the family isn't free.
    ///
    /// Note that an expression like `[T ; D - 1]` is locked under `#[feature(const_generic_exprs)]`.
    #[inline]
    pub fn new(vectors: [SVector<Float, D>; D]) -> Option<(Self, HyperPlaneBasisOrtho<D>)> {
        let mut orthonormalized = vectors;
        (SVector::orthonormalize(&mut orthonormalized[1..]) == D - 1).then_some((
            Self { vectors },
            HyperPlaneBasisOrtho {
                vectors: orthonormalized,
            },
        ))
    }

    /// A reference to the unused first vector in the array.
    #[inline]
    pub fn v0(&self) -> &SVector<Float, D> {
        &self.vectors[0]
    }

    /// A mutable reference to the unused first vector in the array.
    #[inline]
    pub fn v0_mut(&mut self) -> &mut SVector<Float, D> {
        &mut self.vectors[0]
    }

    /// A reference to the basis of the plane's direction hyperplane.
    ///
    /// The returned slice is garanteed to be of length `D - 1`.
    #[inline]
    pub fn basis(&self) -> &[SVector<Float, D>] {
        &self.vectors[1..]
    }

    #[inline]
    pub fn vectors_raw(&self) -> &[SVector<Float, D>; D] {
        &self.vectors
    }

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
    pub fn intersection_coordinates(
        &self,
        ray: &Ray<D>,
        v0: &SVector<Float, D>,
    ) -> Option<SVector<Float, D>> {
        let mut a = SMatrix::<Float, D, D>::from_columns(&self.vectors);
        a.set_column(0, ray.direction.as_ref());

        a.try_inverse_mut()
            // a now contains a^-1
            .then(|| {
                let mut v = a * (ray.origin - v0);
                let first = &mut v[0];
                *first = -*first;
                v
            })
    }
}

/// A hyperplane, like [`HyperPlaneBasis`], but the basis stored is garanteed
/// to be orthonormal, efficiently enabling projections and symmetries.
#[derive(Clone, Debug, PartialEq)]
pub struct HyperPlaneBasisOrtho<const D: usize> {
    /// See [`HyperPlaneBasis::new`] for info on the layout of this field
    vectors: [SVector<Float, D>; D],
}

impl<const D: usize> HyperPlaneBasisOrtho<D> {
    /// A reference to the unused first vector in the array.
    #[inline]
    pub fn v0(&self) -> &SVector<Float, D> {
        &self.vectors[0]
    }

    /// A mutable reference to the unused first vector in the array.
    #[inline]
    pub fn v0_mut(&mut self) -> &mut SVector<Float, D> {
        &mut self.vectors[0]
    }

    /// A reference to an orthonormal basis of `self`
    ///
    /// The returned slice is garanteed to be of length `D - 1`.
    #[inline]
    pub fn basis(&self) -> &[SVector<Float, D>] {
        &self.vectors[1..]
    }

    /// Returns the orthogonal projection of `v` w.r.t. `self`
    #[inline]
    pub fn project(&self, v: SVector<Float, D>) -> SVector<Float, D> {
        self.basis().iter().map(|e| v.dot(e) * e).sum()
    }

    /// Returns the point in this plane whose distance with `p` is smallest.
    #[inline]
    pub fn closest_point_to_plane(
        &self,
        v0: &SVector<Float, D>,
        p: SVector<Float, D>,
    ) -> SVector<Float, D> {
        let v = p - v0;
        v0 + self.project(v)
    }

    /// Returns a vector `[t_1, ..., t_d]` whose coordinates represent
    /// the `intersection` of the given `ray` and the affine hyperplane
    /// starting at `v0` whose direction space is `self`.
    ///
    /// If it exists, the following holds:
    ///
    /// `intersection = ray.origin + t_1 * ray.direction` and,
    ///
    /// let `[v_2, ..., v_d]` be the orthonormal basis of `self` returned by `self.basis()`
    ///
    /// `interserction = v0 + sum for k in [2 ; n] t_k * v_k`
    #[inline]
    pub fn intersection_coordinates(
        &self,
        ray: &Ray<D>,
        v0: &SVector<Float, D>,
    ) -> Option<SVector<Float, D>> {
        let mut a = SMatrix::<Float, D, D>::from_columns(&self.vectors);
        a.set_column(0, ray.direction.as_ref());

        a.try_inverse_mut()
            // a now contains a^-1
            .then(|| {
                let mut v = a * (ray.origin - v0);
                let first = &mut v[0];
                *first = -*first;
                v
            })
    }
}

/// Different ways of representing a hyperplane
#[derive(Clone, Debug, PartialEq)]
pub enum HyperPlane<const D: usize> {
    Plane(HyperPlaneBasisOrtho<D>),
    Normal(Unit<SVector<Float, D>>),
}

impl<const D: usize> HyperPlane<D> {
    /// Reflect a vector w.r.t this hyperplane
    #[inline]
    pub(crate) fn reflect(&self, v: SVector<Float, D>) -> SVector<Float, D> {
        match self {
            HyperPlane::Plane(plane) => 2.0 * plane.project(v) - v,
            HyperPlane::Normal(normal) => {
                let n = normal.as_ref();
                v - 2.0 * v.dot(n) * n
            }
        }
    }

    /// Reflect a unit vector w.r.t. this hyperplane
    #[inline]
    pub(crate) fn reflect_unit(&self, v: Unit<SVector<Float, D>>) -> Unit<SVector<Float, D>> {
        // SAFETY: orthogonal symmetry preserves euclidean norms
        // This function is supposed to be unsafe, why nalgebra? why?
        Unit::new_unchecked(self.reflect(v.into_inner()))
    }

    /// Return the distance `t` such that `ray.at(t)` intersects with the affine
    /// hyperplane starting at `v0`, and whose direction space is `self`.
    ///
    /// Returns `None` if `ray` is parallel to `self`
    #[inline]
    pub(crate) fn try_ray_intersection(
        &self,
        v0: &SVector<Float, D>,
        ray: &Ray<D>,
    ) -> Option<Float> {
        match self {
            HyperPlane::Plane(plane) => plane.intersection_coordinates(ray, v0).map(|v| v[0]),
            HyperPlane::Normal(normal) => {
                let u = ray.direction.dot(normal);
                (u.abs() > Float::EPSILON).then(|| (v0 - ray.origin).dot(normal) / u)
            }
        }
    }
}

/// Different ways of representing a starting point of an affine hyperplane in `D`-dimensional euclidean space
///
/// It may be provided directly or be at a certain distance from a ray.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Intersection<const D: usize> {
    /// If a [`Mirror`] returns `Intersection::Distance(t)` when calculating it's intersections with a `ray`, then `ray.at(t)` belongs to the returned tangent (hyper)plane.
    ///
    /// This is useful if `t` is easy to calculate.
    Distance(Float),
    /// Note that the point in this vector doesn't necessarily intersect with the ray, but it serves as an offset point for the plane represented by [`Plane`]
    StartingPoint(SVector<Float, D>),
}

#[derive(Clone, Debug, PartialEq)]
/// Different ways of representing an _affine_ hyperplane in `D`-dimensional euclidean space
pub struct Plane<const D: usize> {
    pub intersection: Intersection<D>,
    pub direction: HyperPlane<D>,
}

impl<const D: usize> Plane<D> {
    /// Return the distance `t` such that `ray.at(t)` intersects with this tangent plane
    ///
    /// Returns `None` if `ray` is parallel to `self`
    #[inline]
    pub(crate) fn try_ray_intersection(&self, ray: &Ray<D>) -> Option<Float> {
        match &self.intersection {
            Intersection::Distance(t) => Some(*t),
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
    fn add_tangents(&self, ctx: &mut SimulationCtx<D>);
}

impl<const D: usize, T: Mirror<D>> Mirror<D> for [T] {
    #[inline]
    fn add_tangents(&self, ctx: &mut SimulationCtx<D>) {
        self.iter().for_each(|mirror| mirror.add_tangents(ctx))
    }
}

impl<const N: usize, const D: usize, T: Mirror<D>> Mirror<D> for [T; N] {
    #[inline]
    fn add_tangents(&self, ctx: &mut SimulationCtx<D>) {
        self.as_slice().add_tangents(ctx)
    }
}

// It's clear that all these impls use the `Deref` trait, but writing a blanket impl over all types implementing `Deref`
// makes it impossible to implement it for new types downstream.

impl<const D: usize, T: Mirror<D> + ?Sized> Mirror<D> for Box<T> {
    #[inline]
    fn add_tangents(&self, ctx: &mut SimulationCtx<D>) {
        self.deref().add_tangents(ctx)
    }
}

impl<const D: usize, T: Mirror<D> + ?Sized> Mirror<D> for Arc<T> {
    #[inline]
    fn add_tangents(&self, ctx: &mut SimulationCtx<D>) {
        self.deref().add_tangents(ctx)
    }
}

impl<const D: usize, T: Mirror<D> + ?Sized> Mirror<D> for Rc<T> {
    #[inline]
    fn add_tangents(&self, ctx: &mut SimulationCtx<D>) {
        self.deref().add_tangents(ctx)
    }
}

impl<const D: usize, T: Mirror<D>> Mirror<D> for Vec<T> {
    #[inline]
    fn add_tangents(&self, ctx: &mut SimulationCtx<D>) {
        self.as_slice().add_tangents(ctx)
    }
}

impl<'a, const D: usize, T: Mirror<D> + ?Sized> Mirror<D> for &'a T {
    #[inline]
    fn add_tangents(&self, ctx: &mut SimulationCtx<D>) {
        (*self).add_tangents(ctx)
    }
}

impl<'a, const D: usize, T: Mirror<D> + ?Sized> Mirror<D> for &'a mut T {
    #[inline]
    fn add_tangents(&self, ctx: &mut SimulationCtx<D>) {
        self.deref().add_tangents(ctx)
    }
}

pub struct RayPath<'a, const D: usize, M: ?Sized> {
    pub(crate) ctx: SimulationCtx<D>,
    pub(crate) mirror: &'a M,
}

impl<'a, const D: usize, M: Mirror<D> + ?Sized> RayPath<'a, D, M> {
    #[inline]
    pub fn new(mirror: &'a M, ray: Ray<D>) -> Self {
        Self {
            ctx: SimulationCtx::new(ray),
            mirror,
        }
    }

    #[inline]
    pub fn current_ray_dir(&self) -> Unit<SVector<Float, D>> {
        self.ctx.ray.direction
    }
}

impl<'a, const D: usize, M: Mirror<D> + ?Sized> Iterator for RayPath<'a, D, M> {
    type Item = SVector<Float, D>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.ctx.next_ray(self.mirror).map(|ray| ray.origin)
    }
}

#[inline]
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
