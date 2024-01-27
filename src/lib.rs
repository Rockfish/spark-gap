use glam::{Quat, Vec2, Vec3, Vec4};
use std::ffi::c_void;
use std::mem;
use std::os::raw;

pub mod error;
pub mod texture;

pub const SIZE_OF_FLOAT: usize = mem::size_of::<f32>();
pub const SIZE_OF_VEC2: usize = mem::size_of::<Vec2>();
pub const SIZE_OF_VEC3: usize = mem::size_of::<Vec3>();
pub const SIZE_OF_VEC4: usize = mem::size_of::<Vec4>();
pub const SIZE_OF_QUAT: usize = mem::size_of::<Quat>();
pub const NULL: *const c_void = std::ptr::null::<raw::c_void>();

// use winit::event::WindowEvent::KeyboardInput;
// use winit::{
//     event::*,
//     event_loop::{ControlFlow, EventLoop},
//     window::WindowBuilder,
// };
//
// pub fn run() {
//     env_logger::init();
//     let event_loop = EventLoop::new();
//     let window = WindowBuilder::new().build(&event_loop).unwrap();
//
//     event_loop.run(move |event, _, control_flow| match event {
//         Event::WindowEvent {
//             ref event,
//             window_id,
//         } if window_id == window.id() => match event {
//             WindowEvent::CloseRequested
//             | WindowEvent::KeyboardInput {
//                 input:
//                     KeyboardInput {
//                         state: ElementState::Pressed,
//                         virtual_keycode: Some(VirtualKeyCode::Escape),
//                         ..
//                     },
//                 ..
//             } => *control_flow = ControlFlow::Exit,
//             _ => {}
//         },
//         _ => {}
//     });
// }
