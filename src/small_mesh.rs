use std::rc::Rc;

use wgpu::Buffer;
use wgpu::util::DeviceExt;
use crate::gpu_context::GpuContext;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SmallMeshVertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
}

#[derive(Debug, Clone)]
pub struct SmallMesh {
    pub vertex_buffer: Rc<Buffer>,
    pub num_elements: u32,
}

impl SmallMesh {
    pub fn vertex_description() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<SmallMeshVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                // vertices
                wgpu::VertexAttribute {
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 0,
                },
                // tex coords
                wgpu::VertexAttribute {
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                },
            ],
        }
    }
}

#[rustfmt::skip]
const UNIT_SQUARE: [f32; 30] = [
    -1.0, -1.0, 0.0, 0.0, 0.0,
     1.0, -1.0, 0.0, 1.0, 0.0,
     1.0,  1.0, 0.0, 1.0, 1.0,
    -1.0, -1.0, 0.0, 0.0, 0.0,
     1.0,  1.0, 0.0, 1.0, 1.0,
    -1.0,  1.0, 0.0, 0.0, 1.0,
];

pub fn create_unit_square(context: &mut GpuContext) -> SmallMesh {
    let vertex_buffer = context.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Vertex Buffer"),
        contents: bytemuck::cast_slice(&UNIT_SQUARE),
        usage: wgpu::BufferUsages::VERTEX,
    });

    SmallMesh {
        vertex_buffer: vertex_buffer.into(),
        num_elements: 6,
    }
}