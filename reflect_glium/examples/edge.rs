use reflect::Float;
use reflect_glium::SimulationRay;
use reflect_mirrors::Simplex;

fn main() {
    let mirror = Simplex::new([[1., 0.000001], [1., 1.]]);
    let rays = [SimulationRay::new([0., 0.], [1., 0.])];
    reflect_glium::run_simulation(&mirror, rays, Float::EPSILON * 64.)
}
