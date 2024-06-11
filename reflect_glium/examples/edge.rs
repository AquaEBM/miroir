use reflect::{Float, Ray};
use reflect_mirrors::Simplex;

fn main() {
    let mirror = Simplex::new([[1., 0.000001], [1., 1.]]);
    let rays = [(Ray::new([0., 0.], [1., 0.]), None)];
    reflect_glium::run_simulation(&mirror, rays, Float::EPSILON * 64.)
}
