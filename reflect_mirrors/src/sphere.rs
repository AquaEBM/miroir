use super::*;

#[derive(Clone, Copy)]
/// All vectors at a certain distance (radius) from a certain vector (center)
/// where the distance here is the standard euclidean distance
// TODO: We can do other distances, can we huh?
pub struct EuclideanSphereMirror<const D: usize> {
    pub center: SVector<Float, D>,
    radius: Float,
}

impl<const D: usize> EuclideanSphereMirror<D> {
    pub fn new(center: SVector<Float, D>, radius: Float) -> Option<Self> {
        (radius.abs() >= Float::EPSILON).then_some(Self { center, radius })
    }

    pub fn radius(&self) -> &Float {
        &self.radius
    }

    pub fn set_radius(&mut self, r: Float) -> bool {
        let ok = r.abs() >= Float::EPSILON;

        if ok {
            self.radius = r;
        }

        ok
    }
}

impl<const D: usize> Mirror<D> for EuclideanSphereMirror<D> {
    fn append_intersecting_points(&self, ray: &Ray<D>, mut list: List<TangentPlane<D>>) {
        // substituting V for P + t * D in the sphere equation: ||V - C||^2 - r^2 = 0
        // results in a quadratic equation in t, solve it using the discriminant method and
        // return the vector pointing from the center of the sphere to the point of intersection
        // as it is orthogonal to the direction space of the tangent to the sphere at that point
        // the process is almost the same for every quadric shape (see cylinder)

        let d = &ray.direction;
        let a = d.norm_squared();

        let v0 = &self.center;
        let v = ray.origin - v0;

        let b = v.dot(d);

        let r = self.radius();
        let s = v.norm_squared();
        let c = s - r * r;

        let delta = b * b - a * c;

        if delta > Float::EPSILON {
            let root_delta = delta.sqrt();
            let neg_b = -b;

            for t in [(neg_b - root_delta) / a, (neg_b + root_delta) / a] {
                let origin = ray.at(t);
                // SAFETY: the vector `origin - v0` always has length `r = self.radius`
                let normal = Unit::new_unchecked((origin - v0) / r.abs());
                list.push(TangentPlane {
                    intersection: Intersection::Distance(t),
                    direction: TangentSpace::Normal(normal),
                });
            }
        }
    }
}

impl<const D: usize> JsonType for EuclideanSphereMirror<D> {
    fn json_type() -> String {
        "sphere".into()
    }
}

impl<const D: usize> JsonDes for EuclideanSphereMirror<D> {
    /// Deserialize a new eudclidean sphere mirror from a JSON object.
    ///
    /// The JSON object must follow the following format:
    ///
    /// ```json
    /// {
    ///     "center": [1., 2., 3., ...], // (an array of D floats)
    ///     "radius": 4., // (must be a float of magnitude > Float::EPSILON ~= 10^-16 )
    /// }
    /// ```
    fn from_json(json: &serde_json::Value) -> Result<Self, Box<dyn std::error::Error>> {
        let center = json
            .get("center")
            .and_then(serde_json::Value::as_array)
            .map(Vec::as_slice)
            .and_then(json_array_to_vector)
            .ok_or("Failed to parse center")?;

        let radius = json
            .get("radius")
            .and_then(serde_json::Value::as_f64)
            .ok_or("Failed to parse radius")? as Float;

        Self::new(center, radius).ok_or("radius must not be too close to 0.0".into())
    }
}

impl<const D: usize> JsonSer for EuclideanSphereMirror<D> {
    /// Serialize a euclidean sphere mirror into a JSON object.
    ///
    /// The format of the returned object is explained in [`Self::from_json`]
    fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "center": self.center.as_slice(),
            "radius": self.radius(),
        })
    }
}

impl<const D: usize> Random for EuclideanSphereMirror<D> {
    fn random(rng: &mut (impl rand::Rng + ?Sized)) -> Self {
        const MAX_RADIUS: Float = 3.0;

        loop {
            if let Some(mirror) =
                Self::new(rand_vect(rng, 9.0), rng.gen::<Float>() * MAX_RADIUS.abs())
            {
                break mirror;
            }
        }
    }
}

// Use glium_shapes::sphere::Sphere for the 3D implementation
impl OpenGLRenderable for EuclideanSphereMirror<3> {
    fn append_render_data(&self, display: &gl::Display, mut list: List<Box<dyn RenderData>>) {
        let r = *self.radius() as f32;
        let [x, y, z] = self.center.map(|s| s as f32).into();

        // The default sphere from the SphereBuilder is a unit-sphere (radius of 1) with its center of mass located at the origin.
        // So we just have to scale it with the sphere radius on each axis and translate it.
        let sphere = gl_shapes::sphere::SphereBuilder::new()
            .scale(r, r, r)
            .translate(x, y, z)
            .with_divisions(60, 60)
            .build(display)
            .unwrap();

        list.push(Box::new(sphere))
    }
}

// in 2d, the list of vertices of a circle is easy to calculate
impl OpenGLRenderable for EuclideanSphereMirror<2> {
    fn append_render_data(&self, display: &gl::Display, mut list: List<Box<dyn RenderData>>) {
        list.push(Box::new(Circle::new(
            self.center.map(|s| s as f32).into(),
            *self.radius() as f32,
            display,
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_basic_sphere() {
        let mirror = EuclideanSphereMirror::<3>::from_json(&json!({
            "center": [0., 0., 0.],
            "radius": 1.,
        }))
        .expect("json error");

        let mut ray = Ray {
            origin: [-2., 0., 0.].into(),
            direction: Unit::new_normalize([1., 0., 0.].into()),
        };

        let mut intersections = vec![];
        mirror.append_intersecting_points(&ray, List::from(&mut intersections));

        assert_eq!(intersections.len(), 2);

        let tangent = &intersections[0];
        let d = tangent.try_ray_intersection(&ray);

        if let Some(t) = d {
            assert!((t - 1.).abs() < Float::EPSILON * 4.0);
            ray.advance(t);
        } else {
            panic!("there must be distance");
        }

        ray.reflect_dir(&tangent.direction);

        assert!((ray.origin - SVector::from([-1., 0., 0.])).norm().abs() < Float::EPSILON * 4.0);
        assert!(
            (ray.direction.as_ref() - SVector::from([-1., 0., 0.]))
                .norm()
                .abs()
                < Float::EPSILON * 4.0
        );
    }

    #[test]
    fn test_no_intersection() {
        let mirror = EuclideanSphereMirror::<3>::from_json(&json!({
            "center": [0., 0., 0.],
            "radius": 1.,
        }))
        .expect("json error");

        let ray = Ray {
            origin: [-2., 0., 0.].into(),
            direction: Unit::new_normalize([0., 1., 0.].into()),
        };

        let mut intersections = vec![];
        mirror.append_intersecting_points(&ray, List::from(&mut intersections));

        assert_eq!(intersections.len(), 0);
    }

    #[test]
    fn test_angled_ray() {
        let mirror = EuclideanSphereMirror::<3>::from_json(&json!({
            "center": [0., 0., 0.],
            "radius": 1.,
        }))
        .expect("json error");

        let mut ray = Ray {
            origin: [-2., -1., 0.].into(),
            direction: Unit::new_normalize([1., 1., 0.].into()),
        };

        let mut intersections = vec![];
        mirror.append_intersecting_points(&ray, List::from(&mut intersections));

        assert_eq!(intersections.len(), 2);

        let tangent = &intersections[0];
        let d = tangent.try_ray_intersection(&ray);

        if let Some(t) = d {
            assert!((t - 1.4142135623730951).abs() < Float::EPSILON * 4.0);
            ray.advance(t);
        } else {
            panic!("there must be distance");
        }

        ray.reflect_dir(&tangent.direction);

        assert!((ray.origin - SVector::from([-1., 0., 0.])).norm().abs() < Float::EPSILON * 4.0);
        assert!(
            (ray.direction.as_ref() - SVector::from([-0.7071067811865476, 0.7071067811865476, 0.]))
                .norm()
                .abs()
                < Float::EPSILON * 4.0
        );
    }

    #[test]
    fn test_json() {
        let mirror = EuclideanSphereMirror::<3>::from_json(&json!({
            "center": [0., 0., 0.],
            "radius": 1.,
        }))
        .expect("json error");

        let json = mirror.to_json();

        let mirror2 = EuclideanSphereMirror::<3>::from_json(&json).expect("json error");

        assert_eq!(mirror.center, mirror2.center);
        assert_eq!(mirror.radius(), mirror2.radius());
    }
}
