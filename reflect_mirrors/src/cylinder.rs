use super::*;

/// An open, cylinder-shaped mirror,
pub struct Cylinder {
    start: SVector<Float, 3>,
    dist: SVector<Float, 3>,
    inv_norm_dist_squared: Float,
    radius: Float,
    radius_sq: Float,
}

impl Cylinder {
    /// Create a new cylinder from a line segment and a radius
    #[inline]
    #[must_use]
    pub fn new(
        segment_start: impl Into<SVector<Float, 3>>,
        segment_end: impl Into<SVector<Float, 3>>,
        radius: Float,
    ) -> Self {
        let start = segment_start.into();
        let end = segment_end.into();
        let dist = end - start;
        let dist_sq = dist.norm_squared();

        Self {
            start,
            dist,
            radius,
            radius_sq: radius * radius,
            inv_norm_dist_squared: dist_sq.recip(),
        }
    }

    #[inline]
    pub const fn segment_dist(&self) -> SVector<Float, 3> {
        self.dist
    }

    #[inline]
    pub fn line_segment(&self) -> [SVector<Float, 3>; 2] {
        [self.start, self.start + self.dist]
    }

    #[inline]
    pub const fn radius(&self) -> &Float {
        &self.radius
    }

    #[inline]
    pub fn set_radius(&mut self, radius: Float) -> bool {
        let r_abs = radius.abs();
        let ok = r_abs > Float::EPSILON * 16.0;

        if ok {
            self.radius = r_abs;
            self.radius_sq = radius * radius;
        }

        ok
    }
}

impl Mirror<3> for Cylinder {
    type Scalar = Float;
    fn add_tangents(&self, ctx: &mut SimulationCtx<Float, 3>) {
        let line_coord = |v| self.dist.dot(&v) * self.inv_norm_dist_squared;
        let p = |v| line_coord(v) * self.dist;

        let ray = ctx.ray().clone();

        let m = ray.origin - self.start;
        let d = ray.dir.into_inner();
        let pm = p(m);
        let pd = p(d);

        let a = (d - pd).norm_squared();
        let b = d.dot(&pm).mul_add(-2.0, pm.dot(&pd) + m.dot(&d));
        let c = (m - pm).norm_squared() - self.radius_sq;

        let delta = c.mul_add(-a, b * b);

        if delta >= 0. {
            let root_delta = delta.sqrt();
            let neg_b = -b;

            for t in [(neg_b - root_delta) / a, (neg_b + root_delta) / a] {
                let origin = ray.at(t);
                let coord = line_coord(origin - self.start);

                let line_pt = self.start + self.dist * coord;

                // Thanks clippy!
                if (0.0..=1.0).contains(&coord) {
                    // SAFETY: the vector `origin - v0` always has length `r = self.radius`
                    let normal = Unit::new_unchecked((origin - line_pt) / self.radius.abs());

                    ctx.add_tangent(Plane {
                        intersection: Intersection::Distance(t),
                        direction: HyperPlane::Normal(normal),
                    })
                }
            }
        }
    }
}
