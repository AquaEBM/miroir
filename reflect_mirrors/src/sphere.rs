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
    fn add_tangents(&self, ctx: &mut SimulationCtx<D>) {
        // substituting V for P + t * D in the sphere equation: ||V - C||^2 - r^2 = 0
        // results in a quadratic equation in t, solve it using the discriminant method and
        // return the vector pointing from the center of the sphere to the point of intersection
        // as it is orthogonal to the direction space of the tangent to the sphere at that point
        // the process is almost the same for every quadric shape (see cylinder)

        let ray = *ctx.ray();

        let d = &ray.direction;

        let v0 = &self.center;
        let v = ray.origin - v0;

        let r = self.radius();
        let s = v.norm_squared();

        let a = d.norm_squared();
        let b = v.dot(d);
        let c = s - r * r;

        let delta = b * b - a * c;

        if delta > Float::EPSILON {
            let root_delta = delta.sqrt();
            let neg_b = -b;

            for t in [(neg_b - root_delta) / a, (neg_b + root_delta) / a] {
                let origin = ray.at(t);
                // SAFETY: the vector `origin - v0` always has length `r = self.radius`
                let normal = Unit::new_unchecked((origin - v0) / r.abs());
                ctx.add_tangent(Plane {
                    intersection: Intersection::Distance(t),
                    direction: HyperPlane::Normal(normal),
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
