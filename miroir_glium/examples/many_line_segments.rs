use miroir_glium::{SimulationParams, SimulationRay, SimulationWindow};
use miroir_shapes::LineSegment;

fn main() {
    let mirrors = [
        LineSegment::new([[-3.306, -3.677], [-6.23, 0.08]]),
        LineSegment::new([[-2.385, -3.54], [0.634, -0.136]]),
        LineSegment::new([[2.285, -3.804], [4.585, 1.695]]),
        LineSegment::new([[2.255, 0.08], [-2.724, 2.596]]),
        LineSegment::new([[-4.14, 2.235], [-3.58, -1.904]]),
        LineSegment::new([[-2.91, -0.99], [-2.71, -0.79]]),
        LineSegment::new([[-2.24, 0.455], [1.240, -0.325]]),
        LineSegment::new([[0.186, -0.865], [-1.994, -0.044]]),
        LineSegment::new([[-2.975, 2.69], [-2.195, 0.294]]),
        LineSegment::new([[-6.435, -0.605], [-6.16, 2.415]]),
        LineSegment::new([[-7.295, -2.395], [-5.955, -4.175]]),
        LineSegment::new([[-5.435, -3.615], [-1.535, -4.315]]),
        LineSegment::new([[-1.4, -3.695], [4.4, -4.475]]),
        LineSegment::new([[3.154, -4.45], [6.655, -1.891]]),
        LineSegment::new([[6.594, -1.355], [7.295, 0.945]]),
        LineSegment::new([[6.585, 1.57], [5.645, 3.573]]),
        LineSegment::new([[4.345, 4.11], [1.484, 4.92]]),
        LineSegment::new([[1.612, 4.721], [-5.5, 4.84]]),
        LineSegment::new([[-3.49, 5.03], [-7.45, 3.320]]),
        LineSegment::new([[-6.721, 4.275], [-8.56, 1.055]]),
        LineSegment::new([[-8.46, 2.33], [-8.06, -2.79]]),
        LineSegment::new([[-7.955, -1.635], [-5.535, -2.855]]),
        LineSegment::new([[1.824, -2.64], [0.365, -1.798]]),
        LineSegment::new([[0.836, -0.775], [2.654, -0.155]]),
        LineSegment::new([[5.574, -1.91], [4.755, -0.03]]),
        LineSegment::new([[4.215, 2.69], [5.435, 2.01]]),
        LineSegment::new([[4.785, 3.596], [1.685, 1.04]]),
        LineSegment::new([[1.112, 1.315], [-0.516, 4.494]]),
        LineSegment::new([[-4.145, 3.23], [-1.564, 3.39]]),
        LineSegment::new([[-4.65, 1.205], [-4.97, 3.426]]),
        LineSegment::new([[-4.365, 0.879], [-5.66, 0.16]]),
        LineSegment::new([[-7.92, -2.095], [-5.24, -0.435]]),
        LineSegment::new([[-4.22, -3.605], [-1.53, -1.945]]),
        LineSegment::new([[6.101, -3.115], [6.53, -0.655]]),
        LineSegment::new([[6.265, -2.15], [7.84, -0.23]]),
        LineSegment::new([[-0.215, 4.865], [2.564, 4.44]]),
        LineSegment::new([[7.211, 2.165], [5.131, 5.14]]),
        LineSegment::new([[0.455, -3.26], [-2.605, -3.38]]),
    ];

    let rays = [SimulationRay::new([-1., 0.], [1., 1.6])];

    SimulationWindow::default().run(&mirrors, rays, SimulationParams::default());
}
