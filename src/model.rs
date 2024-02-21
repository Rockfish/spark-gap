use crate::animator::{AnimationClip, Animator, WeightedAnimation};
use crate::gpu_context::GpuContext;
use crate::model_mesh::ModelMesh;
use glam::Mat4;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;
use wgpu::{BindGroup, Buffer};
use crate::texture_config::TextureType;

// model data
#[derive(Debug)]
pub struct Model {
    pub name: Rc<str>,
    pub meshes: Rc<Vec<ModelMesh>>,
    pub animator: RefCell<Animator>,
    pub model_transform: Mat4,
    pub model_transform_buffer: Buffer,
    pub node_transform_buffer: Buffer,
    pub final_bones_matrices_buffer: Buffer,
    pub bind_group: BindGroup, // binds buffers into a group
}

impl Model {
    pub fn update_animation(&self, delta_time: f32) {
        self.animator.borrow_mut().update_animation(delta_time);
    }

    pub fn play_clip(&self, clip: &Rc<AnimationClip>) {
        self.animator.borrow_mut().play_clip(clip);
    }

    pub fn play_clip_with_transition(&self, clip: &Rc<AnimationClip>, transition_duration: Duration) {
        self.animator.borrow_mut().play_clip_with_transition(clip, transition_duration);
    }

    pub fn play_weight_animations(&mut self, weighted_animation: &[WeightedAnimation], frame_time: f32) {
        self.animator.borrow_mut().play_weight_animations(weighted_animation, frame_time);
    }

    pub fn update_model_buffers(&self, context: &GpuContext, model_transform: &Mat4) {
        let animator = self.animator.borrow();
        let final_bones = animator.final_bone_matrices.borrow();

        context.queue.write_buffer(
            &self.model_transform_buffer,
            0,
            bytemuck::cast_slice(&model_transform.to_cols_array()),
        );

        context.queue.write_buffer(
            &self.final_bones_matrices_buffer,
            0,
            bytemuck::cast_slice(final_bones.as_ref()));
    }

    pub fn update_mesh_buffers(&self, context: &GpuContext, mesh: &ModelMesh) {
        let animator = self.animator.borrow();
        let final_nodes = animator.final_node_matrices.borrow();

        let node_transform = &final_nodes[mesh.id as usize].to_cols_array();

        context.queue.write_buffer(
            &self.node_transform_buffer,
            0,
            bytemuck::cast_slice(node_transform));
    }

    pub fn get_material_bind_group<'a>(&'a self, mesh: &'a ModelMesh, texture_type: TextureType) -> &BindGroup {
        let diffuse_material = mesh
            .materials
            .iter()
            .find(|m| m.texture_type == texture_type).unwrap();

        &diffuse_material.bind_group
    }
}
