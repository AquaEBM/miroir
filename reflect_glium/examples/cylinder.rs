use reflect::{Float, Ray};
use reflect_mirrors::Cylinder;

fn main() {
    let mirror = Cylinder::new([0., 0., 0.], [10., 0., 0.], 2.);
    let rays = [
        (Ray::new([0., 1., 0.], [0.004, 1., 0.01]), None),
    ];
    reflect_glium::run_simulation(&mirror, rays, Float::EPSILON * 64.);
}
