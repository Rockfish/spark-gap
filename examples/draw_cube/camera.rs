use std::f32::consts;
use glam::{Mat4, Vec3};
use wgpu::{BindGroup, BindGroupLayout, Buffer};
use wgpu::util::DeviceExt;
use crate::context::Context;

pub struct Camera {
    pub perspective_view: Mat4,
}

pub struct CameraHandler {
    pub camera_buffer: Buffer,
    pub bind_group: BindGroup,
    pub bind_group_layout: BindGroupLayout,
}

impl Camera {
    pub fn new() -> Camera {
        Camera {
            perspective_view: Self::get_projection_view_matrix(),
        }
    }

    pub fn get_projection_view_matrix() -> Mat4 {
        let aspect_ratio = 1.0;
        let projection = Mat4::perspective_rh(consts::FRAC_PI_4, aspect_ratio, 1.0, 10.0);
        let view = Mat4::look_at_rh(
            Vec3::new(1.5f32, -5.0, 3.0),
            Vec3::ZERO,
            Vec3::Z,
        );
        projection * view
    }

    pub fn get_camera_uniform(&self) -> [f32; 16] {
        self.perspective_view.to_cols_array()
    }
}

impl CameraHandler {
    pub fn new(context: &Context, camera: &Camera) -> Self {
        let camera_uniform = camera.get_camera_uniform();

        let camera_buffer = context.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Camera Buffer"),
                contents: bytemuck::cast_slice(&[camera_uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        // looks like this should really be the "vertex shader bind group"
        let bind_group_layout =
            context.device.create_bind_group_layout(
                &wgpu::BindGroupLayoutDescriptor {
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::VERTEX,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                    ],
                    label: Some("camera_bind_group_layout"),
                });

        let bind_group = context.device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                layout: &bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: camera_buffer.as_entire_binding(),
                    },
                ],
                label: Some("camera_bind_group"),
            });

        Self {
            camera_buffer,
            bind_group,
            bind_group_layout,
        }
    }

    pub fn update_camera_buffer(&self, context: &Context, camera: &Camera) {
        let camera_uniform = camera.get_camera_uniform();
        context.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[camera_uniform])
        );
    }
}