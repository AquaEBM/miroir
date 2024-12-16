use miroir::Ray;
use miroir_glium::{RayParams, SimulationParams, SimulationWindow};
use miroir_shapes::Cylinder;

fn main() {
    let mirror = Cylinder::new([0f32, 0., 0.], [10., 0., 0.], 2.);
    let rays = [(
        Ray::new_normalize([0., 1., 0.], [0.004, 1., 0.01]),
        RayParams::default(),
    )];
    SimulationWindow::default().display(&mirror, rays, SimulationParams {
        mirror_color: [0., 0., 1., 0.1],
        bg_color: [0.01, 0.01, 0.05, 1.],
    });
}
