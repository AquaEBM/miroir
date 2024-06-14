use reflect_glium::{SimulationParams, SimulationRay, SimulationWindow};
use reflect_mirrors::LineSegment;

fn main() {
    let mirrors = [
        LineSegment::new([[6.69, 1.32], [3.77, 5.08]]),
        LineSegment::new([[7.615, 1.46], [10.635, 4.86]]),
        LineSegment::new([[12.285, 1.195], [14.585, 6.695]]),
        LineSegment::new([[12.255, 5.08], [7.275, 7.6]]),
        LineSegment::new([[5.86, 7.235], [6.42, 3.095]]),
        LineSegment::new([[7.09, 4.01], [7.29, 4.21]]),
        LineSegment::new([[7.76, 5.455], [11.24, 4.675]]),
        LineSegment::new([[10.185, 4.135], [8.005, 4.955]]),
        LineSegment::new([[7.025, 7.69], [7.805, 5.29]]),
        LineSegment::new([[3.56, 4.395], [3.84, 7.415]]),
        LineSegment::new([[2.705, 2.605], [4.045, 0.825]]),
        LineSegment::new([[4.565, 1.385], [8.465, 0.685]]),
        LineSegment::new([[8.6, 1.305], [14.4, 0.525]]),
        LineSegment::new([[13.155, 0.55], [16.655, 3.11]]),
        LineSegment::new([[16.595, 3.645], [17.295, 5.945]]),
        LineSegment::new([[16.585, 6.57], [15.645, 8.57]]),
        LineSegment::new([[14.345, 9.12], [11.485, 9.92]]),
        LineSegment::new([[11.62, 9.72], [4.5, 9.84]]),
        LineSegment::new([[6.51, 10.04], [2.55, 8.32]]),
        LineSegment::new([[3.28, 9.275], [1.44, 6.055]]),
        LineSegment::new([[1.54, 7.33], [1.94, 2.21]]),
        LineSegment::new([[2.045, 3.365], [4.465, 2.145]]),
        LineSegment::new([[11.825, 2.36], [10.365, 3.2]]),
        LineSegment::new([[10.835, 4.225], [12.655, 4.845]]),
        LineSegment::new([[15.575, 3.09], [14.755, 4.97]]),
        LineSegment::new([[14.215, 7.69], [15.435, 7.01]]),
        LineSegment::new([[14.785, 8.6], [11.685, 6.04]]),
        LineSegment::new([[11.12, 6.315], [9.48, 9.495]]),
        LineSegment::new([[5.855, 8.23], [8.435, 8.39]]),
        LineSegment::new([[5.35, 6.205], [5.03, 8.425]]),
        LineSegment::new([[5.635, 5.88], [4.335, 5.16]]),
        LineSegment::new([[2.08, 2.905], [4.76, 4.565]]),
        LineSegment::new([[5.78, 1.395], [8.46, 3.055]]),
        LineSegment::new([[16.1, 1.885], [16.54, 4.345]]),
        LineSegment::new([[16.265, 2.85], [17.845, 4.77]]),
        LineSegment::new([[9.785, 9.865], [12.565, 9.445]]),
        LineSegment::new([[17.21, 7.165], [15.13, 10.145]]),
        LineSegment::new([[10.455, 1.74], [7.395, 1.62]]),
    ];

    let rays = [SimulationRay::new([9., 5.], [1., 2.])];

    SimulationWindow::default().run(&mirrors, rays, SimulationParams::default())
}
