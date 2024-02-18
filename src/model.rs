use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;
use wgpu::{BindGroup, Buffer};
use crate::animator::{AnimationClip, Animator, WeightedAnimation};
use crate::model_mesh::ModelMesh;



// model data
#[derive(Debug)]
pub struct Model {
    pub name: Rc<str>,
    pub meshes: Rc<Vec<ModelMesh>>,
    pub animator: RefCell<Animator>,
    pub model_transform_buffer: Buffer,
    pub node_transform_buffer: Buffer,
    pub final_bones_matrices_buffer: Buffer,
    pub bind_group: BindGroup,
}

impl Model {

    pub fn update_animation(&self, delta_time: f32) {
        self.animator.borrow_mut().update_animation(delta_time);
    }

    pub fn play_clip(&self, clip: &Rc<AnimationClip>) {
        self.animator.borrow_mut().play_clip(clip);
    }

    pub fn play_clip_with_transition(
        &self,
        clip: &Rc<AnimationClip>,
        transition_duration: Duration,
    ) {
        self.animator
            .borrow_mut()
            .play_clip_with_transition(clip, transition_duration);
    }

    pub fn play_weight_animations(
        &mut self,
        weighted_animation: &[WeightedAnimation],
        frame_time: f32,
    ) {
        self.animator
            .borrow_mut()
            .play_weight_animations(weighted_animation, frame_time);
    }
}
