use core::ops::AddAssign;

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
    pub fn intersection(&self, ray: &Ray<SVector<S, D>>) -> Option<S> {
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

impl<S: RealField, const D: usize> Mirror<HyperplaneBasisOrtho<S, D>> for Simplex<S, D> {
    fn closest_intersection(
        &self,
        ray: &Ray<SVector<S, D>>,
        ctx: SimulationCtx<S>,
    ) -> Option<Intersection<HyperplaneBasisOrtho<S, D>>> {
        ctx.closest(
            self.intersection(ray)
                .map(|dist| (dist, self.inner_plane_ortho().clone())),
        )
    }
}

#[cfg(feature = "glium")]
struct SimplexRenderData<const D: usize> {
    vertices: gl::VertexBuffer<Vertex<D>>,
}

#[cfg(feature = "glium")]
impl<const D: usize> RenderData for SimplexRenderData<D> {
    fn vertices(&self) -> gl::vertex::VerticesSource {
        (&self.vertices).into()
    }

    fn indices(&self) -> gl::index::IndicesSource {
        gl::index::IndicesSource::NoIndices {
            primitives: match D {
                0 => unreachable!("dimension must not be zero"),
                1 | 2 => gl::index::PrimitiveType::LinesList,
                _ => gl::index::PrimitiveType::TriangleStrip,
            },
        }
    }
}

#[cfg(feature = "glium")]
impl<S, const D: usize> OpenGLRenderable for Simplex<S, D>
where
    Vertex<D>: gl::Vertex + From<SVector<S, D>>,
    SVector<S, D>: AddAssign + Clone,
{
    fn append_render_data(&self, display: &gl::Display, list: &mut List<Box<dyn RenderData>>) {
        let vertices = self.vertices().map(Vertex::from);

        list.push(Box::new(SimplexRenderData {
            vertices: gl::VertexBuffer::new(display, vertices.as_slice()).unwrap(),
        }))
    }
}

#[cfg(feature = "numworks")]
impl<S: RealField + AsPrimitive<i16>> KandinskyRenderable for LineSegment<S> {
    fn draw(&self, color: Color) {
        let [start, end] = self.vertices();
        draw_line(start.to_point(), end.to_point(), color);
    }
}