use nalgebra::{ComplexField, RealField, SimdComplexField};

use super::*;

/// A (D-1)-simplex in D-dimensional (euclidean) space
/// (A line segment in 2D space, a triangle in 3D space, etc...)
#[derive(Clone, Debug, PartialEq)]
pub struct Simplex<S, const D: usize> {
    /// The plane this mirror belongs to, the unused first vector is used as the starting point
    plane: HyperPlaneBasis<S, D>,
    /// The same plane, but represented with an orthonormal basis, useful for orthogonal symmetries
    orthonormalised: HyperPlaneBasisOrtho<S, D>,
}

pub type Triangle<S> = Simplex<S, 3>;
pub type LineSegment<S> = Simplex<S, 2>;

impl<S: ComplexField, const D: usize> Simplex<S, D> {
    #[inline]
    pub fn try_new(points: [impl Into<SVector<S, D>>; D]) -> Option<Self> {
        let mut vectors: [SVector<_, D>; D] = points.map(Into::into);
        let (v0, basis) = vectors.split_first_mut().unwrap();

        basis.iter_mut().for_each(|v| *v -= v0.clone());

        HyperPlaneBasis::new(vectors).map(|(plane, orthonormalised)| Self {
            plane,
            orthonormalised,
        })
    }

    #[inline]
    pub fn new(vectors: [impl Into<SVector<S, D>>; D]) -> Self {
        Self::try_new(vectors).unwrap()
    }
}

impl<S, const D: usize> Simplex<S, D> {
    #[inline]
    pub const fn inner_plane(&self) -> &HyperPlaneBasis<S, D> {
        &self.plane
    }
}

impl<S: ComplexField, const D: usize, U> TryFrom<[U; D]> for Simplex<S, D>
where
    SVector<S, D>: From<U>,
{
    type Error = ();

    #[inline]
    fn try_from(vectors: [U; D]) -> Result<Self, Self::Error> {
        Self::try_new(vectors).ok_or(())
    }
}

impl<S: SimdComplexField, const D: usize> Simplex<S, D> {
    #[inline]
    pub fn vertices(&self) -> [SVector<S, D>; D] {
        let mut vertices = self.inner_plane().vectors_raw().clone();
        let (v0, vectors) = vertices.split_first_mut().unwrap();

        vectors.iter_mut().for_each(|v| *v += v0.clone());
        vertices
    }
}

impl<S: RealField, const D: usize> Mirror<D> for Simplex<S, D> {
    type Scalar = S;
    fn add_tangents(&self, ctx: &mut SimulationCtx<Self::Scalar, D>) {
        let p = self.inner_plane();

        let ray = ctx.ray();

        let intersection_coords = p.intersection_coordinates(ray, p.v0());

        if let Some(t) = intersection_coords.as_ref().and_then(|v| {
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

            Some(distance)
        }) {
            ctx.add_tangent(Plane {
                // We could return `self.plane.v0()`, but since we already calculated `t`,
                // we might as well save the simulation runner some work, and return that
                intersection: Intersection::Distance(t.clone()),
                direction: HyperPlane::Plane(self.orthonormalised.clone()),
            });
        }
    }
}
