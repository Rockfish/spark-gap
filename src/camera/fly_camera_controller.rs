use crate::camera::camera_handler::CameraUniform;
use crate::context::Context;
use glam::{Mat4, Quat, Vec3};

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
            speed: 10.0,
            sensitivity: 0.1,
            position,
            rotation: Self::get_rotation(yaw, pitch),
            fov: 60.0f32.to_radians(),
            aspect_ratio: aspect,
            near: 0.1,
            far: 1000.0, //100.0,
        }
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

    pub fn resize(&mut self, context: &Context) {
        let size = context.window.inner_size();
        self.aspect_ratio = size.width as f32 / size.height as f32;
    }
}
