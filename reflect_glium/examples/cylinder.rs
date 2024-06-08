use reflect::Ray;
use reflect_mirrors::CylindricalMirror;

fn main() {
    let mirror = CylindricalMirror::new([0., 0., 0.], [10., 0., 0.], 2.);
    let rays = [Ray::new([0., 1., 0.], [0.004, 1., 0.01])];
    reflect_glium::run_simulation(&mirror, rays, None);
}
