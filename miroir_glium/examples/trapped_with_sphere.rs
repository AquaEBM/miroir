use miroir::Ray;
use miroir_glium::{RayParams, SimulationParams, SimulationWindow};
use miroir_shapes::{Sphere, Triangle};

fn main() {
    let max = std::env::args()
        .nth(1)
        .map(|s| s.parse().expect("expected a positive integer"))
        .unwrap_or(300);

    const SPHERE_RADIUS: f64 = 4.;
    const CUBE_DIMS: [f64; 3] = [10., 10., 10.];

    let [x, y, z] = CUBE_DIMS.map(|c| c.abs() / 2.);

    // A sphere trapped in a cube
    let mirrors = (
        Sphere::new([0., 0., 0.], SPHERE_RADIUS),
        [
            // faces of the cube, two triangles form a square.
            [
                Triangle::new([[x, y, z], [x, -y, z], [x, y, -z]]),
                Triangle::new([[x, -y, -z], [x, -y, z], [x, y, -z]]),
            ],
            [
                Triangle::new([[-x, y, z], [-x, -y, z], [-x, y, -z]]),
                Triangle::new([[-x, -y, -z], [-x, -y, z], [-x, y, -z]]),
            ],
            [
                Triangle::new([[x, y, z], [-x, y, z], [x, y, -z]]),
                Triangle::new([[-x, y, -z], [-x, y, z], [x, y, -z]]),
            ],
            [
                Triangle::new([[x, -y, z], [-x, -y, z], [x, -y, -z]]),
                Triangle::new([[-x, -y, -z], [-x, -y, z], [x, -y, -z]]),
            ],
            [
                Triangle::new([[x, y, z], [x, -y, z], [-x, y, z]]),
                Triangle::new([[-x, -y, z], [x, -y, z], [-x, y, z]]),
            ],
            [
                Triangle::new([[x, y, -z], [x, -y, -z], [-x, y, -z]]),
                Triangle::new([[-x, -y, -z], [x, -y, -z], [-x, y, -z]]),
            ],
        ],
    );

    let rays = [(
        Ray::new_normalize([4., 3., 0.1], [-1., -1., 0.]),
        RayParams {
            reflection_cap: Some(max),
            ..Default::default()
        },
    )];

    SimulationWindow::default().display(
        &mirrors,
        rays,
        SimulationParams {
            mirror_color: [0., 0., 1., 0.1],
            bg_color: [0.015, 0.01, 0.05, 1.],
        }
    );
}
