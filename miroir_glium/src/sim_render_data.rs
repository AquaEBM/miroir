use core::f32::consts::{FRAC_PI_2, PI};

use super::*;

use camera::{Camera, CameraController};

use gl::index::{NoIndices, PrimitiveType};
use nalgebra::{Perspective3, Point3};
const LINE_STRIP: NoIndices = NoIndices(PrimitiveType::LineStrip);

struct RayPath<V: Copy> {
    color: [f32 ; 4],
    non_loop_path: gl::VertexBuffer<V>,
    loop_path: Option<([f32; 4], gl::VertexBuffer<V>)>,
}

pub struct SimulationRenderData<V: Copy> {
    ray_origins: gl::VertexBuffer<V>,
    ray_paths: Vec<RayPath<V>>,
    mirrors: Vec<Box<dyn RenderData>>,
    program: gl::Program,
    starting_pts_program: gl::Program,
    global_params: SimulationParams,
}

const FRAGMENT_SHADER_SRC: &str = r"#version 140

uniform vec4 color_vec;
out vec4 color;

void main() {
    color = color_vec;
}";

const STARTING_POINT_GEOMETRY_SHADER_SRC: &str = r"#version 330

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
}";

impl<V: GLSimulationVertex + 'static> SimulationRenderData<V> {
    pub(crate) fn from_simulation<
        H: Hyperplane<Vector: VMulAdd + Vector + ToGLVertex<Vertex = V>>,
    >(
        mirror: &(impl Mirror<H> + OpenGLRenderable + ?Sized),
        rays: impl IntoIterator<Item = (Ray<H::Vector>, RayParams<Scalar<H>>)>,
        display: &gl::Display,
        global_params: SimulationParams,
    ) -> Self
    where
        Scalar<H>: Copy + 'static,
        f64: AsPrimitive<Scalar<H>>,
    {
        let program =
            gl::Program::from_source(display, V::SHADER_SRC, FRAGMENT_SHADER_SRC, None).unwrap();

        let starting_pts_program = gl::Program::from_source(
            display,
            V::SHADER_SRC,
            FRAGMENT_SHADER_SRC,
            Some(STARTING_POINT_GEOMETRY_SHADER_SRC),
        )
        .unwrap();

        let mut mirrors = List(vec![]);

        mirror.append_render_data(display, &mut mirrors);

        let mut vertex_scratch = vec![];

        let mut mirrors = mirrors.into_inner();
        let mut ray_origins = vec![];
        let mut ray_paths = vec![];

        for (mut ray, params) in rays {
            ray_origins.push(ray.pos.to_gl_vertex());

            vertex_scratch.clear();
            vertex_scratch.push(ray.pos.to_gl_vertex());

            let mut count = 0;
            let mut outcome: Result<bool, usize> = Ok(true);

            while let Some((dist, dir)) = ray.closest_intersection(mirror, &params.epsilon) {
                if params.reflection_cap.is_some_and(|n| count == n) {
                    outcome = Ok(false);
                    break;
                }
                ray.advance(dist);
                vertex_scratch.push(ray.pos.to_gl_vertex());
                ray.reflect_dir(&dir);
                count += 1;
            }

            if let Ok(true) = outcome {
                ray.advance(10000.0.as_());
                vertex_scratch.push(ray.pos.to_gl_vertex());
            }

            ray_paths.push(RayPath {
                color: params.path_color,
                non_loop_path: gl::VertexBuffer::immutable(display, &vertex_scratch).unwrap(),
                loop_path: None,
            });
        }

        mirrors.shrink_to_fit();
        ray_paths.shrink_to_fit();

        Self {
            ray_origins: gl::VertexBuffer::immutable(display, &ray_origins).unwrap(),
            ray_paths,
            mirrors,
            program,
            starting_pts_program,
            global_params,
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

    fn render_3d(&self, display: &gl::Display, camera: &Camera, projection: &Perspective3<f32>) {
        let mut target = display.draw();

        use gl::Surface;
        target.clear_color_and_depth(self.global_params.bg_color.into(), 1.0);

        let perspective: [[_; 4]; 4] = projection.into_inner().into();
        let view: [[_; 4]; 4] = camera.calc_matrix().into();

        let aspect = projection.aspect();

        let render_params = gl::DrawParameters {
            blend: gl::Blend::alpha_blending(),
            ..Default::default()
        };

        for RayPath { color, non_loop_path, loop_path } in &self.ray_paths {
            target
                .draw(
                    non_loop_path,
                    LINE_STRIP,
                    &self.program,
                    &gl::uniform! {
                        perspective: perspective,
                        view: view,
                        color_vec: *color,
                    },
                    &render_params,
                )
                .unwrap();

            if let Some((col, buf)) = loop_path {
                target
                    .draw(
                        buf,
                        NoIndices(PrimitiveType::LineLoop),
                        &self.program,
                        &gl::uniform! {
                            perspective: perspective,
                            view: view,
                            color_vec: *col,
                        },
                        &render_params,
                    )
                    .unwrap();
            }
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
                        color_vec: self.global_params.mirror_color,
                    },
                    &render_params,
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
                &render_params,
            )
            .unwrap();

        target.finish().unwrap();

        display.gl_window().window().request_redraw();
    }
}
