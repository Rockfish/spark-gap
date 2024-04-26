use std::f32::consts;
use std::mem;
use std::ops::Range;

use bytemuck::{Pod, Zeroable};
use glam::Mat4;
use wgpu::{Buffer, Texture, TextureView};

use spark_gap::gpu_context::GpuContext;

pub const MAX_LIGHTS: usize = 10;

pub struct Lights {
    pub lights: Vec<Light>,
    pub light_storage_buffer: Buffer,
    pub lights_are_dirty: bool,
}

pub struct Light {
    pub position: glam::Vec3,
    pub color: wgpu::Color,
    pub fov: f32,
    pub depth: Range<f32>,
    pub projection_view: Mat4,
    pub shadow_view: TextureView,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct LightUniform {
    projection: [[f32; 4]; 4],
    position: [f32; 4],
    color: [f32; 4],
}

impl Lights {
    pub fn new(gpu_context: &mut GpuContext, shadow_texture_array: &Texture) -> Self {
        let lights = vec![
            Light {
                position: glam::Vec3::new(7.0, -5.0, 10.0),
                // position: glam::Vec3::new(7.0, 10.0, -5.0),
                color: wgpu::Color {
                    r: 0.5,
                    g: 1.0,
                    b: 0.5,
                    a: 1.0,
                },
                fov: 60.0,
                depth: 1.0..1000.0,
                projection_view: Mat4::IDENTITY,
                shadow_view: create_shadow_texture_view(shadow_texture_array, 0),
            },
            Light {
                position: glam::Vec3::new(-5.0, 7.0, 10.0),
                // position: glam::Vec3::new(-5.0, 10.0, 7.0),
                color: wgpu::Color {
                    r: 1.0,
                    g: 0.5,
                    b: 0.5,
                    a: 1.0,
                },
                fov: 45.0,
                depth: 1.0..1000.0,
                projection_view: Mat4::IDENTITY,
                shadow_view: create_shadow_texture_view(shadow_texture_array, 1),
            },
        ];

        let light_storage_buffer = create_light_storage_buffer(gpu_context);

        Lights {
            lights,
            light_storage_buffer,
            lights_are_dirty: true,
        }
    }

    pub fn update(&mut self, context: &GpuContext) {
        if self.lights_are_dirty {
            self.lights_are_dirty = false;

            for (i, light) in self.lights.iter_mut().enumerate() {
                let view = Mat4::look_at_rh(light.position, glam::Vec3::ZERO, glam::Vec3::Z);
                let projection = Mat4::perspective_rh(light.fov * consts::PI / 180., 1.0, light.depth.start, light.depth.end);
                light.projection_view = projection * view;

                let light_uniform = LightUniform {
                    projection: light.projection_view.to_cols_array_2d(),
                    position: [light.position.x, light.position.y, light.position.z, 1.0],
                    color: [light.color.r as f32, light.color.g as f32, light.color.b as f32, 1.0],
                };

                context.queue.write_buffer(
                    &self.light_storage_buffer,
                    (i * mem::size_of::<LightUniform>()) as wgpu::BufferAddress,
                    bytemuck::bytes_of(&light_uniform),
                );
            }
        }
    }
}

pub fn create_light_storage_buffer(gpu_context: &mut GpuContext) -> Buffer {
    let light_uniform_size = (MAX_LIGHTS * mem::size_of::<LightUniform>()) as wgpu::BufferAddress;

    let light_storage_buf = gpu_context.device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: light_uniform_size,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    light_storage_buf
}

fn create_shadow_texture_view(shadow_texture: &Texture, layer_id: u32) -> TextureView {
    shadow_texture.create_view(&wgpu::TextureViewDescriptor {
        label: Some(&format!("shadow id: {}", layer_id)),
        format: None,
        dimension: Some(wgpu::TextureViewDimension::D2),
        aspect: wgpu::TextureAspect::All,
        base_mip_level: 0,
        mip_level_count: None,
        base_array_layer: layer_id,
        array_layer_count: Some(1),
    })
}
