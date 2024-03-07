use glam::*;

// Default camera values
pub const YAW: f32 = -90.0;
pub const PITCH: f32 = 0.0;
pub const SPEED: f32 = 100.5;
pub const SENSITIVITY: f32 = 0.1;
pub const ZOOM: f32 = 45.0;

// Defines several possible options for camera movement. Used as abstraction
// to stay away from window-system specific input methods
pub enum CameraMovement {
    Forward,
    Backward,
    Left,
    Right,
    Up,
    Down,
}

#[derive(Default, Debug, Clone)]
pub struct Camera {
    // camera Attributes
    pub position: Vec3,
    pub front: Vec3,
    pub world_up: Vec3,
    pub up: Vec3,
    pub right: Vec3,
    // euler Angles
    pub yaw: f32,
    pub pitch: f32,
    // camera options
    pub movement_speed: f32,
    pub mouse_sensitivity: f32,
    pub zoom: f32,
}

impl Camera {
    pub fn new() -> Camera {
        Camera {
            position: vec3(0.0, 0.0, 3.0),
            front: vec3(0.0, 0.0, -1.0),
            world_up: vec3(0.0, 1.0, 0.0),
            up: vec3(0.0, 1.0, 0.0),
            right: Default::default(),
            yaw: YAW,
            pitch: PITCH,
            movement_speed: SPEED,
            mouse_sensitivity: SENSITIVITY,
            zoom: ZOOM,
        }
    }

    pub fn camera_vec3(position: Vec3) -> Camera {
        let mut camera = Camera::new();
        camera.position = position;
        camera.update_camera_vectors();
        camera
    }

    pub fn camera_vec3_up_yaw_pitch(position: Vec3, world_up: Vec3, yaw: f32, pitch: f32) -> Camera {
        let mut camera = Camera::new();
        camera.position = position;
        camera.world_up = world_up;
        camera.yaw = yaw;
        camera.pitch = pitch;
        camera.update_camera_vectors();
        camera
    }

    #[allow(clippy::too_many_arguments)]
    pub fn camera_scalar(pos_x: f32, pos_y: f32, pos_z: f32, up_x: f32, up_y: f32, up_z: f32, yaw: f32, pitch: f32) -> Camera {
        let mut camera = Camera::new();
        camera.position = vec3(pos_x, pos_y, pos_z);
        camera.world_up = vec3(up_x, up_y, up_z);
        camera.yaw = yaw;
        camera.pitch = pitch;
        camera.update_camera_vectors();
        camera
    }

    // calculates the front vector from the Camera's (updated) Euler Angles
    fn update_camera_vectors(&mut self) {
        // calculate the new Front vector
        let front = vec3(
            self.yaw.to_radians().cos() * self.pitch.to_radians().cos(),
            self.pitch.to_radians().sin(),
            self.yaw.to_radians().sin() * self.pitch.to_radians().cos(),
        );

        self.front = front.normalize_or_zero();

        // also re-calculate the Right and Up vector
        // normalize the vectors, because their length gets closer to 0 the more you look up or down which results in slower movement.
        self.right = self.front.cross(self.world_up).normalize_or_zero();
        self.up = self.right.cross(self.front).normalize_or_zero();
    }

    // returns the view matrix calculated using Euler Angles and the LookAt Matrix
    pub fn get_view_matrix(&self) -> Mat4 {
        Mat4::look_to_rh(self.position, self.front, self.up)
    }

    // processes input received from any keyboard-like input system. Accepts input parameter
    // in the form of camera defined ENUM (to abstract it from windowing systems)
    pub fn process_keyboard(&mut self, direction: CameraMovement, delta_time: f32) {
        let velocity: f32 = self.movement_speed * delta_time;

        match direction {
            CameraMovement::Forward => self.position += self.front * velocity,
            CameraMovement::Backward => self.position -= self.front * velocity,
            CameraMovement::Left => self.position -= self.right * velocity,
            CameraMovement::Right => self.position += self.right * velocity,
            CameraMovement::Up => self.position += self.up * velocity,
            CameraMovement::Down => self.position -= self.up * velocity,
        }

        // For FPS: make sure the user stays at the ground level
        // self.Position.y = 0.0; // <-- this one-liner keeps the user at the ground level (xz plane)
    }

    // processes input received from a mouse input system. Expects the offset value in both the x and y direction.
    pub fn process_mouse_movement(&mut self, mut xoffset: f32, mut yoffset: f32, constrain_pitch: bool) {
        xoffset *= self.mouse_sensitivity;
        yoffset *= self.mouse_sensitivity;

        self.yaw += xoffset;
        self.pitch += yoffset;

        // make sure that when pitch is out of bounds, screen doesn't get flipped
        if constrain_pitch {
            if self.pitch > 89.0 {
                self.pitch = 89.0;
            }
            if self.pitch < -89.0 {
                self.pitch = -89.0;
            }
        }

        // update Front, Right and Up Vectors using the updated Euler angles
        self.update_camera_vectors();

        // debug!("camera: {:#?}", self);
    }

    // processes input received from a mouse scroll-wheel event. Only requires input on the vertical wheel-axis
    pub fn process_mouse_scroll(&mut self, yoffset: f32) {
        self.zoom -= yoffset;
        if self.zoom < 1.0 {
            self.zoom = 1.0;
        }
        if self.zoom > 45.0 {
            self.zoom = 45.0;
        }
    }
}
