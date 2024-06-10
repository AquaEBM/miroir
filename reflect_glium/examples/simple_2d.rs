use reflect::{Float, Ray};
use reflect_mirrors::LineSegment;

fn main() {
    let mirrors = [
        LineSegment::new([[1., -1.], [1., 1.]]),
        LineSegment::new([[-1., -1.], [1., 1.]]),
        LineSegment::new([[-1., -4.], [1., -6.]]),
    ];

    let rays = [
        (Ray::new([0.3, 0.], [1., -0.1]), None),
    ];

    reflect_glium::run_simulation(&mirrors, rays, Float::EPSILON * 64.)
}
