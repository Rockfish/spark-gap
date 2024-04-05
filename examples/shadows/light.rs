use crate::render_passes::MAX_LIGHTS;
use bytemuck::{Pod, Zeroable};
use spark_gap::gpu_context::GpuContext;
use std::f32::consts;
use std::mem;
use std::ops::Range;
use wgpu::Buffer;

pub struct Light {
    pub(crate) pos: glam::Vec3,
    pub(crate) color: wgpu::Color,
    pub(crate) fov: f32,
    pub(crate) depth: Range<f32>,
    pub(crate) target_view: wgpu::TextureView,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct LightRaw {
    proj: [[f32; 4]; 4],
    pos: [f32; 4],
    color: [f32; 4],
}

impl Light {
    pub(crate) fn to_raw(&self) -> LightRaw {
        let view = glam::Mat4::look_at_rh(self.pos, glam::Vec3::ZERO, glam::Vec3::Z);
        let projection = glam::Mat4::perspective_rh(self.fov * consts::PI / 180., 1.0, self.depth.start, self.depth.end);
        let view_proj = projection * view;
        LightRaw {
            proj: view_proj.to_cols_array_2d(),
            pos: [self.pos.x, self.pos.y, self.pos.z, 1.0],
            color: [self.color.r as f32, self.color.g as f32, self.color.b as f32, 1.0],
        }
    }
}

pub fn create_light_storage_buffer(gpu_context: &mut GpuContext) -> Buffer {
    let supports_storage_resources = gpu_context
        .adapter
        .get_downlevel_capabilities()
        .flags
        .contains(wgpu::DownlevelFlags::VERTEX_STORAGE)
        && gpu_context.device.limits().max_storage_buffers_per_shader_stage > 0;

    let light_uniform_size = (MAX_LIGHTS * mem::size_of::<LightRaw>()) as wgpu::BufferAddress;

    let light_storage_buf = gpu_context.device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: light_uniform_size,
        usage: if supports_storage_resources {
            wgpu::BufferUsages::STORAGE
        } else {
            wgpu::BufferUsages::UNIFORM
        } | wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    light_storage_buf
}
