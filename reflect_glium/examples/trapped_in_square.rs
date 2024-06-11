use reflect::{Float, Ray};
use reflect_mirrors::LineSegment;

fn main() {
    let mirrors = [
        LineSegment::new([[1., 0.], [0., 1.]]),
        LineSegment::new([[0., 1.], [-1., 0.]]),
        LineSegment::new([[1., 0.], [0., -1.]]),
        LineSegment::new([[0., -1.], [-1., 0.]]),
    ];

    let rays = [(Ray::new([0.5, 0.33], [1., 1.1]), None)];

    reflect_glium::run_simulation(&mirrors, rays, Float::EPSILON * 64.);
}
