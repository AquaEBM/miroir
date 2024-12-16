use miroir::Ray;
use miroir_glium::{RayParams, SimulationParams, SimulationWindow};
use miroir_shapes::{LineSegment, Sphere};

fn main() {
    let mirrors = (
        Sphere::new([4f32, 0.], 1.),
        [
            LineSegment::new([[0., -1.], [0., 1.]]),
            LineSegment::new([[2., 1.], [2., -1.]]),
            LineSegment::new([[0., 1.], [2., 1.]]),
            LineSegment::new([[0., -1.], [2., -1.]]),
        ],
    );

    let params = RayParams {
        reflection_cap: Some(100),
        ..Default::default()
    };

    let rays = [
        (Ray::new_normalize([0., 0.], [1., 1.]), params),
        (Ray::new_normalize([0.25, 0.5], [1., 0.]), params),
        (Ray::new_normalize([4., 0.5], [1., 0.]), params),
    ];

    SimulationWindow::default().display(
        &mirrors,
        rays,
        SimulationParams {
            mirror_color: [0., 1., 0., 1.],
            bg_color: [0.01, 0.01, 0.05, 1.],
        },
    );
}
