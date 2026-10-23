use super::*;

struct CylinderRenderData {
    vertices: gl::VertexBuffer<Vertex3D>,
}

impl RenderData for CylinderRenderData {
    fn vertices(&self) -> gl::vertex::VerticesSource<'_> {
        (&self.vertices).into()
    }

    fn indices(&self) -> gl::index::IndicesSource<'_> {
        gl::index::IndicesSource::NoIndices {
            primitives: gl::index::PrimitiveType::TriangleStrip,
        }
    }
}

impl<S: na::RealField + AsPrimitive<f32>> OpenGLRenderable for miroir_shapes::Cylinder<S> {
    fn append_render_data(&self, display: &gl::Display, list: &mut List<Box<dyn RenderData>>) {
        let d = self.segment_dist().map(|v| v.as_());

        let rot = na::Rotation::<_, 3>::rotation_between(
            &na::SVector::from([0., 0., 1.]),
            &d.normalize(),
        )
        .unwrap();

        let r = self.radius().as_();
        let start = self.start().map(AsPrimitive::as_);

        const NUM_POINTS: usize = 360;
        const NUM_VERTICES: usize = (NUM_POINTS + 1) * 2;

        let mut vertices: [_; NUM_VERTICES] = [Default::default(); NUM_VERTICES];

        vertices
            .as_chunks_mut()
            .0
            .iter_mut()
            .enumerate()
            .for_each(|(i, [a, b])| {
                use core::f32::consts::TAU;

                let (x, y) = (i as f32 / NUM_POINTS as f32 * TAU).sin_cos();
                let vertex = [x * r, y * r, 0.];
                let k = rot * na::SVector::from(vertex) + start;
                (*a, *b) = (k.into(), (k + d).into())
            });

        let vertices = gl::VertexBuffer::immutable(display, vertices.as_slice()).unwrap();

        list.push(Box::new(CylinderRenderData { vertices }))
    }
}

struct SimplexRenderData<const D: usize> {
    vertices: gl::VertexBuffer<Vertex<D>>,
}

impl<const D: usize> RenderData for SimplexRenderData<D> {
    fn vertices(&self) -> gl::vertex::VerticesSource<'_> {
        (&self.vertices).into()
    }

    fn indices(&self) -> gl::index::IndicesSource<'_> {
        gl::index::IndicesSource::NoIndices {
            primitives: match D {
                0 => unreachable!("dimension must not be zero"),
                1 | 2 => gl::index::PrimitiveType::LinesList,
                _ => gl::index::PrimitiveType::TriangleStrip,
            },
        }
    }
}

impl<S, const D: usize> OpenGLRenderable for miroir_shapes::Simplex<S, D>
where
    Vertex<D>: gl::Vertex + From<SVector<S, D>>,
    SVector<S, D>: core::ops::AddAssign + Clone,
{
    fn append_render_data(&self, display: &gl::Display, list: &mut List<Box<dyn RenderData>>) {
        let vertices = self.vertices().map(Vertex::from);

        list.push(Box::new(SimplexRenderData {
            vertices: gl::VertexBuffer::new(display, vertices.as_slice()).unwrap(),
        }))
    }
}

// Use glium_shapes::sphere::Sphere for the 3D implementation
impl<S: na::RealField + AsPrimitive<f32>> OpenGLRenderable for miroir_shapes::Sphere<S, 3> {
    fn append_render_data(&self, display: &gl::Display, list: &mut List<Box<dyn RenderData>>) {
        let r = self.radius().as_();
        let [x, y, z] = self.center.map(|s| s.as_()).into();

        let sphere = gl_shapes::sphere::SphereBuilder::new()
            .scale(r, r, r)
            .translate(x, y, z)
            .with_divisions(60, 60)
            .build(display)
            .unwrap();

        list.push(Box::new(sphere))
    }
}

struct Circle {
    vertices: gl::VertexBuffer<Vertex2D>,
}

impl Circle {
    fn new<const N: usize>(center: [f32; 2], radius: f32, display: &gl::Display) -> Self {
        let c = SVector::from(center);

        use core::f32::consts::TAU;

        let points: [_; N] = core::array::from_fn(|i| {
            let w = i as f32 / N as f32 * TAU;
            let p = na::Vector2::new(w.cos(), w.sin());
            (p * radius + c).into()
        });

        let vertices = gl::VertexBuffer::immutable(display, points.as_slice()).unwrap();

        Self { vertices }
    }
}

impl RenderData for Circle {
    fn vertices(&self) -> gl::vertex::VerticesSource<'_> {
        (&self.vertices).into()
    }

    fn indices(&self) -> gl::index::IndicesSource<'_> {
        gl::index::IndicesSource::NoIndices {
            primitives: gl::index::PrimitiveType::LineLoop,
        }
    }
}

// in 2D, the list of vertices of a circle is easy to calculate
impl<S: na::RealField + AsPrimitive<f32>> OpenGLRenderable for miroir_shapes::Sphere<S, 2> {
    fn append_render_data(&self, display: &gl::Display, list: &mut List<Box<dyn RenderData>>) {
        list.push(Box::new(Circle::new::<360>(
            self.center.map(|s| s.as_()).into(),
            self.radius().as_(),
            display,
        )))
    }
}
