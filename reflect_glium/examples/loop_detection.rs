use reflect::{Float, Ray};
use reflect_mirrors::{LineSegment, Sphere};

fn main() {
    let mirrors = (
        Sphere::new([4., 0.], 1.),
        [
            LineSegment::new([[0., -1.], [0., 1.]]),
            LineSegment::new([[2., 1.], [2., -1.]]),
            LineSegment::new([[0., 1.], [2., 1.]]),
            LineSegment::new([[0., -1.], [2., -1.]]),
        ],
    );

    let rays = [
        Ray::new([0., 0.], [1., 1.]),
        Ray::new([0., 0.], [1., 0.]),
        Ray::new([4., 0.5], [1., 0.]),
    ];

    reflect_glium::run_simulation(&mirrors, rays, Some(5), Float::EPSILON * 64.);
}
