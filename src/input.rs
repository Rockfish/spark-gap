use crate::hash_map::HashSet;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use winit::event::MouseButton;
use winit::keyboard::{KeyCode, PhysicalKey};

#[derive(Debug, Clone)]
pub struct Input {
    pub keys_pressed: HashSet<KeyCode>,
    pub keys_held: HashSet<KeyCode>,
    pub mouse_buttons_pressed: HashSet<MouseButton>,
    pub mouse_buttons_held: HashSet<MouseButton>,
    pub mouse_position: Option<glam::Vec2>,
    pub mouse_delta: glam::Vec2,
    pub mouse_wheel_delta: f32,
    pub last_update: std::time::Instant,
    pub update_delta: std::time::Duration,
    pub enabled: Arc<AtomicBool>,
}

impl Default for Input {
    fn default() -> Self {
        Self {
            keys_pressed: HashSet::default(),
            keys_held: HashSet::default(),
            mouse_buttons_pressed: HashSet::default(),
            mouse_buttons_held: HashSet::default(),
            mouse_position: None,
            mouse_delta: glam::Vec2::ZERO,
            mouse_wheel_delta: 0.0,
            last_update: std::time::Instant::now(),
            update_delta: std::time::Duration::ZERO,
            enabled: Arc::new(AtomicBool::new(true)),
        }
    }
}

impl Input {
    pub fn disable_input(&self) {
        self.enabled.store(false, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn enable_input(&self) {
        self.enabled.store(true, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn is_input_enabled(&self) -> bool {
        self.enabled.load(std::sync::atomic::Ordering::Relaxed)
    }

    pub fn key_just_pressed(&self, key: KeyCode) -> bool {
        self.keys_pressed.contains(&key)
    }

    pub fn key_pressed(&self, key: KeyCode) -> bool {
        self.keys_held.contains(&key)
    }

    pub fn mouse_button_just_pressed(&self, button: MouseButton) -> bool {
        self.mouse_buttons_pressed.contains(&button)
    }

    pub fn mouse_button_pressed(&self, button: MouseButton) -> bool {
        self.mouse_buttons_held.contains(&button)
    }

    pub fn mouse_position(&self) -> Option<glam::Vec2> {
        self.mouse_position
    }

    pub fn mouse_delta(&self) -> glam::Vec2 {
        self.mouse_delta
    }

    pub fn mouse_wheel_delta(&self) -> f32 {
        self.mouse_wheel_delta
    }

    pub fn prepare_for_update(&mut self) {
        self.keys_pressed.clear();
        self.mouse_buttons_pressed.clear();
        self.mouse_delta = glam::Vec2::ZERO;
        self.mouse_wheel_delta = 0.0;

        let now = std::time::Instant::now();
        self.update_delta = now - self.last_update;
        self.last_update = now;
    }

    pub fn secs_since_last_update(&self) -> f32 {
        self.update_delta.as_secs_f32()
    }

    pub fn handle_window_event(&mut self, event: &winit::event::WindowEvent) {
        if !self.is_input_enabled() {
            return;
        }
        match event {
            winit::event::WindowEvent::MouseInput { state, button, .. } => match state {
                winit::event::ElementState::Pressed => {
                    if !self.mouse_buttons_held.contains(button) {
                        self.mouse_buttons_pressed.insert(*button);
                    }
                    self.mouse_buttons_held.insert(*button);
                }
                winit::event::ElementState::Released => {
                    self.mouse_buttons_held.remove(button);
                }
            },
            winit::event::WindowEvent::CursorMoved { position, .. } => {
                self.mouse_position = Some(glam::Vec2::new(position.x as f32, position.y as f32));
            }
            winit::event::WindowEvent::KeyboardInput { event, is_synthetic, .. } => {
                if *is_synthetic {
                    return;
                }

                if let PhysicalKey::Code(code) = event.physical_key {
                    match event.state {
                        winit::event::ElementState::Pressed => {
                            if !self.keys_held.contains(&code) {
                                self.keys_pressed.insert(code);
                            }
                            self.keys_held.insert(code);
                        }
                        winit::event::ElementState::Released => {
                            self.keys_held.remove(&code);
                        }
                    }
                }
            }
            winit::event::WindowEvent::MouseWheel { delta, .. } => match delta {
                winit::event::MouseScrollDelta::LineDelta(_, y) => {
                    self.mouse_wheel_delta += *y;
                }
                winit::event::MouseScrollDelta::PixelDelta(_xy) => {}
            },
            _ => {}
        }
    }

    pub fn handle_device_event(&mut self, event: &winit::event::DeviceEvent) {
        if !self.is_input_enabled() {
            return;
        }
        if let winit::event::DeviceEvent::MouseMotion { delta } = event {
            self.mouse_delta += glam::Vec2::new(delta.0 as f32, delta.1 as f32);
        }
    }
}
