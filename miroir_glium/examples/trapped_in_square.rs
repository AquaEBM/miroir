use miroir::na::Unit;
use miroir_glium::{RayParams, SimulationParams, SimulationWindow};
use miroir_shapes::LineSegment;

fn main() {
    let mirrors = [
        LineSegment::new([[1f32, 0.], [0., 1.]]),
        LineSegment::new([[0., 1.], [-1., 0.]]),
        LineSegment::new([[1., 0.], [0., -1.]]),
        LineSegment::new([[0., -1.], [-1., 0.]]),
    ];

    let rays = [(
        [0.5, 0.33].into(),
        Unit::new_normalize([1., 1.1].into()),
        RayParams {
            reflection_cap: Some(100),
            ..Default::default()
        },
    )];

    SimulationWindow::default().display(
        &mirrors,
        rays,
        SimulationParams {
            mirror_color: [0., 1., 0., 1.],
            bg_color: [0.01, 0.01, 0.05, 1.],
        },
    );
}
