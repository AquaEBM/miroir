use core::{f32::consts::FRAC_PI_2, time::Duration};

use cgmath::{InnerSpace, Matrix4, Point3, Rad, Vector3};
use glium::glutin::{
    dpi::PhysicalPosition,
    event::{ElementState, MouseScrollDelta, VirtualKeyCode},
};

const SAFE_FRAC_PI_2: f32 = FRAC_PI_2 - 0.0001;

pub struct Camera {
    pub position: Point3<f32>,
    yaw: Rad<f32>,
    pitch: Rad<f32>,
}

impl Camera {
    pub fn new<V: Into<cgmath::Point3<f32>>, Y: Into<Rad<f32>>, P: Into<Rad<f32>>>(
        position: V,
        yaw: Y,
        pitch: P,
    ) -> Self {
        Self {
            position: position.into(),
            yaw: yaw.into(),
            pitch: pitch.into(),
        }
    }

    pub fn calc_matrix(&self) -> [[f32; 4]; 4] {
        let (sin_pitch, cos_pitch) = self.pitch.0.sin_cos();
        let (sin_yaw, cos_yaw) = self.yaw.0.sin_cos();

        let up = cgmath::Vector3::unit_y();
        let target =
            cgmath::Vector3::new(cos_pitch * cos_yaw, sin_pitch, cos_pitch * sin_yaw);

        Matrix4::look_to_rh(
            cgmath::Point3::new(self.position.x, self.position.y, self.position.z),
            target,
            up,
        ).into()
    }
}

#[derive(Debug)]
pub struct Projection {
    aspect: f32,
    fov_y: Rad<f32>,
    z_near: f32,
    z_far: f32,
}

impl Projection {
    pub fn new<F: Into<Rad<f32>>>(
        width: u32,
        height: u32,
        fov_y: F,
        z_near: f32,
        z_far: f32,
    ) -> Self {
        Self {
            aspect: width as f32 / height as f32,
            fov_y: fov_y.into(),
            z_near,
            z_far,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.aspect = width as f32 / height as f32;
    }
    
    pub fn get_matrix(&self) -> [[f32; 4]; 4] {
        cgmath::perspective(cgmath::Deg(45.), self.aspect, self.z_near, self.z_far).into()
    }
}

#[derive(Debug)]
pub struct CameraController {
    amount_left: f32,
    amount_right: f32,
    amount_forward: f32,
    amount_backwards: f32,
    amount_up: f32,
    amount_down: f32,
    rotate_horizontal: f32,
    rotate_vertical: f32,
    scroll: f32,
    speed: f32,
    movement_sensitivity: f32,
    mouse_sensitivity: f32,
}

impl CameraController {
    pub fn new(speed: f32, movement_sensitivity: f32, mouse_sensitivity: f32) -> Self {
        Self {
            amount_left: 0.,
            amount_right: 0.,
            amount_forward: 0.,
            amount_backwards: 0.,
            amount_up: 0.,
            amount_down: 0.,
            rotate_horizontal: 0.,
            rotate_vertical: 0.,
            scroll: 0.,
            speed,
            movement_sensitivity,
            mouse_sensitivity,
        }
    }

    pub fn process_keyboard(&mut self, key: VirtualKeyCode, state: ElementState) -> bool {
        let amount = (state == ElementState::Pressed) as u32 as f32;

        let mut res = true;

        match key {
            VirtualKeyCode::Z | VirtualKeyCode::W => self.amount_forward = amount,
            VirtualKeyCode::S => self.amount_backwards = amount,
            VirtualKeyCode::Q | VirtualKeyCode::A => self.amount_left = amount,
            VirtualKeyCode::D => self.amount_right = amount,
            VirtualKeyCode::Space => self.amount_up = amount,
            VirtualKeyCode::LShift => self.amount_down = amount,
            _ => res = false,
        }

        res
    }

    pub fn process_mouse(&mut self, mouse_dx: f64, mouse_dy: f64) {
        self.rotate_horizontal = mouse_dx as f32;
        self.rotate_vertical = mouse_dy as f32;
    }

    pub fn process_scroll(&mut self, delta: &MouseScrollDelta) {
        self.scroll = -match delta {
            MouseScrollDelta::LineDelta(_, scroll) => scroll * 100.,
            MouseScrollDelta::PixelDelta(PhysicalPosition { y: scroll, .. }) => *scroll as f32,
        }
    }

    pub fn update_camera(&mut self, camera: &mut Camera, dt: Duration) {
        let dt = dt.as_secs_f32();

        let (yaw_sin, yaw_cos) = camera.yaw.0.sin_cos();
        let forward = Vector3::new(yaw_cos, 0., yaw_sin).normalize();
        let right = Vector3::new(-yaw_sin, 0., yaw_cos).normalize();

        let (pitch_sin, pitch_cos) = camera.pitch.0.sin_cos();
        let scrollward = Vector3::new(pitch_cos * yaw_cos, pitch_sin, pitch_cos * yaw_sin).normalize();

        let spd = self.speed * dt;
        let move_sens = self.movement_sensitivity * dt;
        let mouse_sens = self.mouse_sensitivity * dt;

        camera.position += forward * (self.amount_forward - self.amount_backwards) * spd;
        camera.position += right * (self.amount_right - self.amount_left) * spd;

        camera.position += scrollward * self.scroll * self.speed * move_sens;

        camera.position.y += (self.amount_up - self.amount_down) * spd;

        camera.yaw += Rad(self.rotate_horizontal) * mouse_sens;
        camera.pitch += Rad(-self.rotate_vertical) * mouse_sens;

        self.scroll = 0.;
        self.rotate_horizontal = 0.;
        self.rotate_vertical = 0.;

        camera.pitch = Rad(camera.pitch.0.clamp(-SAFE_FRAC_PI_2, SAFE_FRAC_PI_2));
    }
}
