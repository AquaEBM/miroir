use core::array;

use super::*;

/// A parallelotope-shaped reflective (hyper)plane
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlaneMirror<const D: usize> {
    /// The plane this mirror belongs to.
    plane: AffineHyperPlane<D>,
    /// The same plane, but represented with an orthonormal basis, useful for orthogonal symmetries
    orthonormalised: AffineHyperPlaneOrtho<D>,
}

impl<const D: usize> PlaneMirror<D> {
    pub fn try_new(vectors: [SVector<Float, D>; D]) -> Option<Self> {
        vectors.try_into().ok()
    }

    pub fn inner_plane(&self) -> &AffineHyperPlane<D> {
        &self.plane
    }
}

impl<const D: usize> TryFrom<[SVector<Float, D>; D]> for PlaneMirror<D> {
    type Error = ();

    fn try_from(vectors: [SVector<Float, D>; D]) -> Result<Self, Self::Error> {
        AffineHyperPlane::new(vectors)
            .map(|(plane, orthonormalised)| Self {
                plane,
                orthonormalised,
            })
            .ok_or(())
    }
}

impl<const D: usize> PlaneMirror<D> {
    pub fn vertices(&self) -> impl Iterator<Item = SVector<Float, D>> + '_ {
        let basis = self.inner_plane().basis();
        let v0 = *self.inner_plane().v0();

        (0..1 << (D - 1)).map(move |i| {
            let mut acc = [SVector::zeros(); 2];

            basis
                .iter()
                .enumerate()
                // returns `v` with the sign flipped if the `j`th bit in `i` is 1
                .for_each(|(j, v)| acc[i >> j & 1] += v);

            let [plus, minus] = acc;

            v0 + plus - minus
        })
    }
}

impl<const D: usize> Mirror<D> for PlaneMirror<D> {
    fn add_tangents(&self, ctx: &mut SimulationCtx<D>) {
        let p = self.inner_plane();

        let ray = ctx.ray();

        let intersection_coords = p.intersection_coordinates(ray, p.v0());

        if let Some(&t) = intersection_coords.as_ref().and_then(|v| {
            let (distance, plane_coords) = v.as_slice().split_first().unwrap();
            plane_coords
                .iter()
                .all(|mu| mu.abs() < 1.0)
                .then_some(distance)
        }) {
            ctx.add_tangent(TangentPlane {
                // We could return `self.plane.v0()`, but since we already calculated `t`,
                // we might as well save the simulation runner some work, and return that
                intersection: Intersection::Distance(t),
                direction: TangentSpace::Plane(self.orthonormalised),
            });
        }
    }
}

impl<const D: usize> JsonType for PlaneMirror<D> {
    fn json_type() -> String {
        "plane".into()
    }
}

impl<const D: usize> JsonDes for PlaneMirror<D> {
    /// Deserialize a new plane mirror from a JSON object.
    ///
    /// The JSON object must follow the same format as that
    /// described in the documentation of [AffineHyperPlane::from_json]
    fn from_json(json: &serde_json::Value) -> Result<Self, Box<dyn std::error::Error>> {
        let mut vectors = [SVector::zeros(); D];

        let (v_0, basis) = vectors.split_first_mut().unwrap();

        *v_0 = json
            .get("center")
            .and_then(serde_json::Value::as_array)
            .map(Vec::as_slice)
            .and_then(json_array_to_vector)
            .ok_or("Failed to parse center")?;

        let basis_json = json
            .get("basis")
            .and_then(serde_json::Value::as_array)
            .map(Vec::as_slice)
            .filter(|l| l.len() == D - 1)
            .ok_or("Failed to parse basis")?;

        for (value, vector) in basis_json.iter().zip(basis) {
            *vector = value
                .as_array()
                .map(Vec::as_slice)
                .and_then(json_array_to_vector)
                .ok_or("Failed to parse basis vector")?;
        }

        Self::try_new(vectors).ok_or("the provided family of vectors must be free".into())
    }
}

impl<const D: usize> JsonSer for PlaneMirror<D> {
    /// Serialize a plane mirror into a JSON object.
    ///
    /// The format of the returned object is explained in [`Self::from_json`]
    fn to_json(&self) -> serde_json::Value {
        let slices = self
            .inner_plane()
            .vectors_raw()
            .each_ref()
            .map(SVector::as_slice);

        let (center, basis) = slices.split_first().unwrap();

        serde_json::json!({
            "center": center,
            "basis": basis,
        })
    }
}

impl<const D: usize> Random for PlaneMirror<D> {
    fn random(rng: &mut (impl rand::Rng + ?Sized)) -> Self {
        loop {
            if let Some(mirror) = Self::try_new(array::from_fn(|_| rand_vect(rng, 10.0))) {
                break mirror;
            }
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

impl OpenGLRenderable for PlaneMirror<2> {
    fn append_render_data(&self, display: &gl::Display, mut list: List<Box<dyn RenderData>>) {
        let vertices: Vec<_> = self.vertices().map(Vertex2D::from).collect();

        list.push(Box::new(PlaneRenderData {
            vertices: gl::VertexBuffer::new(display, vertices.as_slice()).unwrap(),
        }))
    }
}

impl OpenGLRenderable for PlaneMirror<3> {
    fn append_render_data(&self, display: &gl::Display, mut list: List<Box<dyn RenderData>>) {
        let vertices: Vec<_> = self.vertices().map(Vertex3D::from).collect();

        list.push(Box::new(PlaneRenderData {
            vertices: gl::VertexBuffer::new(display, vertices.as_slice()).unwrap(),
        }))
    }
}