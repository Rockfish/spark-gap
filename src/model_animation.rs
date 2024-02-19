use crate::node_animation::NodeAnimation;
use crate::transform::Transform;
use glam::Mat4;
use log::debug;
use russimp::animation::Animation;
use russimp::scene::Scene;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct NodeData {
    pub name: Rc<str>,
    pub transform: Transform,
    pub children: Vec<NodeData>,
    pub meshes: Rc<Vec<u32>>,
}

pub type BoneName = String;

#[derive(Debug, Clone)]
pub struct BoneData {
    pub name: Rc<str>,
    pub bone_index: i32,
    pub offset_transform: Transform,
}

impl BoneData {
    pub fn new(name: &str, id: i32, offset: Mat4) -> Self {
        BoneData {
            name: name.into(),
            bone_index: id,
            offset_transform: Transform::from_matrix(offset),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ModelAnimation {
    pub duration: f32,
    pub ticks_per_second: f32,
    pub node_animations: RefCell<Vec<NodeAnimation>>,
}

impl Default for ModelAnimation {
    fn default() -> Self {
        ModelAnimation {
            duration: 0.0,
            ticks_per_second: 0.0,
            node_animations: RefCell::new(vec![]),
        }
    }
}

impl ModelAnimation {
    pub fn new(scene: &Scene) -> Self {
        if scene.animations.is_empty() {
            return ModelAnimation::default();
        }

        let duration = scene.animations[0].duration as f32;
        let ticks_per_second = scene.animations[0].ticks_per_second as f32;

        debug!("animation - duration: {}   ticks_per_second: {}", &duration, &ticks_per_second);

        // debug!("root_node: {:#?}", &root_node);
        // debug!("bone_data_map: {:#?}", &model.bone_data_map.borrow());

        let mut model_animation = ModelAnimation {
            // model: model.clone(),
            duration,
            ticks_per_second,
            node_animations: vec![].into(),
        };

        model_animation.read_channel_node_animations(&scene.animations[0]);
        model_animation
    }

    /// converts channel vec of Russimp::NodeAnims into vec of NodeAnimation
    fn read_channel_node_animations(&mut self, animation: &Animation) {
        for channel in &animation.channels {
            let node_animation = NodeAnimation::new(&channel.name.clone(), channel);
            self.node_animations.borrow_mut().push(node_animation);
        }
    }
}
