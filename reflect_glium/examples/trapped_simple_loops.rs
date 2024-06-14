use reflect_glium::{SimulationParams, SimulationRay, SimulationWindow};
use reflect_mirrors::{LineSegment, Sphere};

fn main() {
    let mirrors = (
        Sphere::new([4., 0.], 1.),
        [
            LineSegment::new([[0., -1.], [0., 1.]]),
            LineSegment::new([[2., 1.], [2., -1.]]),
            LineSegment::new([[0., 1.], [2., 1.]]),
            LineSegment::new([[0., -1.], [2., -1.]]),
        ],
    );

    let rays = [
        SimulationRay::new([0., 0.], [1., 1.]),
        SimulationRay::new([0.25, 0.5], [1., 0.]),
        SimulationRay::new([4., 0.5], [1., 0.]),
    ];

    SimulationWindow::default().run(&mirrors, rays, SimulationParams::default());
}
