use crate::texture::Texture;
use glam::u32;
use glam::*;
use log::debug;
use std::mem;
use std::rc::Rc;

const MAX_BONE_INFLUENCE: usize = 4;
const OFFSET_OF_NORMAL: usize = mem::offset_of!(ModelVertex, normal);
const OFFSET_OF_TEXCOORDS: usize = mem::offset_of!(ModelVertex, uv);
const OFFSET_OF_TANGENT: usize = mem::offset_of!(ModelVertex, tangent);
const OFFSET_OF_BITANGENT: usize = mem::offset_of!(ModelVertex, bi_tangent);
const OFFSET_OF_BONE_IDS: usize = mem::offset_of!(ModelVertex, bone_ids);
const OFFSET_OF_WEIGHTS: usize = mem::offset_of!(ModelVertex, bone_weights);

#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
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
}

impl Default for ModelVertex {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct ModelMesh {
    pub id: i32,
    pub name: String,
    pub vertices: Vec<ModelVertex>,
    pub indices: Vec<u32>,
    pub textures: Vec<Rc<Texture>>,
}

impl ModelMesh {
    pub fn new(
        id: i32,
        name: impl Into<String>,
        vertices: Vec<ModelVertex>,
        indices: Vec<u32>,
        textures: Vec<Rc<Texture>>,
    ) -> ModelMesh {
        let mut mesh = ModelMesh {
            id,
            name: name.into(),
            vertices,
            indices,
            textures,
        };
        // mesh.setup_mesh();
        mesh
    }
}
