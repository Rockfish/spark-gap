// #![allow(dead_code)]
// #![allow(non_snake_case)]
// #![allow(non_camel_case_types)]
// #![allow(unused_assignments)]
// #![allow(unused_variables)]
// #![allow(clippy::zero_ptr)]
// #![allow(clippy::assign_op_pattern)]
//
//
// use glam::*;
// use std::rc::Rc;
// use std::time::Duration;
// use spark_gap::animator::{AnimationClip, AnimationRepeat};
// use spark_gap::camera::CameraUniform;
// use spark_gap::model_builder::ModelBuilder;
// use spark_gap::texture_config::TextureType;
//
// const SCR_WIDTH: f32 = 800.0;
// const SCR_HEIGHT: f32 = 800.0;
//
// // Lighting
// const LIGHT_FACTOR: f32 = 1.0;
// const NON_BLUE: f32 = 0.9;
//
// const FLOOR_LIGHT_FACTOR: f32 = 0.35;
// const FLOOR_NON_BLUE: f32 = 0.7;
//
// // Struct for passing state between the window loop and the event handler.
// struct State {
//     camera: CameraUniform,
//     lightPos: Vec3,
//     deltaTime: f32,
//     lastFrame: f32,
//     firstMouse: bool,
//     lastX: f32,
//     lastY: f32,
// }
//
//
// fn model_render_setup() {
//
//     let camera = CameraUniform::camera_vec3(vec3(0.0, 40.0, 120.0));
//
//     // Initialize the world state
//     let mut state = State {
//         camera,
//         lightPos: vec3(1.2, 1.0, 2.0),
//         deltaTime: 0.0,
//         lastFrame: 0.0,
//         firstMouse: true,
//         lastX: SCR_WIDTH / 2.0,
//         lastY: SCR_HEIGHT / 2.0,
//     };
//
//
//     let shader = Rc::new(
//         Shader::new(
//             "examples/sample_animation/player_shader.vert",
//             "examples/sample_animation/player_shader.frag",
//         )
//         .unwrap(),
//     );
//
//     let model_path = "examples/sample_animation/vampire/dancing_vampire.dae";
//     // let model_path = "/Users/john/Dev_Assets/glTF-Sample-Models/2.0/CesiumMan/glTF/CesiumMan.gltf"; // works
//     // let model_path = "/Users/john/Dev_Rust/Repos/OpenGL-Tutorials/LearnOpenGL/8.Guest Articles/2020/2.Skeletal Animation/resources/objects/vampire/dancing_vampire.dae";
//     let model_path = "/Users/john/Dev_Rust/Dev/angry_gl_bots_rust/assets/Models/Player/Player.fbx";
//     // let model_path = "/Users/john/Dev_Rust/Dev/bevy/assets/models/animated/Fox.glb";
//     // let model_path = "/Users/john/Dev_Assets/animated-characters-3/Model/characterMedium.fbx";
//     // let model_path = "/Users/john/Dev_Rust/Dev/alien_explorer/assets/models/alien.glb";
//     // let cube = Cube::new("cube", shader.clone());
//     // let model_path = "examples/sample_animation/source/cube_capoeira_martelo_cruzando.FBX.fbx"; // platform with martial arts guy
//     // let model_path = "/Users/john/Dev_Rust/Repos/ogldev/Content/box.obj"; // no animations
//     // let model_path = "/Users/john/Dev_Rust/Repos/OpenGL-Animation/Resources/res/model.dae"; // doesn't load
//     // let model_path = "examples/sample_animation/colorful_cube/scene.gltf";  // small cube, doesn't animate
//     // let model_path = "/Users/john/Dev_Rust/Dev/learn_opengl_with_rust/resources/objects/cyborg/cyborg.obj"; // not animated
//
//     // let scene = AssimpScene::load_assimp_scene(model_path).unwrap();
//     // let scene = ModelBuilder::load_russimp_scene(model_path).unwrap();
//
//     #[rustfmt::skip]
//     let dancing_model = ModelBuilder::new("model", model_path)
//         .add_texture("Player", TextureType::Diffuse, "/Users/john/Dev_Rust/Dev/angry_gl_bots_rust/assets/Models/Player/Textures/Player_D.tga") // Player model
//         .add_texture( "Player", TextureType::Specular, "/Users/john/Dev_Rust/Dev/angry_gl_bots_rust/assets/Models/Player/Textures/Player_M.tga") // Player model
//         .add_texture( "Player", TextureType::Emissive, "/Users/john/Dev_Rust/Dev/angry_gl_bots_rust/assets/Models/Player/Textures/Player_E.tga", ) // Player model
//         .add_texture( "Player", TextureType::Normals, "/Users/john/Dev_Rust/Dev/angry_gl_bots_rust/assets/Models/Player/Textures/Player_NRM.tga", ) // Player model
//         .add_texture( "Gun", TextureType::Diffuse, "/Users/john/Dev_Rust/Dev/angry_gl_bots_rust/assets/Models/Player/Textures/Gun_D.tga", ) // Player model
//         .add_texture( "Gun", TextureType::Specular, "/Users/john/Dev_Rust/Dev/angry_gl_bots_rust/assets/Models/Player/Textures/Gun_M.tga", ) // Player model
//         .add_texture( "Gun", TextureType::Emissive, "/Users/john/Dev_Rust/Dev/angry_gl_bots_rust/assets/Models/Player/Textures/Gun_E.tga", ) // Player model
//         .add_texture( "Gun", TextureType::Normals, "/Users/john/Dev_Rust/Dev/angry_gl_bots_rust/assets/Models/Player/Textures/Gun_NRM.tga", ) // Player model
//         .build()
//         .unwrap();
//
//     let idle = Rc::new(AnimationClip::new(55.0, 130.0, AnimationRepeat::Forever));
//     let forward = Rc::new(AnimationClip::new(134.0, 154.0, AnimationRepeat::Forever));
//     let backwards = Rc::new(AnimationClip::new(159.0, 179.0, AnimationRepeat::Forever));
//     let right = Rc::new(AnimationClip::new(184.0, 204.0, AnimationRepeat::Forever));
//     let left = Rc::new(AnimationClip::new(209.0, 229.0, AnimationRepeat::Forever));
//     let dying = Rc::new(AnimationClip::new(234.0, 293.0, AnimationRepeat::Once));
//
//     dancing_model.play_clip(&idle);
//     dancing_model.play_clip_with_transition(&forward, Duration::from_secs(6));
//
//     let model_path = "/Users/john/Dev_Rust/Dev/angry_gl_bots_rust/assets/Models/Bullet/Bullet.FBX";
//     let bullet_model = ModelBuilder::new("bullet", model_path)
//         .add_texture(
//             "Plane001",
//             TextureType::Diffuse,
//             "/Users/john/Dev_Rust/Dev/angry_gl_bots_rust/assets/Models/Bullet/Textures/BulletTexture.png",
//         )
//         .build()
//         .unwrap();
//
//     // animator.play_clip_with_transition(&forward, Duration::from_secs(6));
//     // animator.play_clip_with_transition(&right, Duration::from_secs(6));
//     // animator.play_clip_with_transition(&left, Duration::from_secs(3));
//     // animator.play_clip_with_transition(&backwards, Duration::from_secs(3));
//     // animator.play_clip_with_transition(&dying, Duration::from_secs(3));
//     // animator.play_clip_with_transition(&left, Duration::from_secs(10));
//
//     // Lighting
//     let lightDir: Vec3 = vec3(-0.8, 0.0, -1.0).normalize_or_zero();
//     let playerLightDir: Vec3 = vec3(-1.0, -1.0, -1.0).normalize_or_zero();
//
//     let lightColor: Vec3 = LIGHT_FACTOR * 1.0 * vec3(NON_BLUE * 0.406, NON_BLUE * 0.723, 1.0);
//     // const lightColor: Vec3 = LIGHT_FACTOR * 1.0 * vec3(0.406, 0.723, 1.0);
//
//     let floorLightColor: Vec3 = FLOOR_LIGHT_FACTOR * 1.0 * vec3(FLOOR_NON_BLUE * 0.406, FLOOR_NON_BLUE * 0.723, 1.0);
//     let floorAmbientColor: Vec3 = FLOOR_LIGHT_FACTOR * 0.50 * vec3(FLOOR_NON_BLUE * 0.7, FLOOR_NON_BLUE * 0.7, 0.7);
//
//     let ambientColor: Vec3 = LIGHT_FACTOR * 1.0 * vec3(NON_BLUE * 0.7, NON_BLUE * 0.7, 0.7);
//
//     state.lastFrame = glfw.get_time() as f32;
//
//     // render loop
//     while !window.should_close() {
//         let currentFrame = glfw.get_time() as f32;
//         state.deltaTime = currentFrame - state.lastFrame;
//         state.lastFrame = currentFrame;
//
//
//         dancing_model.update_animation(state.deltaTime);
//
//         unsafe {
//             // render
//             gl::ClearColor(0.05, 0.1, 0.05, 1.0);
//             gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
//
//             // be sure to activate shader when setting uniforms/drawing objects
//             shader.use_shader();
//
//             // view/projection transformations
//             let projection = Mat4::perspective_rh_gl(state.camera.zoom.to_radians(), SCR_WIDTH / SCR_HEIGHT, 0.1, 1000.0);
//
//             let view = state.camera.get_view_matrix();
//             shader.set_mat4("projection", &projection);
//             shader.set_mat4("view", &view);
//
//             let mut model = Mat4::IDENTITY;
//             // model *= Mat4::from_rotation_x(-90.0f32.to_radians());
//             model *= Mat4::from_translation(vec3(0.0, -10.4, -400.0));
//             // model *= Mat4::from_scale(vec3(0.3, 0.3, 0.3));
//             // let mut model = Mat4::from_translation(vec3(0.0, 5.0, 0.0));
//             // model = model * Mat4::from_scale(vec3(15.0, 15.0, 15.0));
//             model = model * Mat4::from_scale(vec3(1.0, 1.0, 1.0));
//
//             shader.set_mat4("model", &model);
//
//             shader.set_bool("useLight", true);
//             shader.set_vec3("ambient", &ambientColor);
//
//             shader.set_mat4("aimRot", &Mat4::IDENTITY);
//             shader.set_mat4("lightSpaceMatrix", &Mat4::IDENTITY);
//
//             dancing_model.render(&shader);
//
//             let mut model = Mat4::IDENTITY;
//             // model *= Mat4::from_rotation_x(-90.0f32.to_radians());
//             model = model * Mat4::from_scale(vec3(2.0, 2.0, 2.0));
//
//             shader.set_mat4("model", &model);
//             bullet_model.render(&shader);
//         }
//
//         window.swap_buffers();
//     }
// }
//
// //
// // GLFW maps callbacks to events.
// //
// fn handle_window_event(window: &mut glfw::Window, event: glfw::WindowEvent, state: &mut State) {
//     match event {
//         glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => window.set_should_close(true),
//         glfw::WindowEvent::FramebufferSize(width, height) => {
//             framebuffer_size_event(window, width, height);
//         }
//         glfw::WindowEvent::Key(Key::W, _, _, _) => {
//             state.camera.process_keyboard(CameraMovement::Forward, state.deltaTime);
//         }
//         glfw::WindowEvent::Key(Key::S, _, _, _) => {
//             state.camera.process_keyboard(CameraMovement::Backward, state.deltaTime);
//         }
//         glfw::WindowEvent::Key(Key::A, _, _, _) => {
//             state.camera.process_keyboard(CameraMovement::Left, state.deltaTime);
//         }
//         glfw::WindowEvent::Key(Key::D, _, _, _) => {
//             state.camera.process_keyboard(CameraMovement::Right, state.deltaTime);
//         }
//         glfw::WindowEvent::CursorPos(xpos, ypos) => mouse_handler(state, xpos, ypos),
//         glfw::WindowEvent::Scroll(xoffset, ysoffset) => scroll_handler(state, xoffset, ysoffset),
//         _evt => {
//             // println!("WindowEvent: {:?}", evt);
//         }
//     }
// }
//
// fn mouse_handler(state: &mut State, xposIn: f64, yposIn: f64) {
//     let xpos = xposIn as f32;
//     let ypos = yposIn as f32;
//
//     if state.firstMouse {
//         state.lastX = xpos;
//         state.lastY = ypos;
//         state.firstMouse = false;
//     }
//
//     let xoffset = xpos - state.lastX;
//     let yoffset = state.lastY - ypos; // reversed since y-coordinates go from bottom to top
//
//     state.lastX = xpos;
//     state.lastY = ypos;
//
//     state.camera.process_mouse_movement(xoffset, yoffset, true);
// }
//
// fn scroll_handler(state: &mut State, _xoffset: f64, yoffset: f64) {
//     state.camera.process_mouse_scroll(yoffset as f32);
// }
