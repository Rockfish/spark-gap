use crate::animator::{Animator, MAX_BONES};
use crate::context::Context;
use crate::error::Error;
use crate::error::Error::{MeshError, SceneError};
use crate::hash_map::HashMap;
use crate::material::Material;
use crate::model::Model;
use crate::model_animation::{BoneData, BoneName};
use crate::model_mesh::{ModelMesh, ModelVertex};
use crate::texture_config::{TextureConfig, TextureFilter, TextureType, TextureWrap};
use crate::transform::Transform;
use crate::utils::get_exists_filename;
use glam::*;
use log::debug;
use russimp::node::Node;
use russimp::scene::{PostProcess, Scene};
use std::cell::RefCell;
use std::ops::Deref;
use std::path::PathBuf;
use std::rc::Rc;
use wgpu::util::DeviceExt;
use wgpu::{BindGroup, BindGroupLayout, Buffer};

pub const MODEL_BIND_GROUP_LAYOUT: &str = "model_bind_group_layout";

#[derive(Debug)]
pub struct AddedTextures {
    mesh_name: String,
    texture_type: TextureType,
    texture_filename: String,
}

#[derive(Debug)]
pub struct ModelBuilder {
    pub name: String,
    pub meshes: Vec<ModelMesh>,
    pub bone_data_map: RefCell<HashMap<BoneName, BoneData>>,
    pub bone_count: i32,
    pub filepath: String,
    pub directory: PathBuf,
    pub gamma_correction: bool,
    pub flip_v: bool,
    pub flip_h: bool,
    pub load_textures: bool,
    pub textures_cache: RefCell<Vec<Rc<Material>>>,
    added_textures: Vec<AddedTextures>,
    pub mesh_count: i32,
}

impl ModelBuilder {
    pub fn new(name: impl Into<String>, path: impl Into<String>) -> Self {
        let filepath = path.into();
        let directory = PathBuf::from(&filepath).parent().unwrap().to_path_buf();
        ModelBuilder {
            name: name.into(),
            textures_cache: RefCell::new(vec![]),
            meshes: vec![],
            bone_data_map: RefCell::new(HashMap::new()),
            bone_count: 0,
            filepath,
            directory,
            gamma_correction: false,
            flip_v: false,
            flip_h: false,
            load_textures: true,
            added_textures: vec![],
            mesh_count: 0,
        }
    }

    pub fn flipv(mut self) -> Self {
        self.flip_v = true;
        self
    }

    pub fn correct_gamma(mut self) -> Self {
        self.gamma_correction = true;
        self
    }

    pub fn skip_textures(mut self) -> Self {
        self.load_textures = false;
        self
    }

    pub fn add_texture(mut self, mesh_name: impl Into<String>, texture_type: TextureType, texture_filename: impl Into<String>) -> Self {
        let added_texture = AddedTextures {
            mesh_name: mesh_name.into(),
            texture_type,
            texture_filename: texture_filename.into(),
        };
        self.added_textures.push(added_texture);
        self
    }

    pub fn build(mut self, context: &mut Context) -> Result<Model, Error> {
        let scene = ModelBuilder::load_russimp_scene(self.filepath.as_str())?;

        self.load_model(context, &scene)?;

        self.add_textures(context)?;

        let animator = Animator::new(&scene, self.bone_data_map);

        if !context.bind_layout_cache.contains_key(MODEL_BIND_GROUP_LAYOUT) {
            let layout = Self::create_bind_group_layout(context);
            context.bind_layout_cache.insert(String::from(MODEL_BIND_GROUP_LAYOUT), layout);
        }

        let bind_group_layout = context.bind_layout_cache.get(MODEL_BIND_GROUP_LAYOUT).unwrap();

        let model_transform_buffer = Self::create_transform_buffer(context, "model transform buffer", &Mat4::IDENTITY);
        let node_transform_buffer = Self::create_transform_buffer(context, "node transform buffer", &Mat4::IDENTITY);
        let final_bones_matrices_buffer = Self::create_final_bones_buffer(context, &animator.final_bone_matrices);

        let bind_group = Self::create_bind_group(
            context,
            &bind_group_layout,
            &model_transform_buffer,
            &node_transform_buffer,
            &final_bones_matrices_buffer,
        );

        let model = Model {
            name: Rc::from(self.name),
            meshes: Rc::from(self.meshes),
            animator: animator.into(),
            model_transform_buffer,
            node_transform_buffer,
            final_bones_matrices_buffer,
            bind_group,
        };

        Ok(model)
    }

    pub fn load_russimp_scene(file_path: &str) -> Result<Scene, Error> {
        let scene = Scene::from_file(
            file_path,
            vec![
                PostProcess::Triangulate,
                PostProcess::GenerateSmoothNormals,
                PostProcess::FlipUVs,
                PostProcess::CalculateTangentSpace,
                PostProcess::FixOrRemoveInvalidData,
                // PostProcess::JoinIdenticalVertices,
                // PostProcess::SortByPrimitiveType,
                // PostProcess::EmbedTextures,
            ],
        )?;
        Ok(scene)
    }

    fn load_model(&mut self, context: &mut Context, scene: &Scene) -> Result<(), Error> {
        match &scene.root {
            None => Err(SceneError("Error getting scene root node".to_string())),
            Some(root_node) => self.process_node(context, root_node, scene),
        }
    }

    #[allow(clippy::needless_range_loop)]
    fn process_node(&mut self, context: &mut Context, node: &Rc<Node>, scene: &Scene) -> Result<(), Error> {
        for mesh_id in &node.meshes {
            let scene_mesh = &scene.meshes[*mesh_id as usize];
            let mesh = self.process_mesh(context, scene_mesh, scene);
            self.meshes.push(mesh?);
        }

        for child_node in node.children.borrow().iter() {
            self.process_node(context, child_node, scene)?;
        }

        Ok(())
    }

    #[allow(clippy::needless_range_loop)]
    fn process_mesh(&mut self, context: &mut Context, r_mesh: &russimp::mesh::Mesh, scene: &Scene) -> Result<ModelMesh, Error> {
        let mut vertices: Vec<ModelVertex> = vec![];
        let mut indices: Vec<u32> = vec![];
        let mut materials: Vec<Rc<Material>> = vec![];

        for i in 0..r_mesh.vertices.len() {
            let mut vertex = ModelVertex::new();

            // positions
            vertex.position = r_mesh.vertices[i]; // Vec3 has Copy trait

            // normals
            if !r_mesh.normals.is_empty() {
                vertex.normal = r_mesh.normals[i];
            }

            // texture coordinates
            if !r_mesh.texture_coords.is_empty() {
                let tex_coords = r_mesh.texture_coords[0].as_ref().unwrap();
                vertex.uv = vec2(tex_coords[i].x, tex_coords[i].y);
                vertex.tangent = r_mesh.tangents[i];
                vertex.bi_tangent = r_mesh.bitangents[i];
            }
            vertices.push(vertex);
        }

        for face in &r_mesh.faces {
            indices.extend(&face.0)
        }

        let russimp_material = &scene.materials[r_mesh.material_index as usize];

        // debug!("material: {:#?}", material);

        for (r_texture_type, r_texture) in russimp_material.textures.iter() {
            let texture_type = TextureType::convert_from(r_texture_type);
            match self.load_or_get_material(context, &texture_type, r_texture.borrow().filename.as_str()) {
                Ok(material) => materials.push(material),
                Err(e) => debug!("{:?}", e),
            }
        }

        debug!("mesh name: {}", &r_mesh.name);

        self.extract_bone_weights_for_vertices(&mut vertices, r_mesh);

        let mesh = ModelMesh::new(context, self.mesh_count, &r_mesh.name, vertices, indices, materials);

        self.mesh_count += 1;
        Ok(mesh)
    }

    fn extract_bone_weights_for_vertices(&mut self, vertices: &mut [ModelVertex], r_mesh: &russimp::mesh::Mesh) {
        let mut bone_data_map = self.bone_data_map.borrow_mut();

        for bone in &r_mesh.bones {
            let bone_id: i32;

            match bone_data_map.get(&bone.name) {
                None => {
                    let bone_info = BoneData {
                        name: Rc::from(bone.name.as_str()),
                        bone_index: self.bone_count,
                        offset_transform: Transform::from_matrix(bone.offset_matrix),
                    };
                    bone_data_map.insert(bone.name.clone(), bone_info);
                    bone_id = self.bone_count;
                    self.bone_count += 1;
                }

                Some(bone_info) => {
                    bone_id = bone_info.bone_index;
                }
            }

            for bone_weight in &bone.weights {
                let vertex_id = bone_weight.vertex_id as usize;
                let weight = bone_weight.weight;

                assert!(vertex_id <= vertices.len());

                vertices[vertex_id].set_bone_data(bone_id, weight);
            }
        }
    }

    fn add_textures(&mut self, context: &mut Context) -> Result<(), Error> {
        for added_texture in &self.added_textures {
            let texture = self.load_or_get_material(context, &added_texture.texture_type, added_texture.texture_filename.as_str())?;
            let mesh = self.meshes.iter_mut().find(|mesh| mesh.name == added_texture.mesh_name);
            if let Some(model_mesh) = mesh {
                let path = self.directory.join(&added_texture.texture_filename).into_os_string();

                if !model_mesh.materials.iter().any(|t| t.texture_path == path) {
                    model_mesh.materials.push(texture);
                }
            } else {
                return Err(MeshError(format!("add_texture mesh: {} not found", &added_texture.mesh_name)));
            }
        }
        Ok(())
    }

    fn load_or_get_material(
        &self,
        context: &mut Context,
        texture_type: &TextureType,
        texture_filename: &str,
    ) -> Result<Rc<Material>, Error> {
        let filepath = get_exists_filename(&self.directory, texture_filename)?;

        let mut texture_cache = self.textures_cache.borrow_mut();

        let cached_texture = texture_cache.iter().find(|t| t.texture_path == filepath.clone().into_os_string());

        match cached_texture {
            None => {
                let texture = Rc::new(Material::new(
                    context,
                    &filepath,
                    &TextureConfig {
                        flip_v: self.flip_v,
                        flip_h: self.flip_h,
                        gamma_correction: self.gamma_correction,
                        filter: TextureFilter::Linear,
                        wrap: TextureWrap::Repeat,
                        texture_type: *texture_type,
                    },
                )?);
                debug!("loaded texture: {:?}", &texture);
                texture_cache.push(texture.clone());
                Ok(texture)
            }
            Some(texture) => {
                let mut texture_new = texture.deref().clone();
                if texture_new.texture_type != *texture_type {
                    texture_new.texture_type = *texture_type;
                }
                debug!("cloned texture: {:?}", &texture);
                Ok(Rc::new(texture_new))
            }
        }
    }

    fn create_transform_buffer(context: &Context, label: &str, data: &Mat4) -> Buffer {
        context.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(label),
            contents: bytemuck::cast_slice(&data.to_cols_array()),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        })
    }

    fn create_final_bones_buffer(context: &Context, data: &RefCell<Box<[Mat4]>>) -> Buffer {
        context.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("final bones matrices"),
            contents: bytemuck::cast_slice(data.borrow().as_ref()),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        })
    }

    fn create_bind_group_layout(context: &Context) -> BindGroupLayout {
        context.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                // 0: model transform
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
                // 1: node_transform
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // 2: final_bones_matrices
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new((MAX_BONES * 16) as _),
                    },
                    count: None,
                },
            ],
            label: Some("model bind group layout"),
        })
    }

    fn create_bind_group(
        context: &Context,
        bind_group_layout: &BindGroupLayout,
        model_transform: &Buffer,
        node_transform: &Buffer,
        final_bones: &Buffer,
    ) -> BindGroup {
        context.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: model_transform.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: node_transform.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: final_bones.as_entire_binding(),
                },
            ],
            label: Some("model bind group"),
        })
    }
}
