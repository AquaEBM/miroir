use core::{array, mem, ops::Add};

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
        const SHIFT: usize = mem::size_of::<Float>() * 8 - 1;

        let basis = self.inner_plane().basis();
        let v_0 = *self.inner_plane().v0();

        (0..1 << (D - 1)).map(move |i| {
            basis
                .iter()
                .enumerate()
                // returns `v` with the sign flipped if the `j`th bit in `i` is 1
                .map(|(j, v)| Float::from_bits(i >> j << SHIFT ^ Float::to_bits(1.0)) * v)
                .fold(v_0, Add::add)
        })
    }
}

impl<const D: usize> Mirror<D> for PlaneMirror<D> {
    fn append_intersecting_points(&self, ray: &Ray<D>, mut list: util::List<TangentPlane<D>>) {
        let p = self.inner_plane();

        let intersection_coords = p.intersection_coordinates(ray, p.v0());

        if let Some(&t) = intersection_coords.as_ref().and_then(|v| {
            let (distance, plane_coords) = v.as_slice().split_first().unwrap();
            plane_coords
                .iter()
                .all(|mu| mu.abs() < 1.0)
                .then_some(distance)
        }) {
            list.push(TangentPlane {
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
            .and_then(util::json_array_to_vector)
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
                .and_then(util::json_array_to_vector)
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
            if let Some(mirror) = Self::try_new(array::from_fn(|_| util::rand_vect(rng, 10.0))) {
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
    fn append_render_data(
        &self,
        display: &gl::Display,
        mut list: util::List<Box<dyn RenderData>>,
    ) {
        let vertices: Vec<_> = self.vertices().map(Vertex2D::from).collect();

        list.push(Box::new(PlaneRenderData {
            vertices: gl::VertexBuffer::new(display, vertices.as_slice()).unwrap(),
        }))
    }
}

impl OpenGLRenderable for PlaneMirror<3> {
    fn append_render_data(
        &self,
        display: &gl::Display,
        mut list: util::List<Box<dyn RenderData>>,
    ) {
        let vertices: Vec<_> = self.vertices().map(Vertex3D::from).collect();

        list.push(Box::new(PlaneRenderData {
            vertices: gl::VertexBuffer::new(display, vertices.as_slice()).unwrap(),
        }))
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use serde_json::json;

    #[test]
    fn test_left_basic_2d() {
        let mirror = PlaneMirror::<2>::from_json(&json!({
            "center": [0., 0.],
            "basis": [
                [0., 1.],
            ],
            "bounds": [1.],
        }))
        .expect("json monke");

        let mut ray = Ray {
            origin: [-1., 0.].into(),
            direction: Unit::new_normalize([1., 0.].into()),
        };

        let mut intersections = vec![];
        mirror.append_intersecting_points(&ray, util::List::from(&mut intersections));

        let [tangent] = intersections.as_slice() else {
            panic!("there must be one intersection");
        };

        let d = tangent.try_ray_intersection(&ray);

        if let Some(t) = d {
            assert!((t - 1.).abs() < Float::EPSILON * 4.0);
            ray.advance(t);
        } else {
            panic!("there must be distance");
        }

        ray.reflect_dir(&tangent.direction);

        assert!((ray.origin - SVector::from([0., 0.])).norm().abs() < Float::EPSILON * 4.0);
        assert!(
            (ray.direction.into_inner() - SVector::from([-1., 0.]))
                .norm()
                .abs()
                < Float::EPSILON * 4.0
        );
    }

    #[test]
    fn test_right_basic_2d() {
        let mirror = PlaneMirror::<2>::from_json(&json!({
            "center": [0., 0.],
            "basis": [
                [0., 1.],
            ],
            "bounds": [1.],
        }))
        .expect("json monke");

        let mut ray = Ray {
            origin: [1., 0.].into(),
            direction: Unit::new_normalize([-1., 0.].into()),
        };

        let mut intersections = vec![];

        mirror.append_intersecting_points(&ray, util::List::from(util::List::from(&mut intersections)));

        let [tangent] = intersections.as_slice() else {
            panic!("there must be an intersection");
        };

        let d = tangent.try_ray_intersection(&ray);

        if let Some(t) = d {
            assert!((t - 1.).abs() < Float::EPSILON * 4.0);
            ray.advance(t);
        } else {
            panic!("there must be distance");
        }

        ray.reflect_dir(&tangent.direction);

        assert!((ray.origin - SVector::from([0., 0.])).norm().abs() < Float::EPSILON * 4.0);
        assert!(
            (ray.direction.into_inner() - SVector::from([1., 0.]))
                .norm()
                .abs()
                < Float::EPSILON * 4.0
        );
    }

    #[test]
    fn test_diagonal_2d() {
        let mirror = PlaneMirror::<2>::from_json(&json!({
            "center": [0., 0.],
            "basis": [
                [0.70710677, 0.70710677],
            ],
            "bounds": [1.],
        }))
        .expect("json monke");

        let mut ray = Ray {
            origin: [-1., 1.].into(),
            direction: Unit::new_normalize([1., -1.].into()),
        };

        let mut intersections = vec![];
        mirror.append_intersecting_points(&ray, util::List::from(&mut intersections));

        let [tangent] = intersections.as_slice() else {
            panic!("there must be an intersection");
        };

        let d = tangent.try_ray_intersection(&ray);

        if let Some(t) = d {
            assert!((t - 1.4142135623730951).abs() < Float::EPSILON * 4.0);
            ray.advance(t);
        } else {
            panic!("there must be distance");
        }

        ray.reflect_dir(&tangent.direction);

        assert!((ray.origin - SVector::from([0., 0.])).norm().abs() < Float::EPSILON * 4.0);
        assert!(
            (ray.direction.into_inner() - SVector::from([-0.7071067811865476, 0.7071067811865476]))
                .norm()
                .abs()
                < Float::EPSILON * 4.0
        );
    }

    #[test]
    fn test_multiple_intersections_2d() {
        let m1 = PlaneMirror::<2>::from_json(&json!({
            "center": [10., 0.],
            "basis": [
                [0., 1.],
            ],
            "bounds": [1.],
        }))
        .expect("json monke");

        let m2 = PlaneMirror::<2>::from_json(&json!({
            "center": [-1., 0.],
            "basis": [
                [0., 1.],
            ],
            "bounds": [1.],
        }))
        .expect("json monke");

        let mut ray = Ray {
            origin: [0., 0.5].into(),
            direction: Unit::new_normalize([1., 0.].into()),
        };

        let mut pts = vec![];
        m1.append_intersecting_points(&ray, util::List::from(&mut pts));
        m2.append_intersecting_points(&ray, util::List::from(&mut pts));

        let [t1, t2] = pts.as_slice() else {
            panic!("there must be an intersection");
        };

        let d1 = t1.try_ray_intersection(&ray);
        let d2 = t2.try_ray_intersection(&ray);

        if let Some(t) = d1 {
            assert!((t - 10.).abs() < Float::EPSILON * 4.0);
            ray.advance(t);
        } else {
            panic!("there must be distance");
        }

        if let Some(t) = d2 {
            assert!((t - -1.).abs() < Float::EPSILON * 4.0);
        } else {
            panic!("there must be distance");
        }

        ray.reflect_dir(&t1.direction);

        assert!((ray.origin - SVector::from([10., 0.5])).norm().abs() < Float::EPSILON * 4.0);
        assert!(
            (ray.direction.into_inner() - SVector::from([-1., 0.]))
                .norm()
                .abs()
                < Float::EPSILON * 4.0
        );
    }
}
