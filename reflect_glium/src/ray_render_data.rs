use super::*;

pub(crate) struct RayRenderData<const D: usize> {
    // TODO: find another way to draw this, that preserves
    // it's size no matter how far away you are from it
    pub origin: Box<dyn RenderData>,
    pub non_loop_path: gl::VertexBuffer<Vertex<D>>,
    pub loop_path: gl::VertexBuffer<Vertex<D>>,
}

fn path_vertices<const D: usize>(
    ray_path: &reflect::RayPath<D>,
    display: &gl::Display,
) -> (gl::VertexBuffer<Vertex<D>>, gl::VertexBuffer<Vertex<D>>)
where
    Vertex<D>: gl::Vertex,
{
    let (non_loop_pts, loop_pts) = ray_path.all_points();

    let non_loop_pts = Vec::from_iter(
        non_loop_pts
            .iter()
            .copied()
            .chain(
                ray_path
                    .divergence_direction()
                    .map(|dir| non_loop_pts.last().unwrap() + dir.as_ref() * 2000.),
            )
            .map(Vertex::from),
    );
    let loop_pts = Vec::from_iter(loop_pts.iter().copied().map(Vertex::from));

    (
        gl::VertexBuffer::immutable(display, non_loop_pts.as_slice()).unwrap(),
        gl::VertexBuffer::immutable(display, loop_pts.as_slice()).unwrap(),
    )
}

impl RayRenderData<3> {
    pub(crate) fn from_simulation<M: Mirror<3>, R: IntoIterator<Item = Ray<3>>>(
        sim: Simulation<M, R>,
        reflection_limit: usize,
        display: &gl::Display,
    ) -> Vec<RayRenderData<3>> {
        sim.get_ray_paths(reflection_limit)
            .into_iter()
            .map(|ray_path| {
                // we'll change this to a square or circle that's doesn't get scaled by the projection matrix

                // use Sphere for 3D, and Circle for 2D
                let [x, y, z] = ray_path
                    .all_points_raw()
                    .first()
                    .unwrap()
                    .map(|s| s as f32)
                    .into();

                let (non_loop_path, loop_path) = path_vertices(&ray_path, display);

                RayRenderData {
                    origin: Box::new(
                        glium_shapes::sphere::SphereBuilder::new()
                            .scale(0.1, 0.1, 0.1)
                            .translate(x, y, z)
                            .with_divisions(60, 60)
                            .build(display)
                            .unwrap(),
                    ),
                    non_loop_path,
                    loop_path,
                }
            })
            .collect()
    }
}

impl RayRenderData<2> {
    pub(crate) fn from_simulation<M: Mirror<2>, R: IntoIterator<Item = Ray<2>>>(
        sim: Simulation<M, R>,
        reflection_limit: usize,
        display: &gl::Display,
    ) -> Vec<RayRenderData<2>> {
        sim.get_ray_paths(reflection_limit)
            .into_iter()
            .map(|ray_path| {
                // we'll change this to a square or circle that's doesn't get scaled by the projection matrix
                // use Sphere for 3D, and Circle for 2D
                let center = ray_path
                    .all_points_raw()
                    .first()
                    .unwrap()
                    .map(|s| s as f32)
                    .into();

                let (non_loop_path, loop_path) = path_vertices(&ray_path, display);

                RayRenderData {
                    origin: Box::new(FilledCircle::from(Circle::new(center, 0.1, display))),
                    non_loop_path,
                    loop_path,
                }
            })
            .collect()
    }
}
