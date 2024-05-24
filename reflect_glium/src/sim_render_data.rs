use super::*;

use gl::index::{NoIndices, PrimitiveType};
const LINE_STRIP: NoIndices = NoIndices(PrimitiveType::LineStrip);

pub(crate) struct SimRenderData<const D: usize> {
    ray_render_data: Vec<RayRenderData<D>>,
    mirror_render_data: Vec<Box<dyn RenderData>>,
    program: gl::Program,
}

impl<const D: usize> SimRenderData<D>
where
    Vertex<D>: gl::Vertex,
{
    fn new(
        ray_render_data: Vec<RayRenderData<D>>,
        mirror_render_data: Vec<Box<dyn RenderData>>,
        program: gl::Program,
    ) -> Self {
        Self {
            ray_render_data,
            mirror_render_data,
            program,
        }
    }
}

const FRAGMENT_SHADER_SRC: &str = r#"
    #version 140

    uniform vec4 color_vec;

    out vec4 color;

    void main() {
        color = color_vec;
    }
"#;

impl SimRenderData<3> {
    pub(crate) fn from_simulation<T: reflect::mirror::Mirror<3> + OpenGLRenderable>(
        sim: &reflect::Simulation<T, 3>,
        reflection_limit: usize,
        display: &gl::Display,
    ) -> Self {
        const VERTEX_SHADER_SRC_3D: &str = r#"
            #version 140

            in vec3 position;
            uniform mat4 perspective;
            uniform mat4 view;

            void main() {
                gl_Position = perspective * view * vec4(position, 1.0);
            }
        "#;

        let program =
            gl::Program::from_source(display, VERTEX_SHADER_SRC_3D, FRAGMENT_SHADER_SRC, None)
                .unwrap();

        let mut render_data = vec![];

        sim.mirror
            .append_render_data(display, reflect::util::List::from(&mut render_data));

        Self::new(
            RayRenderData::<3>::from_simulation(sim, reflection_limit, display),
            render_data,
            program,
        )
    }
}

impl SimRenderData<2> {
    pub(crate) fn from_simulation<T: reflect::mirror::Mirror<2> + OpenGLRenderable>(
        sim: &reflect::Simulation<T, 2>,
        reflection_limit: usize,
        display: &gl::Display,
    ) -> Self {
        const VERTEX_SHADER_SRC_2D: &str = r#"
            #version 140

            in vec2 position;
            uniform mat4 perspective;
            uniform mat4 view;

            void main() {
                gl_Position = perspective * view * vec4(position, 0.0, 1.0);
            }
        "#;

        let program =
            gl::Program::from_source(display, VERTEX_SHADER_SRC_2D, FRAGMENT_SHADER_SRC, None)
                .unwrap();

        let mut render_data = vec![];

        sim.mirror
            .append_render_data(display, reflect::util::List::from(&mut render_data));

        Self::new(
            RayRenderData::<2>::from_simulation(sim, reflection_limit, display),
            render_data,
            program,
        )
    }
}

impl<const D: usize> SimRenderData<D>
where
    Vertex<D>: gl::Vertex,
{
    pub(crate) fn run(self, display: gl::Display, events_loop: glutin::event_loop::EventLoop<()>) {
        const DEFAULT_CAMERA_POS: cg::Point3<f32> = cg::Point3::new(0., 0., 5.);
        const DEFAULT_CAMERA_YAW: cg::Deg<f32> = cg::Deg(-90.);
        const DEFAULT_CAMERA_PITCH: cg::Deg<f32> = cg::Deg(0.);

        let mut camera = Camera::new(DEFAULT_CAMERA_POS, DEFAULT_CAMERA_YAW, DEFAULT_CAMERA_PITCH);

        const DEFAULT_PROJECCTION_POV: cg::Deg<f32> = cg::Deg(85.);
        const NEAR_PLANE: f32 = 0.0001;
        const FAR_PLANE: f32 = 10000.;

        use glutin::{dpi, event, event_loop, window};
        let dpi::PhysicalSize { width, height } = display.gl_window().window().inner_size();

        let mut projection = Projection::new(
            width,
            height,
            DEFAULT_PROJECCTION_POV,
            NEAR_PLANE,
            FAR_PLANE,
        );

        const SPEED: f32 = 5.;
        const MOUSE_SENSITIVITY: f32 = 1.0;

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

                    display.gl_window().resize(physical_size)
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
        const ORIGIN_COLOR: [f32; 4] = [1.0, 0.0, 0.0, 1.0];
        const RAY_NON_LOOP_COL: [f32; 4] = [0.7, 0.3, 0.1, 1.0];
        const RAY_LOOP_COL: [f32; 4] = [1.0, 0.0, 1.0, 1.0];
        let mirror_color = if D >= 3 {
            [0.3f32, 0.3, 0.9, 0.4]
        } else {
            [0.15, 0.15, 0.5, 1.0]
        };

        let mut target = display.draw();

        use gl::Surface;
        target.clear_color_and_depth((1., 0.95, 0.7, 1.), 1.0);

        let perspective = projection.get_matrix();
        let view = camera.calc_matrix();

        let params = gl::DrawParameters {
            depth: gl::Depth {
                test: gl::draw_parameters::DepthTest::Overwrite,
                write: false,
                ..Default::default()
            },
            line_width: Some(1.0),
            multisampling: true,
            blend: gl::Blend::alpha_blending(),
            ..Default::default()
        };

        for ray in &self.ray_render_data {
            target
                .draw(
                    &ray.non_loop_path,
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
                    &ray.loop_path,
                    LINE_STRIP,
                    &self.program,
                    &gl::uniform! {
                        perspective: perspective,
                        view: view,
                        color_vec: RAY_LOOP_COL,
                    },
                    &params,
                )
                .unwrap();

            let o = &ray.origin;
            target
                .draw(
                    o.vertices(),
                    o.indices(),
                    &self.program,
                    &gl::uniform! {
                        perspective: perspective,
                        view: view,
                        color_vec: ORIGIN_COLOR,
                    },
                    &params,
                )
                .unwrap();
        }

        for render_data in self.mirror_render_data.iter().map(Box::as_ref) {
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

        target.finish().unwrap();

        display.gl_window().window().request_redraw();
    }
}
