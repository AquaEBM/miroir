use miroir_glium::{SimulationParams, SimulationRay, SimulationWindow};
use miroir_shapes::{Sphere, Triangle};

fn main() {
    let max = std::env::args()
        .nth(1)
        .map(|s| s.parse().expect("expected a positive integer"))
        .unwrap_or(300);

    const SPHERE_RADIUS: f32 = 4.;
    const CUBE_DIMS: [f32; 3] = [10., 10., 10.];

    let [x, y, z] = CUBE_DIMS.map(|c| c.abs() / 2.);

    // A sphere trapped in a cube
    #[rustfmt::skip]
    let mirrors = (
        Sphere::new([0., 0., 0.], SPHERE_RADIUS),
        [ // faces of the cube, two triangles form a square.
            [
                Triangle::new([[ x,  y,  z], [ x, -y,  z], [ x,  y, -z]]),
                Triangle::new([[ x, -y, -z], [ x, -y,  z], [ x,  y, -z]]),
            ], [
                Triangle::new([[-x,  y,  z], [-x, -y,  z], [-x,  y, -z]]),
                Triangle::new([[-x, -y, -z], [-x, -y,  z], [-x,  y, -z]]),
            ], [
                Triangle::new([[ x,  y,  z], [-x,  y,  z], [ x,  y, -z]]),
                Triangle::new([[-x,  y, -z], [-x,  y,  z], [ x,  y, -z]]),
            ], [
                Triangle::new([[ x, -y,  z], [-x, -y,  z], [ x, -y, -z]]),
                Triangle::new([[-x, -y, -z], [-x, -y,  z], [ x, -y, -z]]),
            ], [
                Triangle::new([[ x,  y,  z], [ x, -y,  z], [-x,  y,  z]]),
                Triangle::new([[-x, -y,  z], [ x, -y,  z], [-x,  y,  z]]),
            ], [
                Triangle::new([[ x,  y, -z], [ x, -y, -z], [-x,  y, -z]]),
                Triangle::new([[-x, -y, -z], [ x, -y, -z], [-x,  y, -z]]),
            ],
        ],
    );

    let rays = [SimulationRay::new([4., 3., 0.1], [-1., -1., 0.]).with_reflection_cap(max)];

    SimulationWindow::default().run(&mirrors, rays, SimulationParams::default());
}
