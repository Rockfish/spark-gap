use crate::camera::CameraController;
use crate::context::Context;
use wgpu::util::DeviceExt;
use wgpu::{BindGroup, BindGroupLayout, Buffer};

pub const CAMERA_BIND_GROUP_LAYOUT: &str = "camera_bind_group_layout";

pub struct CameraHandler {
    pub camera_buffer: Buffer,
    pub bind_group: BindGroup,
}

impl CameraHandler {
    pub fn new(context: &mut Context, camera_controller: &CameraController) -> Self {

        let camera_uniform = camera_controller.get_camera_uniform();

        let camera_buffer = context
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Camera Buffer"),
                contents: bytemuck::cast_slice(&[camera_uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        if !context.bind_layout_cache.contains_key(CAMERA_BIND_GROUP_LAYOUT) {
            let layout = create_camera_bind_group_layout(context);
            context.bind_layout_cache.insert(String::from(CAMERA_BIND_GROUP_LAYOUT), layout);
        }

        let bind_group_layout = context.bind_layout_cache.get(CAMERA_BIND_GROUP_LAYOUT).unwrap();

        let bind_group = context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                }],
                label: Some("camera_bind_group"),
            });

        Self {
            camera_buffer,
            bind_group,
        }
    }

    pub fn update_camera(&self, context: &Context, camera_controller: &CameraController) {
        let camera_uniform = camera_controller.get_camera_uniform();
        context.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[camera_uniform]),
        );
    }
}

fn create_camera_bind_group_layout(context: &Context) -> BindGroupLayout {
    context
        .device
        .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some(CAMERA_BIND_GROUP_LAYOUT),
        })
}
