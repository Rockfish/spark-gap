use glam::{Mat4, Vec3};
use wgpu::TextureView;
use spark_gap::camera::CameraController;
use spark_gap::camera_handler::CameraHandler;
use spark_gap::model::Model;


pub struct World {
    pub camera_controller: CameraController,
    pub camera_handler: CameraHandler,
    pub model: Model,
    pub model_position: Vec3,
    pub model_transform: Mat4,
    pub depth_texture_view: TextureView,
    pub run: bool,
    pub delta_time: f32,
    pub frame_time: f32,
    pub first_mouse: bool,
    pub mouse_x: f32,
    pub mouse_y: f32,
}