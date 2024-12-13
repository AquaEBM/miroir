use miroir::Ray;
use miroir_glium::{RayParams, SimulationParams, SimulationWindow};
use miroir_shapes::LineSegment;

fn main() {
    let mirrors = [
        LineSegment::new([[1., 0.], [0., 1.]]),
        LineSegment::new([[0., 1.], [-1., 0.]]),
        LineSegment::new([[1., 0.], [0., -1.]]),
        LineSegment::new([[0., -1.], [-1., 0.]]),
    ];

    let rays = [(
        Ray::new_normalize([0.5, 0.33], [1., 1.1]),
        RayParams {
            reflection_cap: Some(100),
            ..Default::default()
        },
    )];

    SimulationWindow::default().display(&mirrors, rays, SimulationParams::default());
}
