use reflect_glium::{SimulationParams, SimulationRay, SimulationWindow};
use reflect_mirrors::LineSegment;

fn main() {
    let mirrors = [
        LineSegment::new([[1., -1.], [1., 1.]]),
        LineSegment::new([[-1., -1.], [1., 1.]]),
        LineSegment::new([[-1., -4.], [1., -6.]]),
    ];

    let rays = [SimulationRay::new([0.3, 0.], [1., -0.1])];

    SimulationWindow::default().run(&mirrors, rays, SimulationParams::default())
}
