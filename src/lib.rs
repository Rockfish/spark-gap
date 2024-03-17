#![feature(box_into_inner)]

use glam::{Quat, Vec2, Vec3, Vec4};
use std::ffi::c_void;
use std::mem;
use std::os::raw;

pub mod animator;
pub mod camera;
pub mod error;
pub mod frame_counter;
pub mod gpu_context;
pub mod hash_any;
pub mod hash_map;
pub mod input;
pub mod material;
pub mod math;
pub mod model;
pub mod model_animation;
pub mod model_builder;
pub mod model_mesh;
pub mod node_animation;
pub mod texture;
pub mod texture_config;
pub mod transform;
pub mod utils;

pub const SIZE_OF_FLOAT: usize = mem::size_of::<f32>();
pub const SIZE_OF_VEC2: usize = mem::size_of::<Vec2>();
pub const SIZE_OF_VEC3: usize = mem::size_of::<Vec3>();
pub const SIZE_OF_VEC4: usize = mem::size_of::<Vec4>();
pub const SIZE_OF_QUAT: usize = mem::size_of::<Quat>();
pub const NULL: *const c_void = std::ptr::null::<raw::c_void>();
