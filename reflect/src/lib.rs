#![no_std]

extern crate alloc;

use alloc::{rc::Rc, sync::Arc, vec::Vec, boxed::Box};
use core::ops::Deref;

pub use nalgebra;

use nalgebra::{SMatrix, SVector, Unit};

pub type Float = f64;

pub struct SimulationCtx<const D: usize> {
    pub(crate) ray: Ray<D>,
    pub(crate) closest: Option<(Float, TangentSpace<D>)>,
}

impl<const D: usize> SimulationCtx<D> {

    #[inline]
    pub fn ray(&self) -> &Ray<D> {
        &self.ray
    }

    pub fn add_tangent(&mut self, tangent: TangentPlane<D>) {
        let d = tangent.try_ray_intersection(self.ray()).expect("a mirror returned a plane parallel to the ray: aborting");

        const E: Float = Float::EPSILON * 64.0;

        if d >= E && self.closest.as_ref().map(|(t, _)| *t > d).unwrap_or(true) {
            self.closest = Some((d, tangent.direction));
        }
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
    fn reflect_dir(&mut self, tangent: &TangentSpace<D>) {
        self.direction = tangent.reflect_unit(self.direction);
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

/// An affine hyperplane
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AffineHyperPlane<const D: usize> {
    /// See [`AffineHyperPlane::new`] for info on the layout of this field
    vectors: [SVector<Float, D>; D],
}

impl<const D: usize> AffineHyperPlane<D> {
    /// The first element of the `vectors` array is the plane's "starting point" (i. e. `v0`).
    ///
    /// The remaining `N-1` vectors are a free family spanning it's direction hyperplane
    ///
    /// Returns `None` if the provided family isn't free.
    ///
    /// Note that an expression like `[T ; N - 1]` is locked under `#[feature(const_generic_exprs)]`.
    #[inline]
    pub fn new(vectors: [SVector<Float, D>; D]) -> Option<(Self, AffineHyperPlaneOrtho<D>)> {
        let mut orthonormalized = vectors;
        (SVector::orthonormalize(&mut orthonormalized[1..]) == D - 1).then_some((
            Self { vectors },
            AffineHyperPlaneOrtho {
                vectors: orthonormalized,
            },
        ))
    }

    /// A reference to the plane's starting point
    #[inline]
    pub fn v0(&self) -> &SVector<Float, D> {
        self.vectors.first().unwrap()
    }

    /// A mutable reference to the plane's starting point
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
    /// the `intersection` of the given `ray` and `self`.
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
        starting_pt: &SVector<Float, D>,
    ) -> Option<SVector<Float, D>> {
        let mut a = SMatrix::<Float, D, D>::from_columns(&self.vectors);
        a.set_column(0, ray.direction.as_ref());

        a.try_inverse_mut()
            // a now contains a^-1
            .then(|| {
                let mut v = a * (ray.origin - starting_pt);
                let first = &mut v[0];
                *first = -*first;
                v
            })
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
/// An affine hyperplane, like [`AffineHyperPlane`], but the basis stored is garanteed
/// to be orthonormal, enabling projections and symmetries efficiently.
pub struct AffineHyperPlaneOrtho<const D: usize> {
    /// See [`AffineHyperPlane::new`] for info on the layout of this field
    vectors: [SVector<Float, D>; D],
}

impl<const D: usize> AffineHyperPlaneOrtho<D> {
    /// A reference to the plane's starting point
    #[inline]
    pub fn v0(&self) -> &SVector<Float, D> {
        self.vectors.first().unwrap()
    }

    /// A mutable reference to the plane's starting point
    #[inline]
    pub fn v0_mut(&mut self) -> &mut SVector<Float, D> {
        &mut self.vectors[0]
    }

    /// A reference to an orthonormal basis of the plane's direction hyperplane.
    ///
    /// The returned slice is garanteed to be of length `D - 1`.
    #[inline]
    pub fn basis(&self) -> &[SVector<Float, D>] {
        &self.vectors[1..]
    }

    /// Returns the orthogonal projection of `v` w.r.t. this plane's direction subspace.
    #[inline]
    pub fn orthogonal_projection(&self, v: SVector<Float, D>) -> SVector<Float, D> {
        self.basis().iter().map(|e| v.dot(e) * e).sum()
    }

    /// Returns the point in this plane whose distance with `p` is smallest.
    #[inline]
    pub fn orthogonal_point_projection(&self, p: SVector<Float, D>) -> SVector<Float, D> {
        let v0 = self.v0();
        let v = p - v0;
        v0 + self.orthogonal_projection(v)
    }

    /// Returns a vector `[t_1, ..., t_d]` whose coordinates represent
    /// the `intersection` of the given `ray` and `self`.
    ///
    /// If it exists, the following holds:
    ///
    /// `intersection = ray.origin + t_1 * ray.direction` and,
    ///
    /// let `[v_2, ..., v_d]` be the orthonormal basis of `self`'s direction space ( returned by `self.basis()`)
    ///
    /// `interserction = plane.origin + sum for k in [2 ; n] t_k * v_k`
    #[inline]
    pub fn intersection_coordinates(
        &self,
        ray: &Ray<D>,
        starting_pt: &SVector<Float, D>,
    ) -> Option<SVector<Float, D>> {
        let mut a = SMatrix::<Float, D, D>::from_columns(&self.vectors);
        a.set_column(0, ray.direction.as_ref());

        a.try_inverse_mut()
            // a now contains a^-1
            .then(|| {
                let mut v = a * (ray.origin - starting_pt);
                let first = &mut v[0];
                *first = -*first;
                v
            })
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
/// Different ways of representing a hyperplane in `D`-dimensional euclidean space
pub enum TangentSpace<const D: usize> {
    /// Only the basis of this object's direction hyperplane is used.
    ///
    /// The starting point will be ignored by [`Simulation`]s and can be arbitrary.
    Plane(AffineHyperPlaneOrtho<D>),
    Normal(Unit<SVector<Float, D>>),
}

impl<const D: usize> TangentSpace<D> {
    /// Reflect a vector w.r.t this hyperplane
    #[inline]
    pub fn reflect(&self, v: SVector<Float, D>) -> SVector<Float, D> {
        match self {
            TangentSpace::Plane(plane) => 2.0 * plane.orthogonal_projection(v) - v,
            TangentSpace::Normal(normal) => {
                let n = normal.as_ref();
                v - 2.0 * v.dot(n) * n
            }
        }
    }

    /// Reflect a unit vector w.r.t. this hyperplane
    #[inline]
    pub fn reflect_unit(&self, v: Unit<SVector<Float, D>>) -> Unit<SVector<Float, D>> {
        // SAFETY: orthogonal symmetry preserves euclidean norms
        // This function is supposed to be unsafe, why nalgebra? why?
        Unit::new_unchecked(self.reflect(v.into_inner()))
    }

    /// Return the distance `t` such that `ray.at(t)` intersects with the affine hyperplane
    /// whose direction space is `self`, and whose starting point is `p`.
    ///
    /// Returns `None` if `ray` is parallel to `self`
    #[inline]
    pub fn try_ray_intersection(&self, p: &SVector<Float, D>, ray: &Ray<D>) -> Option<Float> {
        match self {
            TangentSpace::Plane(plane) => plane.intersection_coordinates(ray, p).map(|v| v[0]),
            TangentSpace::Normal(normal) => {
                let u = ray.direction.dot(normal);
                (u.abs() > Float::EPSILON).then(|| (p - ray.origin).dot(normal) / u)
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
/// Different ways of representing a starting point of an affine hyperplane in `D`-dimensional euclidean space
///
/// It may be provided directly or be at a certain distance from a ray.
pub enum Intersection<const D: usize> {
    /// If a [`Mirror`] returns `Intersection::Distance(t)` when calculating it's intersections with a `ray`, then `ray.at(t)` belongs to the returned tangent (hyper)plane.
    ///
    /// This is useful if `t` is easy to calculate. as the `Simulation will not have to calculate it itself`
    Distance(Float),
    /// Note that the point in this vector doesn't necessarily intersect with the ray, but it serves as a starting/center point for the plane represented by [TangentPlane]
    StartingPoint(SVector<Float, D>),
}

#[derive(Clone, Copy, Debug, PartialEq)]
/// Different ways of representing an _affine_ hyperplane in `D`-dimensional euclidean space
pub struct TangentPlane<const D: usize> {
    pub intersection: Intersection<D>,
    pub direction: TangentSpace<D>,
}

impl<const D: usize> TangentPlane<D> {
    /// Reflect a vector w.r.t this tangent plane's direction hyperplane
    #[inline]
    pub fn reflect(&self, v: SVector<Float, D>) -> SVector<Float, D> {
        self.direction.reflect(v)
    }

    /// Reflect a unit vector w.r.t. this tangent plane's direction hyperplane
    #[inline]
    pub fn reflect_unit(&self, v: Unit<SVector<Float, D>>) -> Unit<SVector<Float, D>> {
        self.direction.reflect_unit(v)
    }

    /// Return the distance `t` such that `ray.at(t)` intersects with this tangent plane
    ///
    /// Returns `None` if `ray` is parallel to `self`
    #[inline]
    pub fn try_ray_intersection(&self, ray: &Ray<D>) -> Option<Float> {
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
// `D` could have been an associated constant but, lack of
// `#[feature(generic_const_exprs)]` screws us over, once again.
pub trait Mirror<const D: usize> {
    /// Adds a number of affine (hyper)planes, tangent to this mirror, at the
    /// points of intersection between it and the `ray`, in no particular order.
    /// 
    /// The `ray` can be accessed through `ctx.ray()`
    ///
    /// The ray is expected to "bounce" off the plane closest to it.
    ///
    /// Here, "bounce" refers to the process of:
    ///     - Moving forward to the intersection.
    ///     - Then, orthogonally reflecting it's direction vector with
    ///       respect to the direction hyperplane.
    ///
    /// Appends nothing if the ray doesn't intersect with the mirror that `self` represents.
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
        self.iter()
            .for_each(|mirror| mirror.add_tangents(ctx))
    }
}

impl<const N: usize, const D: usize, T: Mirror<D>> Mirror<D> for [T; N] {
    #[inline]
    fn add_tangents(&self, ctx: &mut SimulationCtx<D>) {
        self.as_slice().add_tangents(ctx)
    }
}

// It's clear that all these impls use the `Deref` trait, but writing a blanket impl over all types implementing `Deref`
// makes the trait unusable downstream

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
        self.deref().add_tangents(ctx)
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

#[derive(Clone, Debug, PartialEq, Default)]
pub struct RayPath<const D: usize> {
    points: Vec<SVector<Float, D>>,
    loop_start: Option<usize>,
    divergence_direction: Option<Unit<SVector<Float, D>>>,
}

impl<const D: usize> RayPath<D> {
    #[inline]
    pub fn all_points_raw(&self) -> &[SVector<Float, D>] {
        self.points.as_slice()
    }

    #[inline]
    /// returns a pair (non_loop_points, loop_points)
    pub fn all_points(&self) -> (&[SVector<Float, D>], &[SVector<Float, D>]) {
        self.points
            .split_at(self.loop_start.unwrap_or(self.points.len()))
    }

    #[inline]
    // name bikeshedding welcome
    pub fn non_loop_points(&self) -> &[SVector<Float, D>] {
        &self.points[..self.loop_start.unwrap_or(self.points.len())]
    }

    #[inline]
    pub fn loop_points(&self) -> &[SVector<Float, D>] {
        self.loop_start
            .map(|index| &self.points[index..])
            .unwrap_or_default()
    }

    #[inline]
    pub fn divergence_direction(&self) -> Option<&Unit<SVector<Float, D>>> {
        self.divergence_direction.as_ref()
    }

    #[inline]
    pub fn push_point(&mut self, pt: SVector<Float, D>) {
        self.points.push(pt);
    }

    #[inline]
    pub fn causes_loop_at(&self, pt: SVector<Float, D>, epsilon: Float) -> Option<usize> {
        self.points.split_last().and_then(|(last_pt, points)| {
            points.windows(2).enumerate().find_map(|(i, window)| {
                // ugly, but `slice::array_windows` is unstable
                let [this_pt, next_pt] = window else {
                    // because window.len() is always 2
                    unreachable!()
                };
                ((last_pt - this_pt).norm() <= epsilon && (pt - next_pt).norm() < epsilon)
                    .then_some(i)
            })
        })
    }

    /// Attempts to push a point to the path. If it causes an infinite loop, aborts,
    /// registers the section of the path that loops, and returns `false`
    #[inline]
    pub fn try_push_point(&mut self, pt: SVector<Float, D>, epsilon: Float) -> bool {
        let maybe_loop_index = self.causes_loop_at(pt, epsilon);

        if let Some(loop_index) = maybe_loop_index {
            self.loop_start = Some(loop_index);
        } else {
            self.push_point(pt);
        }

        maybe_loop_index.is_none()
    }

    #[inline]
    pub fn set_divergence_direction(&mut self, dir: Unit<SVector<Float, D>>) -> bool {
        let first_time = self.divergence_direction.is_none();
        self.divergence_direction = Some(dir);
        first_time
    }
}

pub struct Simulation<M, R> {
    pub rays: R,
    pub mirror: M,
}

impl<const D: usize, M: Mirror<D>, R: IntoIterator<Item = Ray<D>>> Simulation<M, R> {
    #[inline]
    pub fn get_ray_paths(self, reflection_limit: usize) -> impl Iterator<Item = RayPath<D>> {
        let Self { rays, mirror } = self;
        rays.into_iter().map(move |ray| {

            let mut ctx = SimulationCtx { ray, closest: None };
            let mut ray_path = RayPath::default();
            ray_path.push_point(ray.origin);

            for _n in 0..reflection_limit {
                mirror.add_tangents(&mut ctx);

                if let Some((distance, space)) = ctx.closest.take() {
                    ctx.ray.advance(distance);
                    if !ray_path.try_push_point(ctx.ray.origin, Float::EPSILON * 64.0) {
                        break;
                    }
                    ctx.ray.reflect_dir(&space)
                } else {
                    ray_path.set_divergence_direction(ctx.ray.direction);
                    break;
                }
            }
            ray_path
        })
    }
}
