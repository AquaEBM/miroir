use reflect_glium::{SimulationParams, SimulationRay, SimulationWindow};
use reflect_mirrors::LineSegment;

fn main() {
    let mirrors = [
        LineSegment::new([[1., 0.], [0., 1.]]),
        LineSegment::new([[0., 1.], [-1., 0.]]),
        LineSegment::new([[1., 0.], [0., -1.]]),
        LineSegment::new([[0., -1.], [-1., 0.]]),
    ];

    let rays = [SimulationRay::new([0.5, 0.33], [1., 1.1])];

    SimulationWindow::default().run(&mirrors, rays, SimulationParams::default());
}
