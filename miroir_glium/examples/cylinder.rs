use miroir::na::Unit;
use miroir_glium::{RayParams, SimulationParams, SimulationWindow};
use miroir_shapes::Cylinder;

fn main() {
    let mirror = Cylinder::new([0f32, 0., 0.], [10., 0., 0.], 2.);
    let rays = [(
        [0., 1., 0.].into(),
        Unit::new_normalize([0.004, 1., 0.01].into()),
        RayParams::default(),
    )];
    SimulationWindow::default().display(
        &mirror,
        rays,
        SimulationParams {
            mirror_color: [0., 0., 1., 0.1],
            bg_color: [0.015, 0.01, 0.05, 1.],
        },
    );
}
