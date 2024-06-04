use super::*;

pub(crate) struct RayRenderData<const D: usize> {
    pub(crate) path: gl::VertexBuffer<Vertex<D>>,
}

impl<const D: usize> RayRenderData<D>
where
    Vertex<D>: gl::Vertex,
{
    pub(crate) fn from_ray<M: Mirror<D> + ?Sized>(
        vertex_scratch: &mut Vec<Vertex<D>>,
        mirror: &M,
        ray: Ray<D>,
        reflection_limit: Option<usize>,
        display: &gl::Display,
    ) -> Self {

        vertex_scratch.push(ray.origin.into());

        let path = get_ray_path(mirror, ray).map(Vertex::from);

        if let Some(n) = reflection_limit {
            vertex_scratch.extend(path.take(n))
        } else {
            vertex_scratch.extend(path)
        }

        let path = gl::VertexBuffer::immutable(display, &vertex_scratch).unwrap();

        Self { path }
    }
}
