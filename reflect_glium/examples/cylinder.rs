use core::iter;

use reflect;
use reflect_mirrors;

fn main() {
    let mirror = reflect_mirrors::CylindricalMirror::new(
        [[0.0, 0.0, 0.0].into(), [10.0, 0.0, 0.0].into()],
        2.0,
    )
    .unwrap();

    let rays = iter::once(reflect::Ray {
        origin: [0.0, 1.0, 0.0].into(),
        direction: reflect::nalgebra::Unit::new_normalize([0.004, 1.0, 0.01].into()),
    });

    reflect_glium::run_simulation(&mirror, rays, None);
}
