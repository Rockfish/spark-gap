use std::f32::consts;
use glam::{Mat4, Vec3};
use wgpu::{BindGroup, BindGroupLayout, Buffer};
use wgpu::util::DeviceExt;
use winit::dpi::PhysicalSize;
use crate::context::Context;

pub struct Camera {
    pub window_size: PhysicalSize<u32>,
    pub perspective_view: Mat4,
}

pub struct CameraHandler {
    pub camera_buffer: Buffer,
    pub bind_group: BindGroup,
    pub bind_group_layout: BindGroupLayout,
}

impl Camera {
    pub fn new(context: &Context) -> Camera {
        let size = context.window.inner_size();

        Camera {
            window_size: size,
            perspective_view: Self::get_projection_view_matrix(size),
        }
    }

    pub fn get_projection_view_matrix(size: PhysicalSize<u32>) -> Mat4 {
        let aspect_ratio = size.width as f32 / size.height as f32;
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

    pub fn update(&mut self, context: &Context) {
        let size = context.window.inner_size();
        if self.window_size != size {
           self.window_size = size;
            self.perspective_view = Self::get_projection_view_matrix(size);
        }
    }
}

impl CameraHandler {
    pub fn new(context: &Context, camera: &Camera) -> Self {
        // let camera_uniform = camera.get_camera_uniform();
        let camera_uniform = camera.perspective_view.to_cols_array();

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

    pub fn update_camera(&self, context: &Context, camera: &mut Camera) {

        camera.update(context);

        let camera_uniform = camera.get_camera_uniform();
        context.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[camera_uniform])
        );
    }
}