extern crate glam;

use glam::{vec4, Vec3, Vec4Swizzles};

pub fn screen_to_model_glam(
    mouse_x: f32,
    mouse_y: f32,
    viewport_width: f32,
    viewport_height: f32,
    view_matrix: &glam::Mat4,
    projection_matrix: &glam::Mat4,
) -> Vec3 {
    // Convert screen coordinates to normalized device coordinates

    let ndc_x = (2.0 * mouse_x) / viewport_width - 1.0;
    let ndc_y = 1.0 - (2.0 * mouse_y) / viewport_height;
    let ndc_z = 0.7345023; // 1.0; // Assuming the point is on the near plane
    let ndc = glam::Vec4::new(ndc_x, ndc_y, ndc_z, 1.0);

    // debug!("ndc: {:?}", ndc);

    // Convert NDC to clip space (inverse projection matrix)
    let clip_space = projection_matrix.inverse() * ndc;

    // Convert clip space to eye space (w-divide)
    let eye_space = glam::Vec4::new(clip_space.x / clip_space.w, clip_space.y / clip_space.w, -1.0, 0.0);
    // let eye_space = clip_space / clip_space.w;

    // Convert eye space to world space (inverse view matrix)
    let world_space = view_matrix.inverse() * eye_space;

    Vec3::new(world_space.x, world_space.y, world_space.z)
}

pub fn get_world_ray_from_mouse(
    mouse_x: f32,
    mouse_y: f32,
    viewport_width: f32,
    viewport_height: f32,
    view_matrix: &glam::Mat4,
    projection: &glam::Mat4,
) -> Vec3 {
    // normalize device coordinates
    let ndc_x = (2.0 * mouse_x) / viewport_width - 1.0;
    let ndc_y = 1.0 - (2.0 * mouse_y) / viewport_height;
    let ndc_z = -1.0; // face the same direction as the opengl camera
    let ndc = glam::Vec4::new(ndc_x, ndc_y, ndc_z, 1.0);

    let projection_inverse = projection.inverse();
    let view_inverse = view_matrix.inverse();

    // eye space
    let ray_eye = projection_inverse * ndc;
    let ray_eye = vec4(ray_eye.x, ray_eye.y, -1.0, 0.0);

    // world space
    let ray_world = (view_inverse * ray_eye).xyz();

    // ray from camera
    ray_world.normalize_or_zero()
}

pub fn ray_plane_intersection(ray_origin: Vec3, ray_direction: Vec3, plane_point: Vec3, plane_normal: Vec3) -> Option<Vec3> {
    let denom = plane_normal.dot(ray_direction);
    if denom.abs() > f32::EPSILON {
        let p0l0 = plane_point - ray_origin;
        let t = p0l0.dot(plane_normal) / denom;
        if t >= 0.0 {
            return Some(ray_origin + t * ray_direction);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use crate::math::{get_world_ray_from_mouse, ray_plane_intersection, screen_to_model_glam};
    use glam::{vec2, vec3, vec4, Mat4, Vec4Swizzles};
    use log::debug;

    #[test]
    fn test_screen_to_model() {
        let mouse_x: f32 = 50.0;
        let mouse_y: f32 = 50.0;
        let viewport_width: f32 = 100.0;
        let viewport_height: f32 = 100.0;
        let zoom: f32 = 45.0;

        let projection = Mat4::perspective_rh_gl(zoom.to_radians(), viewport_width / viewport_height, 1.0, 100.0);

        let player_position = vec3(0.0, 0.0, 0.0);

        let camera_follow_vec = vec3(-4.0, 4.3, 0.0);
        let camera_position = player_position + camera_follow_vec;

        let camera_up = vec3(0.0, 1.0, 0.0);

        let view_matrix = Mat4::look_at_rh(camera_position, player_position, camera_up);

        debug!("view_matrix: {:?}", view_matrix);

        let result = screen_to_model_glam(mouse_x, mouse_y, viewport_width, viewport_height, &view_matrix, &projection);

        debug!("result: {:?}", result);
    }

    #[test]
    fn test_step_by_step() {
        let viewport_width: f32 = 100.0;
        let viewport_height: f32 = 100.0;
        let zoom: f32 = 45.0;

        let model_position = glam::Vec3::new(0.0, 0.0, 0.0);
        // let model_position_vec4 = glam::Vec4::new(0.0, 1.0, 0.0, 1.0);
        let model_transform = Mat4::from_translation(model_position);
        // debug!("model_transform: {:?}\n", model_transform);

        let zoom_radians = zoom.to_radians();

        debug!("zoom_radians: {:?}", zoom_radians);

        let projection = Mat4::perspective_rh_gl(zoom_radians, viewport_width / viewport_height, 1.0, 100.0);

        debug!("projection: {:?}\n", projection);

        // let camera_follow_vec = vec3(1.0, 4.3, 0.0);
        // let camera_follow_vec = vec3(0.0, 5.0, 0.0);
        // let camera_position = model_position.clone() + camera_follow_vec;

        let camera_position = vec3(0.0, 5.0, -5.0);
        let center = vec3(0.0, 0.0, 0.0);
        let camera_up = vec3(0.0, 1.0, 0.0);

        let view_matrix = Mat4::look_at_rh(camera_position, center, camera_up);

        // forward calculation
        debug!("model_position.extend(1.0): {:?}", model_position.extend(1.0));

        let mvp_matrix = projection * view_matrix * model_transform;
        let clip_space_position = mvp_matrix * model_position.extend(1.0);

        debug!("clip_space_position: {:?}", clip_space_position);

        // let clip_inverse = mvp_matrix.inverse() * clip_space_position;
        // debug!("clip_inverse: {:?}", clip_inverse);

        // Screen position
        let clip_screen_position_w = clip_space_position / clip_space_position.w;
        debug!("clip_screen_position_w: {:?}\n", clip_screen_position_w);

        // reverse clip_screen_position
        let inv_clip_screen_position = mvp_matrix.inverse() * clip_space_position;
        debug!("inv_clip_screen_position: {:?}", inv_clip_screen_position);

        let inv_clip_w = inv_clip_screen_position / inv_clip_screen_position.w;
        debug!("inv_clip_w: {:?}\n", inv_clip_w);

        // let clip_inverse_w = clip_inverse / clip_inverse.w;
        // debug!("clip_inverse_w: {:?}", clip_inverse_w);

        let scene_position = vec2(
            clip_screen_position_w.x * viewport_width,
            clip_screen_position_w.y * viewport_height,
        );
        debug!("scene_position: {:?}", scene_position);

        let mouse_x = ((clip_screen_position_w.x + 1.0) * viewport_width) / 2.0;
        let mouse_y = ((-clip_screen_position_w.y + 1.0) * viewport_height) / 2.0;

        let mouse = vec2(mouse_x, mouse_y);
        debug!("mouse: {:?}", mouse);

        // glReadPixels(winX, winY, 1, 1, GL_DEPTH_COMPONENT, GL_FLOAT, &winZ);

        // reverse calculation

        let ndc_x = (2.0 * mouse_x) / viewport_width - 1.0;
        let ndc_y = 1.0 - (2.0 * mouse_y) / viewport_height;
        let ndc_z = -1.0; // -1 to face the same direction as the opengl camera
        let ndc = glam::Vec4::new(ndc_x, ndc_y, ndc_z, 1.0);

        debug!("ndc: {:?}\n", ndc);

        let projection_inverse = projection.inverse();
        let view_inverse = view_matrix.inverse();

        let ray_eye = projection_inverse * ndc;
        let ray_eye = vec4(ray_eye.x, ray_eye.y, -1.0, 0.0);

        let ray_world = (view_inverse * ray_eye).xyz();
        let ray_world = ray_world.normalize_or_zero();

        debug!("ray_world: {:?}", ray_world);

        // the xz plane
        let plane_point = vec3(0.0, 0.0, 0.0);
        let plane_normal = vec3(0.0, 1.0, 0.0);

        let intersection = ray_plane_intersection(camera_position, ray_world, plane_point, plane_normal);

        debug!("intersection: {:?}", intersection);
    }

    #[test]
    fn test_function() {
        let mouse_x: f32 = 50.0;
        let mouse_y: f32 = 50.0;

        let viewport_width: f32 = 100.0;
        let viewport_height: f32 = 100.0;

        let model_position = glam::Vec3::new(0.0, 0.0, 0.0);
        let _model_transform = Mat4::from_translation(model_position);

        let zoom: f32 = 45.0;
        let zoom_radians = zoom.to_radians();

        let projection = Mat4::perspective_rh_gl(zoom_radians, viewport_width / viewport_height, 1.0, 100.0);

        let camera_position = vec3(0.0, 5.0, -5.0);
        let center = vec3(0.0, 0.0, 0.0);
        let camera_up = vec3(0.0, 1.0, 0.0);

        let view_matrix = Mat4::look_at_rh(camera_position, center, camera_up);

        let world_ray = get_world_ray_from_mouse(mouse_x, mouse_y, viewport_width, viewport_height, &view_matrix, &projection);

        // the xz plane
        let plane_point = vec3(0.0, 0.0, 0.0);
        let plane_normal = vec3(0.0, 1.0, 0.0);

        let intersection = ray_plane_intersection(camera_position, world_ray, plane_point, plane_normal);

        debug!("intersection: {:?}", intersection);
    }
}
