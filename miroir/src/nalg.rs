use super::*;

pub use nalgebra;
use nalgebra::{zero, ComplexField, SMatrix, SVector, SimdComplexField, Unit};

impl<S: SimdComplexField, const D: usize> Vector for SVector<S, D> {
    type Scalar = S;
}

impl<S: SimdComplexField, const D: usize> VMulAdd for SVector<S, D> {
    #[inline]
    fn mul_add(&self, t: Self::Scalar, other: &Self) -> Self
    where
        Self: Sized,
    {
        self * t + other
    }
}

impl<S: SimdComplexField, const D: usize> Hyperplane for Unit<SVector<S, D>> {
    type Vector = SVector<S, D>;

    #[inline]
    fn reflect(&self, v: &mut Self::Vector) {
        let n = self.as_ref();

        let p = v.dot(n);

        *v -= n * (p.clone() + p);
        v.normalize_mut();
    }
}

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
        ray: &Ray<SVector<S, D>>,
        v0: &SVector<S, D>,
    ) -> Option<SVector<S, D>> {
        let mut a = SMatrix::<S, D, D>::from_columns(&self.vectors);
        a.set_column(0, &ray.dir);

        a.try_inverse_mut()
            // a now contains a^-1
            .then(|| {
                let mut v = a * (&ray.pos - v0);
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
    #[must_use]
    pub const fn v0_mut(&mut self) -> &mut SVector<S, D> {
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
        &self.vectors[1..]
    }

    #[inline]
    #[must_use]
    pub const fn vectors_raw(&self) -> &[SVector<S, D>; D] {
        &self.vectors
    }
}

/// A hyperplane, like [`HyperplaneBasis`], but the basis stored is garanteed
/// to be (approximately) orthonormal, efficiently enabling projections and symmetries.
///
/// `D` must be non-zero, or unexpected panics could happen
#[derive(Clone, Debug, PartialEq)]
pub struct HyperplaneBasisOrtho<S, const D: usize> {
    /// See [`HyperPlaneBasis::new`] for info on the layout of this field
    plane: HyperplaneBasis<S, D>,
}

impl<S, const D: usize> Deref for HyperplaneBasisOrtho<S, D> {
    type Target = HyperplaneBasis<S, D>;

    #[inline]
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
    #[inline]
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
    /// with the smallest distance to `p`.
    #[inline]
    #[must_use]
    pub fn closest_point_to_plane(&self, v0: &SVector<S, D>, p: &SVector<S, D>) -> SVector<S, D> {
        let v = p - v0;
        v0 + self.project(&v)
    }
}

impl<S: SimdComplexField, const D: usize> Hyperplane for HyperplaneBasisOrtho<S, D> {
    type Vector = SVector<S, D>;

    #[inline]
    /// Reflect a vector w.r.t this hyperplane
    fn reflect(&self, v: &mut SVector<S, D>) {
        let p2: Self::Vector = self
            .basis()
            .iter()
            .map(|e| {
                let c = v.dot(e);
                e * (c.clone() + c)
            })
            .sum();
        *v = p2 - &*v;
        v.normalize_mut();
    }
}

impl<V> Ray<V> {
    #[inline]
    #[must_use]
    pub fn new_unchecked_dir(pos: impl Into<V>, dir: impl Into<V>) -> Self {
        Self {
            pos: pos.into(),
            dir: dir.into(),
        }
    }
}

impl<S, const D: usize> Ray<SVector<S, D>> {
    #[inline]
    #[must_use]
    pub fn new_unit_dir(pos: impl Into<SVector<S, D>>, dir: Unit<SVector<S, D>>) -> Self {
        Self::new_unchecked_dir(pos, dir.into_inner())
    }
}

impl<S: ComplexField, const D: usize> Ray<SVector<S, D>> {
    #[inline]
    #[must_use]
    pub fn try_new_normalize(
        pos: impl Into<SVector<S, D>>,
        dir: impl Into<SVector<S, D>>,
    ) -> Option<Self> {
        Unit::try_new(dir.into(), zero()).map(|unit| Self::new_unit_dir(pos, unit))
    }

    /// # Panics
    ///
    /// if `dir` is zero
    #[inline]
    #[must_use]
    pub fn new_normalize(pos: impl Into<SVector<S, D>>, dir: impl Into<SVector<S, D>>) -> Self {
        Self::try_new_normalize(pos, dir).expect("direction must not be zero")
    }
}

/// Checks if adding `new_pt` to `path` results in a ray doing an infinite loop.
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
            let [this_pt, next_pt] = window else { panic!() };

            let impact_dir = Unit::new_normalize(next_pt - this_pt).into_inner();

            ((new_pt - next_pt).norm() <= *eps && (impact_dir - &current_dir).norm() <= *eps)
                .then_some(i)
        })
    })
}
