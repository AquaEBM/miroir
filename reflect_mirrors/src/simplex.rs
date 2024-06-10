use super::*;

/// A (D-1)-simplex in D-dimensional (euclidean) space
/// (A line segment in 2D space, a triangle in 3D space, etc...)
#[derive(Clone, Debug, PartialEq)]
pub struct Simplex<const D: usize> {
    /// The plane this mirror belongs to, the unused first vector is used as the starting point
    plane: HyperPlaneBasis<Float, D>,
    /// The same plane, but represented with an orthonormal basis, useful for orthogonal symmetries
    orthonormalised: HyperPlaneBasisOrtho<Float, D>,
}

pub type Triangle = Simplex<3>;
pub type LineSegment = Simplex<2>;

impl<const D: usize> Simplex<D> {
    #[inline]
    pub fn try_new(points: [impl Into<SVector<Float, D>>; D]) -> Option<Self> {
        let mut vectors: [SVector<_, D>; D] = points.map(Into::into);
        let (&mut v0, basis) = vectors.split_first_mut().unwrap();
        basis.iter_mut().for_each(|v| *v -= v0);
        HyperPlaneBasis::new(vectors).map(|(plane, orthonormalised)| Self {
            plane,
            orthonormalised,
        })
    }

    #[inline]
    pub fn new(vectors: [impl Into<SVector<Float, D>>; D]) -> Self {
        Self::try_new(vectors).unwrap()
    }

    #[inline]
    pub const fn inner_plane(&self) -> &HyperPlaneBasis<Float, D> {
        &self.plane
    }
}

impl<const D: usize, U> TryFrom<[U; D]> for Simplex<D>
where
    SVector<Float, D>: From<U>,
{
    type Error = ();

    #[inline]
    fn try_from(vectors: [U; D]) -> Result<Self, Self::Error> {
        HyperPlaneBasis::new(vectors.map(SVector::from))
            .map(|(plane, orthonormalised)| Self {
                plane,
                orthonormalised,
            })
            .ok_or(())
    }
}

impl<const D: usize> Simplex<D> {
    #[inline]
    pub fn vertices(&self) -> [SVector<Float, D>; D] {
        let mut vertices = *self.inner_plane().vectors_raw();
        let (&mut v0, vectors) = vertices.split_first_mut().unwrap();
        vectors.iter_mut().for_each(|v| *v += v0);
        vertices
    }
}

impl<const D: usize> Mirror<D> for Simplex<D> {
    type Scalar = Float;
    fn add_tangents(&self, ctx: &mut SimulationCtx<Float, D>) {
        let p = self.inner_plane();

        let ray = ctx.ray();

        let intersection_coords = p.intersection_coordinates(ray, p.v0());

        if let Some(&t) = intersection_coords.as_ref().and_then(|v| {
            let (distance, plane_coords) = v.as_slice().split_first().unwrap();
            let mut sum = 0.;
            for &coord in plane_coords {
                if coord < 0. {
                    return None;
                }
                sum += coord;
            }

            if sum > 1. {
                return None;
            }

            Some(distance)
        }) {
            ctx.add_tangent(Plane {
                // We could return `self.plane.v0()`, but since we already calculated `t`,
                // we might as well save the simulation runner some work, and return that
                intersection: Intersection::Distance(t),
                direction: HyperPlane::Plane(self.orthonormalised.clone()),
            });
        }
    }
}

struct PlaneRenderData<const D: usize> {
    vertices: gl::VertexBuffer<Vertex<D>>,
}

impl<const D: usize> RenderData for PlaneRenderData<D> {
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

impl<const D: usize> OpenGLRenderable for Simplex<D>
where
    Vertex<D>: gl::Vertex,
{
    fn append_render_data(&self, display: &gl::Display, list: &mut List<Box<dyn RenderData>>) {
        let vertices = self.vertices().map(Vertex::from);

        list.push(Box::new(PlaneRenderData {
            vertices: gl::VertexBuffer::new(display, vertices.as_slice()).unwrap(),
        }))
    }
}
