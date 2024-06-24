use arrayvec::ArrayVec;
use nalgebra::RealField;

use super::*;

/// An open, cylinder-shaped mirror,
#[derive(Clone, Debug, PartialEq)]
pub struct Cylinder<S> {
    start: SVector<S, 3>,
    dist: SVector<S, 3>,
    inv_norm_dist_squared: S,
    radius: S,
    radius_sq: S,
}

impl<S: RealField> Cylinder<S> {
    /// Create a new cylinder from a line segment and a radius
    #[inline]
    #[must_use]
    pub fn new(
        segment_start: impl Into<SVector<S, 3>>,
        segment_end: impl Into<SVector<S, 3>>,
        radius: S,
    ) -> Self {
        let start = segment_start.into();
        let end = segment_end.into();
        let dist = end - &start;
        let dist_sq = dist.norm_squared();

        Self {
            start,
            dist,
            radius: radius.clone(),
            radius_sq: radius.clone() * radius,
            inv_norm_dist_squared: dist_sq.recip(),
        }
    }

    #[inline]
    #[must_use]
    pub const fn start(&self) -> &SVector<S, 3> {
        &self.start
    }

    #[inline]
    #[must_use]
    pub const fn segment_dist(&self) -> &SVector<S, 3> {
        &self.dist
    }

    #[inline]
    #[must_use]
    pub fn line_segment(&self) -> [SVector<S, 3>; 2] {
        [self.start.clone(), self.start.clone() + self.dist.clone()]
    }

    #[inline]
    #[must_use]
    pub const fn radius(&self) -> &S {
        &self.radius
    }

    #[inline]
    pub fn set_radius(&mut self, radius: S) {
        self.radius = radius.clone().abs();
        self.radius_sq = radius.clone() * radius;
    }

    #[inline]
    #[must_use]
    pub fn tangents_at_intersections(
        &self,
        ray: &Ray<S, 3>,
    ) -> ArrayVec<(S, Unit<SVector<S, 3>>), 2> {
        let line_coord = |v| self.dist.dot(&v) * self.inv_norm_dist_squared.clone();
        let p = |v| &self.dist * line_coord(v);

        let m = &ray.origin - &self.start;
        let d = ray.dir.as_ref();
        let pm = p(m.clone());
        let pd = p(d.clone());

        let a = (d - &pd).norm_squared();
        let dpm = d.dot(&pm);
        let b = (pm.dot(&pd) + m.dot(d)) - (dpm.clone() + dpm);
        let c = (&m - pm).norm_squared() - self.radius_sq.clone();

        let delta = c.mul_add(-a.clone(), b.clone() * b.clone());

        let mut out = ArrayVec::<_, 2>::new();

        if let Some(root) = delta.try_sqrt() {
            let neg_b = -b;
            let t1 = (neg_b.clone() - root.clone()) / a.clone();
            let t2 = (neg_b + root) / a;
            for t in [t1, t2] {
                let origin = ray.at(t.clone());
                let v = &origin - &self.start;
                let coord = line_coord(v);

                if (S::zero()..=S::one()).contains(&coord) {
                    let line_pt = &self.start + self.dist.clone() * coord;

                    out.push((
                        t,
                        Unit::new_unchecked((origin - line_pt).unscale(self.radius.clone())),
                    ));
                }
            }
        }

        out
    }
}

impl<S: RealField> Mirror<3> for Cylinder<S> {
    type Scalar = S;
    fn add_tangents(&self, ctx: &mut SimulationCtx<Self::Scalar, 3>) {
        for (d, n) in self.tangents_at_intersections(ctx.ray()) {
            ctx.add_tangent(d, Hyperplane::Normal(n));
        }
    }
}
