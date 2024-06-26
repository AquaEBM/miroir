use miroir_glium::{SimulationParams, SimulationRay, SimulationWindow};
use miroir_shapes::Simplex;

fn main() {
    let mirror = Simplex::new([[1., 0.000001], [1., 1.]]);
    let rays = [SimulationRay::new([0., 0.], [1., 0.])];
    SimulationWindow::default().run(&mirror, rays, SimulationParams::default())
}
