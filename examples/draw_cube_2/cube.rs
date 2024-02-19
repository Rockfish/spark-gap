use glam::{vec2, vec3};
use spark_gap::model_mesh::ModelVertex;

pub struct Cube {
    pub vertex_data: Vec<ModelVertex>,
    pub num_elements: u32,
}

impl Cube {
    pub fn new() -> Self {
        let vertex_data = Self::get_cube_vertices();

        let mut vertices: Vec<ModelVertex> = vec![];

        for [x, y, z, u, v] in vertex_data.iter().array_chunks() {
            let vertex = ModelVertex {
                position: vec3(*x, *y, *z),
                normal: Default::default(),
                uv: vec2(*u, *v),
                tangent: Default::default(),
                bi_tangent: Default::default(),
                bone_ids: [-1, -1, -1, -1],
                bone_weights: [0.0, 0.0, 0.0, 0.0],
            };

            vertices.push(vertex);
        }

        let num_elements = vertices.len() as u32;

        Self {
            vertex_data: vertices,
            num_elements,
        }
    }

    // todo: fix faces
    pub(crate) fn get_cube_vertices() -> [f32; 180] {
        #[rustfmt::skip]
        let cube_vertices: [f32; 180] = [
            // positions       // texture Coords
            -0.5, -0.5, -0.5,  0.0, 0.0,
             0.5, -0.5, -0.5,  1.0, 0.0,
             0.5,  0.5, -0.5,  1.0, 1.0,
             0.5,  0.5, -0.5,  1.0, 1.0,
            -0.5,  0.5, -0.5,  0.0, 1.0,
            -0.5, -0.5, -0.5,  0.0, 0.0,

            -0.5, -0.5,  0.5,  0.0, 0.0,
             0.5, -0.5,  0.5,  1.0, 0.0,
             0.5,  0.5,  0.5,  1.0, 1.0,
             0.5,  0.5,  0.5,  1.0, 1.0,
            -0.5,  0.5,  0.5,  0.0, 1.0,
            -0.5, -0.5,  0.5,  0.0, 0.0,

            -0.5,  0.5,  0.5,  1.0, 0.0,
            -0.5, -0.5, -0.5,  0.0, 1.0,
            -0.5,  0.5, -0.5,  1.0, 1.0,
            -0.5, -0.5, -0.5,  0.0, 1.0,
            -0.5,  0.5,  0.5,  1.0, 0.0,
            -0.5, -0.5,  0.5,  0.0, 0.0,

             0.5,  0.5,  0.5,  1.0, 0.0,
             0.5, -0.5, -0.5,  0.0, 1.0,
             0.5,  0.5, -0.5,  1.0, 1.0,
             0.5, -0.5, -0.5,  0.0, 1.0,
             0.5,  0.5,  0.5,  1.0, 0.0,
             0.5, -0.5,  0.5,  0.0, 0.0,

            -0.5, -0.5, -0.5,  0.0, 1.0,
             0.5, -0.5, -0.5,  1.0, 1.0,
             0.5, -0.5,  0.5,  1.0, 0.0,
             0.5, -0.5,  0.5,  1.0, 0.0,
            -0.5, -0.5,  0.5,  0.0, 0.0,
            -0.5, -0.5, -0.5,  0.0, 1.0,

            -0.5,  0.5, -0.5,  0.0, 1.0,
             0.5,  0.5, -0.5,  1.0, 1.0,
             0.5,  0.5,  0.5,  1.0, 0.0,
             0.5,  0.5,  0.5,  1.0, 0.0,
            -0.5,  0.5,  0.5,  0.0, 0.0,
            -0.5,  0.5, -0.5,  0.0, 1.0
        ];

        cube_vertices
    }
}
