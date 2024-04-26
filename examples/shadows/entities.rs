use std::f32::consts;
use std::mem;
use std::sync::Arc;

use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Quat, Vec3};
use wgpu::util::{align_to, DeviceExt};
use wgpu::{BindGroup, BindGroupLayout, Buffer};

use spark_gap::gpu_context::GpuContext;

use crate::cube::{create_cube, create_plane};

pub struct Entity {
    pub mx_world: Mat4,
    pub rotation_speed: f32,
    pub color: wgpu::Color,
    pub vertex_buf: Arc<Buffer>,
    pub index_buf: Arc<Buffer>,
    pub index_format: wgpu::IndexFormat,
    pub index_count: usize,
    pub uniform_offset: wgpu::DynamicOffset,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct EntityUniform {
    pub model: [[f32; 4]; 4],
    pub color: [f32; 4],
}

pub struct CubeDesc {
    offset: Vec3,
    angle: f32,
    scale: f32,
    rotation: f32,
}

pub struct Entities {
    pub entity_uniform_buf: Buffer,
    pub entities: Vec<Entity>,
    pub entity_bind_group_layout: BindGroupLayout,
    pub entity_bind_group: BindGroup,
}

impl Entities {
    pub fn new(gpu_context: &mut GpuContext) -> Self {
        let (plane_vertex_data, plane_index_data) = create_plane(7);

        let plane_vertex_buf = gpu_context.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Plane Vertex Buffer"),
            contents: bytemuck::cast_slice(&plane_vertex_data),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let plane_index_buf = gpu_context.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Plane Index Buffer"),
            contents: bytemuck::cast_slice(&plane_index_data),
            usage: wgpu::BufferUsages::INDEX,
        });

        let (cube_vertex_data, cube_index_data) = create_cube();

        let cube_vertex_buf = Arc::new(gpu_context.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Cubes Vertex Buffer"),
            contents: bytemuck::cast_slice(&cube_vertex_data),
            usage: wgpu::BufferUsages::VERTEX,
        }));

        let cube_index_buf = Arc::new(gpu_context.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Cubes Index Buffer"),
            contents: bytemuck::cast_slice(&cube_index_data),
            usage: wgpu::BufferUsages::INDEX,
        }));

        let entity_uniform_size = mem::size_of::<EntityUniform>() as wgpu::BufferAddress;

        let entity_bind_group_layout = gpu_context.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: true,
                    min_binding_size: wgpu::BufferSize::new(entity_uniform_size),
                },
                count: None,
            }],
            label: None,
        });

        let cube_descriptions = get_cube_descriptions();

        let num_entities = 1 + cube_descriptions.len() as wgpu::BufferAddress;

        // Make the `uniform_alignment` >= `entity_uniform_size` and aligned to `min_uniform_buffer_offset_alignment`.
        let uniform_alignment = {
            let alignment = gpu_context.device.limits().min_uniform_buffer_offset_alignment as wgpu::BufferAddress;
            align_to(entity_uniform_size, alignment)
        };

        // Note: dynamic uniform offsets also have to be aligned to `Limits::min_uniform_buffer_offset_alignment`.
        let entity_uniform_buf = gpu_context.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: num_entities * uniform_alignment,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let entity_bind_group = gpu_context.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &entity_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &entity_uniform_buf,
                    offset: 0,
                    size: wgpu::BufferSize::new(entity_uniform_size),
                }),
            }],
            label: None,
        });

        let index_format = wgpu::IndexFormat::Uint16;

        let mut entities = vec![{
            Entity {
                mx_world: Mat4::IDENTITY,
                rotation_speed: 0.0,
                color: wgpu::Color::WHITE,
                vertex_buf: Arc::new(plane_vertex_buf),
                index_buf: Arc::new(plane_index_buf),
                index_format,
                index_count: plane_index_data.len(),
                uniform_offset: 0,
            }
        }];

        for (i, cube) in cube_descriptions.iter().enumerate() {

            // todo: temp for just one cube
            if i > 0 { break; }

            let mx_world = Mat4::from_scale_rotation_translation(
                Vec3::splat(cube.scale),
                Quat::from_axis_angle(cube.offset.normalize(), cube.angle * consts::PI / 180.),
                cube.offset,
            );

            entities.push(Entity {
                mx_world,
                rotation_speed: cube.rotation,
                color: wgpu::Color::GREEN,
                vertex_buf: Arc::clone(&cube_vertex_buf),
                index_buf: Arc::clone(&cube_index_buf),
                index_format,
                index_count: cube_index_data.len(),
                uniform_offset: ((i + 1) * uniform_alignment as usize) as _,
            });
        }

        Entities {
            entity_uniform_buf,
            entities,
            entity_bind_group_layout,
            entity_bind_group,
        }
    }

    pub fn update(&mut self, context: &GpuContext) {
        // update uniforms
        for entity in self.entities.iter_mut() {
            if entity.rotation_speed != 0.0 {
                let rotation = Mat4::from_rotation_x(entity.rotation_speed * consts::PI / 180.);
                entity.mx_world *= rotation;
            }
            let data = EntityUniform {
                model: entity.mx_world.to_cols_array_2d(),
                color: [
                    entity.color.r as f32,
                    entity.color.g as f32,
                    entity.color.b as f32,
                    entity.color.a as f32,
                ],
            };
            context.queue.write_buffer(
                &self.entity_uniform_buf,
                entity.uniform_offset as wgpu::BufferAddress,
                bytemuck::bytes_of(&data),
            );
        }
    }
}

pub fn get_cube_descriptions() -> [CubeDesc; 4] {
    let cube_descriptions = [
        CubeDesc {
            offset: Vec3::new(-2.0, -2.0, 2.0),
            angle: 10.0,
            scale: 0.7,
            rotation: 0.1,
        },
        CubeDesc {
            offset: Vec3::new(2.0, -2.0, 2.0),
            angle: 50.0,
            scale: 1.3,
            rotation: 0.2,
        },
        CubeDesc {
            offset: Vec3::new(-2.0, 2.0, 2.0),
            angle: 140.0,
            scale: 1.1,
            rotation: 0.3,
        },
        CubeDesc {
            offset: Vec3::new(2.0, 2.0, 2.0),
            angle: 210.0,
            scale: 0.9,
            rotation: 0.4,
        },
    ];

    cube_descriptions
}
