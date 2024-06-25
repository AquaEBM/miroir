use core::ops::AddAssign;

use nalgebra::{ComplexField, RealField};

use super::*;

/// A (D-1)-simplex in D-dimensional (euclidean) space
/// (A line segment in 2D space, a triangle in 3D space, etc...)
#[derive(Clone, Debug, PartialEq)]
pub struct Simplex<S, const D: usize> {
    /// The plane this mirror belongs to, the unused first vector is used as the starting point
    plane: HyperplaneBasis<S, D>,
    /// The same plane, but represented with an orthonormal basis, useful for orthogonal symmetries
    orthonormalised: HyperplaneBasisOrtho<S, D>,
}

pub type Triangle<S> = Simplex<S, 3>;
pub type LineSegment<S> = Simplex<S, 2>;

impl<S: ComplexField, const D: usize> Simplex<S, D> {
    /// Attempts to create a `D-1`-simplex in using an array of `D` affinely independent points.
    /// 
    /// Returns `None` if they are affinely dependent.
    /// 
    /// # Panics
    /// 
    /// if `D == 0`
    #[inline]
    pub fn try_new(points: [impl Into<SVector<S, D>>; D]) -> Option<Self> {
        let mut vectors: [SVector<_, D>; D] = points.map(Into::into);
        let (v0, basis) = vectors.split_first_mut().unwrap();

        basis.iter_mut().for_each(|v| *v -= v0.clone());

        HyperplaneBasis::try_new(vectors).map(|(plane, orthonormalised)| Self {
            plane,
            orthonormalised,
        })
    }

    /// A panicking version of [`Self::try_new`]
    /// 
    /// # Panics
    /// 
    /// if `D == 0` or, the points in `points` are affinely dependent
    #[inline]
    pub fn new(points: [impl Into<SVector<S, D>>; D]) -> Self {
        Self::try_new(points).unwrap()
    }
}

impl<S, const D: usize> Simplex<S, D> {
    #[inline]
    #[must_use]
    pub const fn inner_plane(&self) -> &HyperplaneBasis<S, D> {
        &self.plane
    }

    /// It is worth noting that translating the `v0` vector of the returned value:
    /// ```ignore
    /// *self.inner_plane_mut().v0_mut() += v;
    /// ```
    /// 
    /// Effectively translates this whole simplex.
    #[inline]
    #[must_use]
    pub fn inner_plane_mut(&mut self) -> &mut HyperplaneBasis<S, D> {
        &mut self.plane
    }

    #[inline]
    #[must_use]
    pub const fn inner_plane_ortho(&self) -> &HyperplaneBasisOrtho<S, D> {
        &self.orthonormalised
    }
}

impl<S: ComplexField, const D: usize, U> TryFrom<[U; D]> for Simplex<S, D>
where
    SVector<S, D>: From<U>,
{
    type Error = ();

    #[inline]
    fn try_from(points: [U; D]) -> Result<Self, Self::Error> {
        Self::try_new(points).ok_or(())
    }
}

impl<S, const D: usize> Simplex<S, D>
where
    SVector<S, D>: AddAssign + Clone,
{
    /// Returns the vertices of this simplex
    /// 
    /// # Panics
    /// 
    /// if `D == 0`
    #[inline]
    pub fn vertices(&self) -> [SVector<S, D>; D] {
        let mut vertices = self.inner_plane().vectors_raw().clone();
        let (v0, vectors) = vertices.split_first_mut().unwrap();

        vectors.iter_mut().for_each(|v| *v += v0.clone());
        vertices
    }
}

impl<S: RealField, const D: usize> Simplex<S, D> {
    /// Returns the distance `d` such that [`ray.at(d)`](Ray::at) intersects with `self`
    #[inline]
    pub fn intersection(&self, ray: &Ray<S, D>) -> Option<S> {
        let p = self.inner_plane();

        let intersection_coords = p.intersection_coordinates(ray, p.v0());

        intersection_coords.as_ref().and_then(|v| {
            let (distance, plane_coords) = v.as_slice().split_first().unwrap();
            let mut sum = S::zero();
            for coord in plane_coords {
                if coord.is_negative() {
                    return None;
                }
                sum += coord.clone();
            }

            if sum > S::one() {
                return None;
            }

            Some(distance.clone())
        })
    }
}

impl<S: RealField, const D: usize> Mirror<D> for Simplex<S, D> {
    type Scalar = S;
    fn add_tangents(&self, ctx: &mut SimulationCtx<Self::Scalar, D>) {
        if let Some(t) = self.intersection(ctx.ray()) {
            ctx.add_tangent(t, Hyperplane::Plane(self.inner_plane_ortho().clone()));
        }
    }
}
