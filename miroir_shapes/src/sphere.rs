use nalgebra::{ComplexField, RealField};

use super::*;

/// All points at a certain distance (`radius`) from a certain vector (`center`)
/// where the distance here is the standard euclidean distance
// TODO: We can do other distances, can we?
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Sphere<S: ComplexField, const D: usize> {
    pub center: SVector<S, D>,
    radius: S::RealField,
    radius_sq: S::RealField,
}

impl<S: ComplexField, const D: usize> Sphere<S, D> {
    #[inline]
    #[must_use]
    pub fn new(center: impl Into<SVector<S, D>>, radius: impl Into<S::RealField>) -> Self {
        let radius = radius.into();
        Self {
            center: center.into(),
            radius: radius.clone().abs(),
            radius_sq: radius.clone() * radius,
        }
    }

    #[inline]
    #[must_use]
    pub fn radius(&self) -> &S::RealField {
        &self.radius
    }

    #[inline]
    pub fn set_radius(&mut self, r: S::RealField) {
        self.radius = r.clone();
        self.radius_sq = r.clone() * r;
    }

    #[inline]
    #[must_use]
    pub fn intersections(&self, ray: &Ray<SVector<S, D>>) -> Option<[S; 2]> {
        // substituting `V` for `P + t * D` in the sphere equation:
        // `||V - C||^2 = r^2` results in a quadratic equation in `t`.

        let v = &ray.pos - &self.center;

        let b = v.dotc(&ray.dir).real();
        let c = v.norm_squared() - self.radius_sq.clone();

        let delta = b.clone().mul_add(b.clone(), -c);

        S::from_real(delta).try_sqrt().map(move |root| {
            let neg_b = S::from_real(-b);

            [neg_b.clone() - root.clone(), neg_b + root]
        })
    }

    #[inline]
    #[must_use]
    pub fn tangents_at_intersections(
        &self,
        ray: &Ray<SVector<S, D>>,
    ) -> Option<[(S, Unit<SVector<S, D>>); 2]> {
        self.intersections(ray).map(|ds| {
            ds.map(|d| {
                (
                    d.clone(),
                    // SAFETY: p := ray.at(d) is in the sphere,
                    // so ||p - self.center|| = |self.radius|
                    Unit::new_unchecked(
                        (ray.at(d) - self.center.clone()).unscale(self.radius.clone().abs()),
                    ),
                )
            })
        })
    }
}

impl<S: RealField, const D: usize> Mirror<Unit<SVector<S, D>>> for Sphere<S, D> {
    fn add_tangents(
        &self,
        ctx: &SimulationCtx<SVector<S, D>>,
    ) -> Option<Intersection<Unit<SVector<S, D>>>> {
        ctx.closest(
            self.tangents_at_intersections(ctx.ray)
                .into_iter()
                .flatten(),
        )
    }
}
