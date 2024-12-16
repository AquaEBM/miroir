use miroir::Ray;
use miroir_glium::{RayParams, SimulationParams, SimulationWindow};
use miroir_shapes::LineSegment;

fn main() {
    let mirrors = [
        LineSegment::new([[1f32, -1.], [1., 1.]]),
        LineSegment::new([[-1., -1.], [1., 1.]]),
        LineSegment::new([[-1., -4.], [1., -6.]]),
    ];

    let rays = [(
        Ray::new_normalize([0.3, 0.], [1., -0.1]),
        RayParams::default(),
    )];

    SimulationWindow::default().display(
        &mirrors,
        rays,
        SimulationParams {
            mirror_color: [0., 1., 0., 1.],
            bg_color: [0.01, 0.01, 0.05, 1.],
        },
    )
}
