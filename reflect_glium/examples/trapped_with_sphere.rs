use reflect::{Float, Ray};
use reflect_mirrors::{Sphere, Triangle};

#[rustfmt::skip]
fn main() {
    let max = std::env::args()
        .nth(1)
        .map(|s| s.parse().expect("expected a positive integer"))
        .unwrap_or(300);

    // A sphere trapped in a cube
    let mirrors = (
        Sphere::new([0., 0., 0.], 4.),
        [
            // faces of the cube, use two triangles to form a square
            Triangle::new([[ 5.,  5.,  5.], [ 5., -5.,  5.], [ 5.,  5., -5.]]),
            Triangle::new([[ 5., -5., -5.], [ 5., -5.,  5.], [ 5.,  5., -5.]]),
            Triangle::new([[-5.,  5.,  5.], [-5., -5.,  5.], [-5.,  5., -5.]]),
            Triangle::new([[-5., -5., -5.], [-5., -5.,  5.], [-5.,  5., -5.]]),
            Triangle::new([[ 5.,  5.,  5.], [-5.,  5.,  5.], [ 5.,  5., -5.]]),
            Triangle::new([[-5.,  5., -5.], [-5.,  5.,  5.], [ 5.,  5., -5.]]),
            Triangle::new([[ 5., -5.,  5.], [-5., -5.,  5.], [ 5., -5., -5.]]),
            Triangle::new([[-5., -5., -5.], [-5., -5.,  5.], [ 5., -5., -5.]]),
            Triangle::new([[ 5.,  5.,  5.], [ 5., -5.,  5.], [-5.,  5.,  5.]]),
            Triangle::new([[-5., -5.,  5.], [ 5., -5.,  5.], [-5.,  5.,  5.]]),
            Triangle::new([[ 5.,  5., -5.], [ 5., -5., -5.], [-5.,  5., -5.]]),
            Triangle::new([[-5., -5., -5.], [ 5., -5., -5.], [-5.,  5., -5.]]),
        ],
    );

    let rays = [(Ray::new([4., 3., 0.1], [-1., -1., 0.]), Some(max))];

    reflect_glium::run_simulation(&mirrors, rays, Float::EPSILON * 64.);
}
