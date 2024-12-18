use miroir::Ray;
use miroir_glium::{RayParams, SimulationParams, SimulationWindow};
use miroir_shapes::Simplex;

fn main() {
    let mirror = Simplex::new([[1f32, 0.000001], [1., 1.]]);
    let rays = [(Ray::new_normalize([0., 0.], [1., 0.]), RayParams::default())];
    SimulationWindow::default().display(
        &mirror,
        rays,
        SimulationParams {
            mirror_color: [0., 1., 0., 1.],
            bg_color: [0.01, 0.01, 0.05, 1.],
        },
    )
}
