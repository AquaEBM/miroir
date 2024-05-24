use core::ops::Deref;

use super::*;
use util::List;

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
    pub fn reflect_dir(&mut self, tangent: &TangentSpace<D>) {
        self.direction = tangent.reflect_unit(self.direction);
    }

    /// Move the ray's position forward (or backward if t < 0.0) by `t`
    pub fn advance(&mut self, t: Float) {
        self.origin += t * self.direction.into_inner();
    }

    /// Get the point at distance `t` (can be negative) from the ray's origin
    pub fn at(&self, t: Float) -> SVector<Float, D> {
        self.origin + self.direction.into_inner() * t
    }
}

/// An affine hyperplane
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AffineHyperPlane<const D: usize> {
    /// See [`AffineHyperPlane::new`] for info on the layout of this field
    vectors: [SVector<Float, D>; D],
}

impl<const D: usize> AffineHyperPlane<D> {
    /// The first element of the `vectors` array is the plane's "starting point" (i. e. v_0).
    ///
    /// The remaining `N-1` vectors are a free family spanning it's direction hyperplane
    ///
    /// Returns `None` if the provided family isn't free.
    ///
    /// Note that an expression like `[T ; N - 1]` is locked under `#[feature(const_generic_exprs)]`.
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
    pub fn v0(&self) -> &SVector<Float, D> {
        self.vectors.first().unwrap()
    }

    /// A mutable reference to the plane's starting point
    pub fn v0_mut(&mut self) -> &mut SVector<Float, D> {
        &mut self.vectors[0]
    }

    /// A reference to the basis of the plane's direction hyperplane.
    ///
    /// The returned slice is garanteed to be of length `D - 1`.
    pub fn basis(&self) -> &[SVector<Float, D>] {
        &self.vectors[1..]
    }

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
pub struct AffineHyperPlaneOrtho<const D: usize> {
    /// See [`AffineHyperPlane::new`] for info on the layout of this field
    vectors: [SVector<Float, D>; D],
}

impl<const D: usize> AffineHyperPlaneOrtho<D> {
    /// A reference to the plane's starting point
    pub fn v0(&self) -> &SVector<Float, D> {
        self.vectors.first().unwrap()
    }

    /// A mutable reference to the plane's starting point
    pub fn v0_mut(&mut self) -> &mut SVector<Float, D> {
        &mut self.vectors[0]
    }

    /// A reference to an orthonormal basis of the plane's direction hyperplane.
    ///
    /// The returned slice is garanteed to be of length `D - 1`.
    pub fn basis(&self) -> &[SVector<Float, D>] {
        &self.vectors[1..]
    }

    /// Returns the orthogonal projection of `v` w.r.t. this plane's direction subspace.
    pub fn orthogonal_projection(&self, v: SVector<Float, D>) -> SVector<Float, D> {
        self.basis().iter().map(|e| v.dot(e) * e).sum()
    }

    /// Returns the point in this plane whose distance with `p` is smallest.
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
    /// The starting point will be ignored and can be arbitrary.
    Plane(AffineHyperPlaneOrtho<D>),
    Normal(Unit<SVector<Float, D>>),
}

impl<const D: usize> TangentSpace<D> {
    /// Reflect a vector w.r.t this hyperplane
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
    pub fn reflect_unit(&self, v: Unit<SVector<Float, D>>) -> Unit<SVector<Float, D>> {
        // SAFETY: orthogonal symmetry preserves euclidean norms
        // This function is supposed to be unsafe, why nalgebra? why?
        Unit::new_unchecked(self.reflect(v.into_inner()))
    }

    /// Return the distance `t` such that `ray.at(t)` intersects with the affine hyperplane
    /// whose direction space is `self`, and whose starting point is `p`.
    ///
    /// Returns `None` if `ray` is parallel to `self`
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
    /// If a mirror returns `Intersection::Distance(t)` when calculating it's intersections with a `ray`, then `ray.at(t)` belongs to the returned tangent (hyper)plane.
    ///
    /// This is useful if `t` is easy to calculate.
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
    pub fn reflect(&self, v: SVector<Float, D>) -> SVector<Float, D> {
        self.direction.reflect(v)
    }

    /// Reflect a unit vector w.r.t. this tangent plane's direction hyperplane
    pub fn reflect_unit(&self, v: Unit<SVector<Float, D>>) -> Unit<SVector<Float, D>> {
        self.direction.reflect_unit(v)
    }

    /// Return the distance `t` such that `ray.at(t)` intersects with this tangent plane
    ///
    /// Returns `None` if `ray` is parallel to `self`
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
/// Some mirrors, (see [`Plane`][plane::PlaneMirror] and [`Sphere`][sphere::EuclideanSphereMirror])
/// exist and are easy to implement in all dimensions. Hence why they implement [`Mirror<D>`]
/// for all (non-zero) `D`. Others, (like [`Cylinder`][cylinder::CylindricalMirror]) are only
/// implemented in _some_ dimensions.
///
/// However, note that, currently, only simulations in dimensions `2` and `3` can be rendered
/// (with OpenGL) and generated randomly. Other functionality can be easily implemented,
/// (refer to other APIs in this crate).
// `D` could have been an associated constant but, lack of
// `#[feature(generic_const_exprs)]` screws us over, once again.
pub trait Mirror<const D: usize> {
    /// Appends to `list` a number of affine (hyper)planes, tangent to this mirror, in no particular order.
    ///
    /// `ray` is expected to "bounce" off the plane closest to it in `list`.
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
    fn append_intersecting_points(&self, ray: &Ray<D>, list: List<TangentPlane<D>>);
}

impl<const D: usize, T: Mirror<D>> Mirror<D> for [T] {
    fn append_intersecting_points(&self, ray: &Ray<D>, mut list: List<TangentPlane<D>>) {
        self.iter()
            .for_each(|mirror| mirror.append_intersecting_points(ray, list.reborrow()))
    }
}

impl<const D: usize, T: Deref> Mirror<D> for T
where
    T::Target: Mirror<D>,
{
    fn append_intersecting_points(&self, ray: &Ray<D>, list: List<TangentPlane<D>>) {
        self.deref().append_intersecting_points(ray, list)
    }
}

pub trait JsonType {
    /// Returns a string, unique to the type, found in the "type" field of the json
    /// representation of a "dynamic" mirror containing a mirror of this type
    fn json_type() -> String;
}

impl<T: JsonType> JsonType for [T] {
    fn json_type() -> String {
        format!("[]{}", T::json_type())
    }
}

impl<T: Deref> JsonType for T
where
    T::Target: JsonType,
{
    fn json_type() -> String {
        T::Target::json_type()
    }
}

pub trait JsonSer {
    /// Serialize `self` into a JSON object.
    fn to_json(&self) -> serde_json::Value;
}

impl<const D: usize> JsonSer for Ray<D> {
    /// Serialize a ray into a JSON object.
    ///
    /// The format of the returned object is explained in [`Self::from_json`]
    fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "origin": self.origin.as_slice(),
            "direction": self.direction.as_ref().as_slice(),
        })
    }
}

impl<T: JsonSer> JsonSer for [T] {
    fn to_json(&self) -> serde_json::Value {
        serde_json::Value::Array(Vec::from_iter(self.iter().map(T::to_json)))
    }
}

impl<T: Deref> JsonSer for T
where
    T::Target: JsonSer,
{
    fn to_json(&self) -> serde_json::Value {
        self.deref().to_json()
    }
}

pub trait JsonDes {
    /// Deserialize from a JSON object.
    ///
    /// Returns an error if `json`'s format or values are invalid.
    fn from_json(json: &serde_json::Value) -> Result<Self, Box<dyn Error>>
    where
        Self: Sized;
}

impl<const D: usize> JsonDes for Ray<D> {
    /// Deserialize a new ray from a JSON object.
    ///
    /// The JSON object must follow the following format:
    ///
    /// ```json
    /// {
    ///     "origin": [9., 8., 7., ...], // (an array of D floats)
    ///     "direction": [9., 8., 7., ...], // (an array of D floats, must have at least one non-zero value)
    /// }
    /// ```
    fn from_json(json: &serde_json::Value) -> Result<Self, Box<dyn Error>> {
        let origin = json
            .get("origin")
            .and_then(serde_json::Value::as_array)
            .ok_or("Missing ray origin")?;

        let direction = json
            .get("direction")
            .and_then(serde_json::Value::as_array)
            .ok_or("Missing ray direction")?;

        let origin = util::json_array_to_vector(origin).ok_or("Invalid ray origin")?;

        let direction = util::json_array_to_vector(direction).ok_or("Invalid ray direction")?;

        let direction =
            Unit::try_new(direction, Float::EPSILON).ok_or("Unable to normalize ray direction")?;

        Ok(Self { origin, direction })
    }
}

impl<T: JsonDes> JsonDes for Vec<T> {
    fn from_json(json: &serde_json::Value) -> Result<Self, Box<dyn Error>> {
        util::map_json_array(json, T::from_json)
    }
}

pub trait Random {
    /// Generate a randomized version of this mirror using the provided `rng`
    ///
    /// This method must not fail. If creating a mirror is faillible, keep trying until success
    fn random(rng: &mut (impl rand::Rng + ?Sized)) -> Self
    where
        Self: Sized;
}

impl<const D: usize> Random for Ray<D> {
    fn random(rng: &mut (impl rand::Rng + ?Sized)) -> Self {
        let origin = util::rand_vect(rng, 7.0);

        let direction = loop {
            if let Some(v) = Unit::try_new(util::rand_vect(rng, 1.0), Float::EPSILON * 8.0) {
                break v;
            }
        };
        Self { origin, direction }
    }
}
