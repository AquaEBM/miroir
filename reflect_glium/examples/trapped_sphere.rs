use reflect::Ray;
use reflect_mirrors::{EuclideanSphereMirror, PlaneMirror};

fn main() {
    let reflection_cap = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(300);

    let mirrors = (
        EuclideanSphereMirror::new([0., 0., 0.], 4.),
        [
            PlaneMirror::new([[5., 0., 0.], [0., 5., 0.], [0., 0., 5.]]),
            PlaneMirror::new([[-5., 0., 0.], [0., 5., 0.], [0., 0., 5.]]),
            PlaneMirror::new([[0., 5., 0.], [5., 0., 0.], [0., 0., 5.]]),
            PlaneMirror::new([[0., -5., 0.], [5., 0., 0.], [0., 0., 5.]]),
            PlaneMirror::new([[0., 0., 5.], [0., 5., 0.], [5., 0., 0.]]),
            PlaneMirror::new([[0., 0., -5.], [0., 5., 0.], [5., 0., 0.]]),
        ],
    );

    let rays = [Ray::new([4., 3., 0.1], [-1., -1., 0.])];
    reflect_glium::run_simulation(&mirrors, rays, Some(reflection_cap));
}
