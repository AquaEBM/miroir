use reflect::Float;
use reflect_glium::SimulationRay;
use reflect_mirrors::Cylinder;

fn main() {
    let mirror = Cylinder::new([0., 0., 0.], [10., 0., 0.], 2.);
    let rays = [SimulationRay::new([0., 1., 0.], [0.004, 1., 0.01])];
    reflect_glium::run_simulation(&mirror, rays, Float::EPSILON * 64.);
}
