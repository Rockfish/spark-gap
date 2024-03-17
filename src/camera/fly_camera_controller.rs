use crate::camera::camera_handler::CameraUniform;
use crate::gpu_context::GpuContext;
use crate::input::Input;
use glam::{Mat4, Quat, Vec3};
use winit::event::MouseButton;
use winit::keyboard::KeyCode;

#[derive(Debug, Clone, Copy)]
pub struct FlyCameraController {
    pub speed: f32,
    pub sensitivity: f32,
    pub position: Vec3,
    pub rotation: Quat,
    pub fov: f32,
    pub aspect_ratio: f32,
    pub near: f32,
    pub far: f32,
}

impl FlyCameraController {
    pub fn new(aspect: f32, position: Vec3, yaw: f32, pitch: f32) -> FlyCameraController {
        Self {
            speed: 80.0,
            sensitivity: 0.001,
            position,
            rotation: Self::get_rotation(yaw, pitch),
            fov: 60.0f32.to_radians(),
            aspect_ratio: aspect,
            near: 0.1,
            far: 1000.0, //100.0,
        }
    }

    pub fn update(&mut self, input: &Input, delta_time: f32) {
        if input.mouse_button_pressed(MouseButton::Left) {
            let mouse_delta = input.mouse_delta();

            let (mut yaw, mut pitch, _roll) = self.rotation.to_euler(glam::EulerRot::YXZ);

            let forward = self.rotation * Vec3::NEG_Z;
            let right = self.rotation * Vec3::X;

            let mut velocity = Vec3::ZERO;

            if input.key_pressed(KeyCode::KeyW) {
                velocity += forward;
            }
            if input.key_pressed(KeyCode::KeyS) {
                velocity -= forward;
            }
            if input.key_pressed(KeyCode::KeyD) {
                velocity += right;
            }
            if input.key_pressed(KeyCode::KeyA) {
                velocity -= right;
            }
            if input.key_pressed(KeyCode::Space) {
                velocity += Vec3::Y;
            }
            if input.key_pressed(KeyCode::ControlLeft) {
                velocity -= Vec3::Y;
            }

            velocity = velocity.normalize_or_zero() * self.speed * delta_time;

            if input.key_pressed(KeyCode::ShiftLeft) {
                velocity *= 2.0;
            }

            self.position += velocity;

            yaw += -(mouse_delta.x * self.sensitivity).to_radians();
            pitch += -(mouse_delta.y * self.sensitivity).to_radians();

            pitch = pitch.clamp(-std::f32::consts::FRAC_PI_2 + 0.001, std::f32::consts::FRAC_PI_2 - 0.001);

            self.rotation = Quat::from_axis_angle(Vec3::Y, yaw) * Quat::from_axis_angle(Vec3::X, pitch);
            self.rotation = self.rotation.normalize();
        }

        // self.aspect_ratio = aspect;
        // camera.view_matrix = self.view_matrix();
        // camera.projection_matrix = self.projection_matrix();
    }

    fn get_rotation(yaw: f32, pitch: f32) -> Quat {
        let up = Vec3::Y; // yaw_axis
        let right = Vec3::X; // pitch_axis
        let rotation = Quat::from_axis_angle(up, yaw.to_radians()) * Quat::from_axis_angle(right, pitch.to_radians());
        rotation.normalize()
    }

    pub fn get_projection_matrix(&self) -> Mat4 {
        Mat4::perspective_rh(self.fov, self.aspect_ratio, self.near, self.far)
    }

    pub fn get_view_matrix(&self) -> Mat4 {
        // println!("roation: {:?}  position: {:?}", &self.rotation, &self.position);
        Mat4::from_rotation_translation(self.rotation, self.position).inverse()
    }

    pub fn get_lookat_view_matrix(&self, target: Vec3) -> Mat4 {
        Mat4::look_at_rh(self.position, target, Vec3::Y)
    }

    pub fn get_camera_uniform(&self) -> CameraUniform {
        CameraUniform {
            projection: self.get_projection_matrix(),
            view: self.get_view_matrix(),
            position: self.position,
            _padding: 0,
        }
    }

    pub fn resize(&mut self, context: &GpuContext) {
        let size = context.window.inner_size();
        self.aspect_ratio = size.width as f32 / size.height as f32;
    }
}
