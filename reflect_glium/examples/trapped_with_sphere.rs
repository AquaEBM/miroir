use reflect::{Float, Ray};
use reflect_mirrors::{Sphere, Triangle};

fn main() {
    let reflection_cap = std::env::args()
        .nth(1)
        .map(|s| {
            s.parse()
                .expect("reflection cap must be a positive integer")
        })
        .unwrap_or(300);

    // A sphere trapped in a cube

    let mirrors = (
        Sphere::new([0., 0., 0.], 4.),
        [
            // faces of the cube
            Triangle::new([[5., 5., 5.], [5., -5., 5.], [5., 5., -5.]]),
            Triangle::new([[5., -5., -5.], [5., -5., 5.], [5., 5., -5.]]),
            Triangle::new([[-5., 5., 5.], [-5., -5., 5.], [-5., 5., -5.]]),
            Triangle::new([[-5., -5., -5.], [-5., -5., 5.], [-5., 5., -5.]]),
            Triangle::new([[5., 5., 5.], [-5., 5., 5.], [5., 5., -5.]]),
            Triangle::new([[-5., 5., -5.], [-5., 5., 5.], [5., 5., -5.]]),
            Triangle::new([[5., -5., 5.], [-5., -5., 5.], [5., -5., -5.]]),
            Triangle::new([[-5., -5., -5.], [-5., -5., 5.], [5., -5., -5.]]),
            Triangle::new([[5., 5., 5.], [5., -5., 5.], [-5., 5., 5.]]),
            Triangle::new([[-5., -5., 5.], [5., -5., 5.], [-5., 5., 5.]]),
            Triangle::new([[5., 5., -5.], [5., -5., -5.], [-5., 5., -5.]]),
            Triangle::new([[-5., -5., -5.], [5., -5., -5.], [-5., 5., -5.]]),
        ],
    );

    let rays = [Ray::new([4., 3., 0.1], [-1., -1., 0.])];
    reflect_glium::run_simulation(&mirrors, rays, Some(reflection_cap), Float::EPSILON * 64.);
}
