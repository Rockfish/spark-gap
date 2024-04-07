
const AMBIENT_COLOR: vec3<f32> = vec3<f32>(0.05, 0.05, 0.05);
const MAX_LIGHTS: u32 = 10u;

//struct ShaderParams {
//    projection_view: mat4x4<f32>,
//    num_lights: vec4<u32>,
//};

struct Light {
    projection_view: mat4x4<f32>,
    position: vec4<f32>,
    color: vec4<f32>,
};

struct Entity {
    world: mat4x4<f32>,
    color: vec4<f32>,
};

@group(0) @binding(0) var<uniform> projection_view: mat4x4<f32>;
@group(0) @binding(1) var<uniform> num_lights: u32;

@group(0) @binding(2) var<uniform> lights_uniform: array<Light, MAX_LIGHTS>;
@group(0) @binding(3) var shadow_texture: texture_depth_2d_array;
@group(0) @binding(4) var shadow_sampler: sampler_comparison;

@group(1) @binding(0) var<uniform> entity_data: Entity;

@vertex fn vs_bake(@location(0) position: vec4<i32>) -> @builtin(position) vec4<f32> {
    return projection_view * entity_data.world * vec4<f32>(position);
}

struct VertexOutput {
    @builtin(position) proj_position: vec4<f32>,
    @location(0) world_normal: vec3<f32>,
    @location(1) world_position: vec4<f32>
};

@vertex fn vs_main(@location(0) position: vec4<i32>, @location(1) normal: vec4<i32>) -> VertexOutput {
    let w = entity_data.world;
    let world_pos = entity_data.world * vec4<f32>(position);
    var result: VertexOutput;
    result.world_normal = mat3x3<f32>(w[0].xyz, w[1].xyz, w[2].xyz) * vec3<f32>(normal.xyz);
    result.world_position = world_pos;
    result.proj_position = projection_view * world_pos;
    return result;
}

// fragment shader

fn fetch_shadow(light_id: u32, homogeneous_coords: vec4<f32>) -> f32 {
    if (homogeneous_coords.w <= 0.0) {
        return 1.0;
    }
    // compensate for the Y-flip difference between the NDC and texture coordinates
    let flip_correction = vec2<f32>(0.5, -0.5);
    // compute texture coordinates for shadow lookup
    let proj_correction = 1.0 / homogeneous_coords.w;
    let light_local = homogeneous_coords.xy * flip_correction * proj_correction + vec2<f32>(0.5, 0.5);
    // do the lookup, using HW PCF and comparison
    return textureSampleCompareLevel(shadow_texture, shadow_sampler, light_local, i32(light_id), homogeneous_coords.z * proj_correction);
}

@fragment fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {

    let normal = normalize(vertex.world_normal);
    var color: vec3<f32> = AMBIENT_COLOR;
    
    for (var i = 0u; i < min(num_lights, MAX_LIGHTS); i += 1u) {
        let light = lights_uniform[i];
        let shadow = fetch_shadow(i, light.projection_view * vertex.world_position);
        let light_dir = normalize(light.position.xyz - vertex.world_position.xyz);
        let diffuse = max(0.0, dot(normal, light_dir));
        color += shadow * diffuse * light.color.xyz;
    }

    return vec4<f32>(color, 1.0) * entity_data.color;
}
