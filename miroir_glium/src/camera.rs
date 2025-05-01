use super::*;

use core::{f32::consts::FRAC_PI_2, time::Duration};
use glium::glutin::event::{ElementState, VirtualKeyCode};
use na::{Matrix4, Point3, Vector3};

const SAFE_FRAC_PI_2: f32 = FRAC_PI_2 - 0.0001;

pub struct Camera {
    pos: Point3<f32>,
    yaw: f32,
    pitch: f32,
}

impl Camera {
    pub fn new(position: impl Into<Point3<f32>>, yaw: f32, pitch: f32) -> Self {
        Self {
            pos: position.into(),
            yaw,
            pitch,
        }
    }

    pub fn calc_matrix(&self) -> Matrix4<f32> {
        let (sin_pitch, cos_pitch) = self.pitch.sin_cos();
        let (sin_yaw, cos_yaw) = self.yaw.sin_cos();

        let up = Vector3::y();
        let target = Vector3::new(cos_pitch * cos_yaw, sin_pitch, cos_pitch * sin_yaw);

        Matrix4::look_at_rh(&self.pos, &(self.pos + target), &up)
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
    speed: f32,
    mouse_sensitivity: f32,
}

impl CameraController {
    pub const fn new(speed: f32, mouse_sensitivity: f32) -> Self {
        Self {
            amount_left: 0.,
            amount_right: 0.,
            amount_forward: 0.,
            amount_backwards: 0.,
            amount_up: 0.,
            amount_down: 0.,
            rotate_horizontal: 0.,
            rotate_vertical: 0.,
            speed,
            mouse_sensitivity,
        }
    }

    pub fn process_keyboard(&mut self, key: VirtualKeyCode, state: ElementState) -> bool {
        const RATIO: f32 = 0.8;

        use VirtualKeyCode::*;

        let amount = (state == ElementState::Pressed) as u32 as f32;

        let mut res = true;

        match key {
            Z | W => self.amount_forward = amount,
            S => self.amount_backwards = amount,
            Q | A => self.amount_left = amount,
            D => self.amount_right = amount,
            Space => self.amount_up = amount,
            LShift => self.amount_down = amount,
            Up => self.speed /= RATIO,
            Down => self.speed *= RATIO,
            Right => self.mouse_sensitivity /= RATIO,
            Left => self.mouse_sensitivity *= RATIO,
            _ => res = false,
        }

        res
    }

    pub fn set_mouse_delta(&mut self, mouse_dx: f64, mouse_dy: f64) {
        self.rotate_horizontal = mouse_dx as f32;
        self.rotate_vertical = -mouse_dy as f32;
    }

    pub fn update_camera(&mut self, camera: &mut Camera, dt: Duration) {
        let dt = dt.as_secs_f32();

        let (yaw_sin, yaw_cos) = camera.yaw.sin_cos();

        let forward = Vector3::new(yaw_cos, 0., yaw_sin);
        let right = Vector3::new(-yaw_sin, 0., yaw_cos);

        let spd = self.speed * dt;
        let mouse_sens = self.mouse_sensitivity * dt;

        camera.pos += forward * (self.amount_forward - self.amount_backwards) * spd;
        camera.pos += right * (self.amount_right - self.amount_left) * spd;
        camera.pos.y += (self.amount_up - self.amount_down) * spd;

        camera.yaw += self.rotate_horizontal * mouse_sens;
        camera.pitch += self.rotate_vertical * mouse_sens;

        camera.pitch = camera.pitch.clamp(-SAFE_FRAC_PI_2, SAFE_FRAC_PI_2);

        self.rotate_horizontal = 0.;
        self.rotate_vertical = 0.;
    }
}
