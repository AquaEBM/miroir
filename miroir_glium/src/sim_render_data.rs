use core::f32::consts::{FRAC_PI_2, PI};

use super::*;

use camera::{Camera, CameraController};

use gl::index::{NoIndices, PrimitiveType};
use nalgebra::{Perspective3, Point3};
const LINE_STRIP: NoIndices = NoIndices(PrimitiveType::LineStrip);

pub struct SimulationRenderData<const D: usize> {
    ray_origins: gl::VertexBuffer<Vertex<D>>,
    ray_paths: Vec<(gl::VertexBuffer<Vertex<D>>, gl::VertexBuffer<Vertex<D>>)>,
    mirrors: Vec<Box<dyn RenderData>>,
    program: gl::Program,
    starting_pts_program: gl::Program,
}

const FRAGMENT_SHADER_SRC: &str = r"
    #version 140

    uniform vec4 color_vec;

    out vec4 color;

    void main() {
        color = color_vec;
    }
";

const STARTING_POINT_GEOMETRY_SHADER_SRC: &str = r"
    #version 330

    layout (points) in;
    layout (line_strip, max_vertices = 4) out;

    mat4 translate(vec2 delta) {
        return(mat4(
            vec4(1.0, 0.0, 0.0, 0.0),
            vec4(0.0, 1.0, 0.0, 0.0),
            vec4(0.0, 0.0, 1.0, 0.0),
            vec4(delta, 0.0, 1.0)
        ));
    }

    uniform float aspect;

    void main() {
        vec4 pos = gl_in[0].gl_Position;

        float v = 0.025;

        vec2 t1 = vec2(v, v * aspect);

        gl_Position = translate(t1) * pos;
        EmitVertex();

        gl_Position = translate(-t1) * pos;
        EmitVertex();
        EndPrimitive();

        vec2 t2 = vec2(v, -v * aspect);

        gl_Position = translate(t2) * pos;
        EmitVertex();

        gl_Position = translate(-t2) * pos;
        EmitVertex();
        EndPrimitive();
    }
";

const VERTEX_SHADER_2D_SRC: &str = r"
    #version 140

    in vec2 position;
    uniform mat4 perspective;
    uniform mat4 view;

    void main() {
        gl_Position = perspective * view * vec4(position, 0.0, 1.0);
    }
";

const VERTEX_SHADER_3D_SRC: &str = r"
    #version 140

    in vec3 position;
    uniform mat4 perspective;
    uniform mat4 view;

    void main() {
        gl_Position = perspective * view * vec4(position, 1.0);
    }
";

impl<const D: usize> SimulationRenderData<D>
where
    Vertex<D>: gl::Vertex,
{
    pub(crate) fn from_simulation<M, R>(
        mirror: &M,
        rays: R,
        display: &gl::Display,
        params: SimulationParams<M::Scalar>,
    ) -> Self
    where
        M: Mirror<D, Scalar: RealField> + OpenGLRenderable + ?Sized,
        R: IntoIterator<Item = SimulationRay<M::Scalar, D>>,
        Vertex<D>: From<SVector<M::Scalar, D>>,
    {
        let vertex_shader_src = if D == 2 {
            VERTEX_SHADER_2D_SRC
        } else if D == 3 {
            VERTEX_SHADER_3D_SRC
        } else {
            unreachable!()
        };

        let program =
            gl::Program::from_source(display, vertex_shader_src, FRAGMENT_SHADER_SRC, None)
                .unwrap();

        let starting_pts_program = gl::Program::from_source(
            display,
            vertex_shader_src,
            FRAGMENT_SHADER_SRC,
            Some(STARTING_POINT_GEOMETRY_SHADER_SRC),
        )
        .unwrap();

        let mut mirrors = List(vec![]);

        mirror.append_render_data(display, &mut mirrors);

        let mut vertex_scratch = vec![];
        let mut pt_scratch = vec![];

        let mut mirrors = mirrors.into_inner();
        let mut ray_origins = vec![];
        let mut ray_paths = vec![];

        for SimulationRay {
            ray,
            reflection_cap,
        } in rays
        {
            let origin = ray.origin.clone();

            ray_origins.push(Vertex::from(origin.clone()));

            vertex_scratch.clear();
            pt_scratch.push(origin);

            let mut path = RayPath {
                mirror,
                ray,
                eps: params.epsilon.clone(),
            };

            let path_iter = path.by_ref();

            let outcome = 'block: {
                if let Some(n) = reflection_cap {
                    for pt in path_iter.take(n) {
                        let out = loop_index(&pt_scratch, &pt, &params.epsilon);
                        if out.is_some() {
                            break 'block Some(out);
                        }

                        pt_scratch.push(pt);
                    }

                    (pt_scratch.len() <= n).then_some(None)
                } else {
                    for pt in path_iter {
                        let out = loop_index(&pt_scratch, &pt, &params.epsilon);
                        if out.is_some() {
                            break 'block Some(out);
                        }

                        pt_scratch.push(pt);
                    }

                    Some(None)
                }
            };

            let loop_path = if let Some(Some(loop_index)) = outcome {
                vertex_scratch.extend(pt_scratch.drain(loop_index..).map(Vertex::from));
                gl::VertexBuffer::immutable(display, &vertex_scratch).unwrap()
            } else {
                gl::VertexBuffer::empty_immutable(display, 0).unwrap()
            };

            vertex_scratch.clear();
            vertex_scratch.extend(pt_scratch.drain(..).map(Vertex::from));

            if let Some(None) = outcome {
                let last = *vertex_scratch.last().unwrap();
                let dir = Vertex::from(path.ray.dir.clone().into_inner());
                vertex_scratch.push(last + 20000. * dir);
            }

            let non_loop_path = gl::VertexBuffer::immutable(display, &vertex_scratch).unwrap();

            ray_paths.push((non_loop_path, loop_path));
        }

        mirrors.shrink_to_fit();
        ray_paths.shrink_to_fit();

        Self {
            ray_origins: gl::VertexBuffer::immutable(display, &ray_origins).unwrap(),
            ray_paths,
            mirrors,
            program,
            starting_pts_program,
        }
    }

    pub(crate) fn run(self, display: gl::Display, events_loop: glutin::event_loop::EventLoop<()>) {
        const DEFAULT_CAMERA_POS: Point3<f32> = Point3::new(0., 0., 0.);
        const DEFAULT_CAMERA_YAW: f32 = -FRAC_PI_2;
        const DEFAULT_CAMERA_PITCH: f32 = 0.;
        const SPEED: f32 = 5.;
        const MOUSE_SENSITIVITY: f32 = 1.0;
        const DEFAULT_PROJECTION_FOV: f32 = 85. / 180. * PI;
        const NEAR_PLANE: f32 = 0.001;
        const FAR_PLANE: f32 = 1000.;

        use glutin::{dpi, event, event_loop, window};

        let mut camera = Camera::new(DEFAULT_CAMERA_POS, DEFAULT_CAMERA_YAW, DEFAULT_CAMERA_PITCH);

        let dpi::PhysicalSize { width, height } = display.gl_window().window().inner_size();

        let mut projection = Perspective3::new(
            width as f32 / height as f32,
            DEFAULT_PROJECTION_FOV,
            NEAR_PLANE,
            FAR_PLANE,
        );

        let mut camera_controller = CameraController::new(SPEED, MOUSE_SENSITIVITY);

        let mut last_render_time = std::time::Instant::now();
        let mut mouse_pressed = false;

        events_loop.run(move |ev, _, control_flow| match ev {
            event::Event::WindowEvent { event, .. } => match event {
                event::WindowEvent::CloseRequested => *control_flow = event_loop::ControlFlow::Exit,

                event::WindowEvent::Resized(physical_size) => {
                    if physical_size.width > 0 && physical_size.height > 0 {
                        projection
                            .set_aspect(physical_size.width as f32 / physical_size.height as f32);
                    }

                    display.gl_window().resize(physical_size);
                },

                event::WindowEvent::KeyboardInput { input, .. } => {
                    if let Some(keycode) = input.virtual_keycode {
                        camera_controller.process_keyboard(keycode, input.state);
                    }
                },

                event::WindowEvent::MouseInput { button, state, .. } => {
                    if button == event::MouseButton::Left {
                        match state {
                            event::ElementState::Pressed => {
                                mouse_pressed = true;
                                display
                                    .gl_window()
                                    .window()
                                    .set_cursor_grab(window::CursorGrabMode::Locked)
                                    .or_else(|_| {
                                        display
                                            .gl_window()
                                            .window()
                                            .set_cursor_grab(window::CursorGrabMode::Confined)
                                    })
                                    .unwrap();

                                display.gl_window().window().set_cursor_visible(false);
                            }

                            event::ElementState::Released => {
                                mouse_pressed = false;
                                display
                                    .gl_window()
                                    .window()
                                    .set_cursor_grab(window::CursorGrabMode::None)
                                    .unwrap();
                                display.gl_window().window().set_cursor_visible(true);
                            }
                        }
                    }
                },
                _ => {}
            },
            event::Event::RedrawRequested(_) => {
                let now = time::Instant::now();
                let dt = now - last_render_time;
                last_render_time = now;

                camera_controller.update_camera(&mut camera, dt);
                self.render_3d(&display, &camera, &projection);
            }
            event::Event::MainEventsCleared => display.gl_window().window().request_redraw(),
            event::Event::DeviceEvent {
                event: event::DeviceEvent::MouseMotion { delta, .. },
                ..
            } => {
                if mouse_pressed {
                    let inner_window_size = display.gl_window().window().inner_size();

                    display
                        .gl_window()
                        .window()
                        .set_cursor_position(dpi::PhysicalPosition {
                            x: inner_window_size.width / 2,
                            y: inner_window_size.height / 2,
                        })
                        .unwrap();
                    camera_controller.set_mouse_delta(delta.0, delta.1)
                }
            }
            _ => (),
        });
    }

    fn render_3d(&self, display: &gl::Display, camera: &Camera, projection: &Perspective3<f32>) {
        const RAY_LOOP_COL: [f32; 4] = [0.9, 0.2, 0.9, 1.0];
        const RAY_NON_LOOP_COL: [f32; 4] = [0.7, 0.7, 0.7, 0.9];
        let mirror_color = if D == 3 {
            [0.05f32, 0.2, 0.2, 0.4]
        } else if D == 2 {
            [0.15, 0.5, 0.5, 1.0]
        } else {
            unreachable!();
        };

        let mut target = display.draw();

        use gl::Surface;
        target.clear_color_and_depth((0.01, 0.01, 0.05, 1.), 1.0);

        let perspective: [[_; 4]; 4] = projection.as_matrix().clone().into();
        let view: [[_; 4]; 4] = camera.calc_matrix().into();

        let aspect = projection.aspect();

        let params = gl::DrawParameters {
            blend: gl::Blend::alpha_blending(),
            ..Default::default()
        };

        for (non_loop_path, loop_path) in &self.ray_paths {
            target
                .draw(
                    non_loop_path,
                    LINE_STRIP,
                    &self.program,
                    &gl::uniform! {
                        perspective: perspective,
                        view: view,
                        color_vec: RAY_NON_LOOP_COL,
                    },
                    &params,
                )
                .unwrap();

            target
                .draw(
                    loop_path,
                    NoIndices(PrimitiveType::LineLoop),
                    &self.program,
                    &gl::uniform! {
                        perspective: perspective,
                        view: view,
                        color_vec: RAY_LOOP_COL,
                    },
                    &params,
                )
                .unwrap();
        }

        for render_data in self.mirrors.iter().map(Box::as_ref) {
            target
                .draw(
                    render_data.vertices(),
                    render_data.indices(),
                    &self.program,
                    &gl::uniform! {
                        perspective: perspective,
                        view: view,
                        color_vec: mirror_color,
                    },
                    &params,
                )
                .unwrap();
        }

        target
            .draw(
                &self.ray_origins,
                NoIndices(PrimitiveType::Points),
                &self.starting_pts_program,
                &gl::uniform! {
                    perspective: perspective,
                    view: view,
                    // red
                    color_vec: [1.0f32, 0.0, 0.0, 1.0],
                    aspect: aspect,
                },
                &params,
            )
            .unwrap();

        target.finish().unwrap();

        display.gl_window().window().request_redraw();
    }
}
