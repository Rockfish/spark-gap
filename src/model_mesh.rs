use std::mem;
use crate::material::Material;
use glam::u32;
use glam::*;
use std::rc::Rc;
use wgpu::util::DeviceExt;
use crate::context::Context;

const MAX_BONE_INFLUENCE: usize = 4;
const OFFSET_OF_NORMAL: usize = mem::offset_of!(ModelVertex, normal);
const OFFSET_OF_TEXCOORDS: usize = mem::offset_of!(ModelVertex, uv);
const OFFSET_OF_TANGENT: usize = mem::offset_of!(ModelVertex, tangent);
const OFFSET_OF_BITANGENT: usize = mem::offset_of!(ModelVertex, bi_tangent);
const OFFSET_OF_BONE_IDS: usize = mem::offset_of!(ModelVertex, bone_ids);
const OFFSET_OF_WEIGHTS: usize = mem::offset_of!(ModelVertex, bone_weights);

#[repr(C, packed)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ModelVertex {
    pub position: Vec3,
    pub normal: Vec3,
    pub uv: Vec2,
    pub tangent: Vec3,
    pub bi_tangent: Vec3,
    pub bone_ids: [i32; MAX_BONE_INFLUENCE],
    pub bone_weights: [f32; MAX_BONE_INFLUENCE],
}

impl ModelVertex {
    pub fn new() -> ModelVertex {
        ModelVertex {
            position: Vec3::default(),
            normal: Vec3::default(),
            uv: Vec2::default(),
            tangent: Vec3::default(),
            bi_tangent: Vec3::default(),
            bone_ids: [-1; MAX_BONE_INFLUENCE],
            bone_weights: [0.0; MAX_BONE_INFLUENCE],
        }
    }

    pub fn set_bone_data_to_default(&mut self) {
        for i in 0..MAX_BONE_INFLUENCE {
            self.bone_ids[i] = -1;
            self.bone_weights[i] = 0.0;
        }
    }

    pub fn set_bone_data(&mut self, bone_id: i32, weight: f32) {
        //set first available free spot if there is any
        for i in 0..MAX_BONE_INFLUENCE {
            if self.bone_ids[i] < 0 {
                self.bone_ids[i] = bone_id;
                self.bone_weights[i] = weight;
                break;
            }
        }
    }

    pub fn vertex_description() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<ModelVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                // position
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                // normal
                wgpu::VertexAttribute {
                    offset: OFFSET_OF_NORMAL as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
                // tex coords
                wgpu::VertexAttribute {
                    offset: OFFSET_OF_TEXCOORDS as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x2,
                },
                // tangent
                wgpu::VertexAttribute {
                    offset: OFFSET_OF_TANGENT as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x3,
                },
                // bitangent
                wgpu::VertexAttribute {
                    offset: OFFSET_OF_BITANGENT as wgpu::BufferAddress,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Float32x3,
                },
                // bone ids
                wgpu::VertexAttribute {
                    offset: OFFSET_OF_BONE_IDS as wgpu::BufferAddress,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Sint32x4,
                },
                // weights
                wgpu::VertexAttribute {
                    offset: OFFSET_OF_WEIGHTS as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

impl Default for ModelVertex {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct ModelMesh {
    pub id: i32,
    pub name: String,
    pub vertices: Vec<ModelVertex>,
    pub indices: Vec<u32>,
    pub materials: Vec<Rc<Material>>,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_elements: u32,
}

impl ModelMesh {
    pub fn new(
        context: &Context,
        id: i32,
        name: impl Into<String>,
        vertices: Vec<ModelVertex>,
        indices: Vec<u32>,
        materials: Vec<Rc<Material>>,
    ) -> ModelMesh {

        let num_elements = vertices.len() as u32;

        let vertex_data: Box<[ModelVertex]> = Box::from(vertices.as_slice());

        let vertex_buffer = context
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(&vertex_data),
                usage: wgpu::BufferUsages::VERTEX,
            });

        let index_buffer = context
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(&indices.as_slice()),
                usage: wgpu::BufferUsages::INDEX,
            });

        ModelMesh {
            id,
            name: name.into(),
            vertices,
            indices,
            materials,
            vertex_buffer,
            index_buffer,
            num_elements,
        }
    }
}
