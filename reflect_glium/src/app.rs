use super::*;

use gl::index::{NoIndices, PrimitiveType};
use nalgebra::{ComplexField, SVector};
const LINE_STRIP: NoIndices = NoIndices(PrimitiveType::LineStrip);

pub struct App<const D: usize> {
    ray_origins: gl::VertexBuffer<Vertex<D>>,
    ray_paths: Vec<gl::VertexBuffer<Vertex<D>>>,
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

impl<const D: usize> App<D>
where
    Vertex<D>: gl::Vertex,
{
    pub(crate) fn from_simulation<M, R>(
        mirror: &M,
        rays: R,
        display: &gl::Display,
        eps: <M::Scalar as ComplexField>::RealField,
    ) -> Self
    where
        M: Mirror<D> + OpenGLRenderable + ?Sized,
        R: IntoIterator<Item = (Ray<M::Scalar, D>, Option<usize>)>,
        Vertex<D>: From<SVector<M::Scalar, D>>,
    {
        let vertex_shader = if D == 2 {
            r"
            #version 140

            in vec2 position;
            uniform mat4 perspective;
            uniform mat4 view;

            void main() {
                gl_Position = perspective * view * vec4(position, 0.0, 1.0);
            }
        "
        } else if D == 3 {
            r"
            #version 140

            in vec3 position;
            uniform mat4 perspective;
            uniform mat4 view;

            void main() {
                gl_Position = perspective * view * vec4(position, 1.0);
            }
        "
        } else {
            unreachable!()
        };

        let program =
            gl::Program::from_source(display, vertex_shader, FRAGMENT_SHADER_SRC, None).unwrap();

        let starting_pts_program = gl::Program::from_source(
            display,
            vertex_shader,
            FRAGMENT_SHADER_SRC,
            Some(STARTING_POINT_GEOMETRY_SHADER_SRC),
        )
        .unwrap();

        let mut mirrors = List::from(vec![]);

        mirror.append_render_data(display, &mut mirrors);

        let mut mirrors = mirrors.into_inner();

        mirrors.shrink_to_fit();

        let mut vertex_scratch = vec![];
        let mut ray_origins = vec![];

        let mut ray_paths: Vec<_> = rays
            .into_iter()
            .map(|(ray, max_reflections)| {
                let origin = ray.origin.clone().into();

                ray_origins.push(origin);

                vertex_scratch.clear();
                vertex_scratch.push(origin);

                let path = RayPath::new(mirror, ray, eps.clone()).map(Vertex::from);

                if let Some(n) = max_reflections {
                    vertex_scratch.extend(path.take(n));
                } else {
                    vertex_scratch.extend(path);
                }

                gl::VertexBuffer::immutable(display, &vertex_scratch).unwrap()
            })
            .collect();

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
        const DEFAULT_CAMERA_POS: cg::Point3<f32> = cg::Point3::new(0., 0., 5.);
        const DEFAULT_CAMERA_YAW: cg::Deg<f32> = cg::Deg(-90.);
        const DEFAULT_CAMERA_PITCH: cg::Deg<f32> = cg::Deg(0.);
        const SPEED: f32 = 5.;
        const MOUSE_SENSITIVITY: f32 = 1.0;
        const DEFAULT_PROJECCTION_POV: cg::Deg<f32> = cg::Deg(85.);
        const NEAR_PLANE: f32 = 0.0001;
        const FAR_PLANE: f32 = 10000.;

        use glutin::{dpi, event, event_loop, window};

        let mut camera = Camera::new(DEFAULT_CAMERA_POS, DEFAULT_CAMERA_YAW, DEFAULT_CAMERA_PITCH);

        let dpi::PhysicalSize { width, height } = display.gl_window().window().inner_size();

        let mut projection = Projection::new(
            width,
            height,
            DEFAULT_PROJECCTION_POV,
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
                        projection.resize(physical_size.width, physical_size.height);
                    }

                    display.gl_window().resize(physical_size);
                }
                event::WindowEvent::MouseWheel { delta, .. } => {
                    camera_controller.set_scroll(&delta);
                }

                event::WindowEvent::KeyboardInput { input, .. } => {
                    if let Some(keycode) = input.virtual_keycode {
                        camera_controller.process_keyboard(keycode, input.state);
                    }
                }

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
                }
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

    fn render_3d(&self, display: &gl::Display, camera: &Camera, projection: &Projection) {
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

        let perspective: [[f32; 4]; 4] = projection.get_matrix().into();
        let view: [[f32; 4]; 4] = camera.calc_matrix().into();

        let aspect = projection.aspect();

        let params = gl::DrawParameters {
            blend: gl::Blend::alpha_blending(),
            ..Default::default()
        };

        for ray in &self.ray_paths {
            target
                .draw(
                    ray,
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
