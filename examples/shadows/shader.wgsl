
const AMBIENT_COLOR: vec3<f32> = vec3<f32>(0.05, 0.05, 0.05);
const MAX_LIGHTS: u32 = 10u;

struct Light {
    projection_view: mat4x4<f32>,
    position: vec4<f32>,
    color: vec4<f32>,
};

struct Entity {
    world: mat4x4<f32>,
    color: vec4<f32>,
};

@group(0) @binding(0) var<uniform> lights_uniform: array<Light, MAX_LIGHTS>;

@group(0) @binding(1) var<uniform> num_lights: u32;
@group(0) @binding(2) var<uniform> projection_view: mat4x4<f32>;

@group(0) @binding(3) var shadow_texture_array: texture_depth_2d_array;
@group(0) @binding(4) var shadow_sampler: sampler_comparison;

@group(1) @binding(0) var<uniform> entity_data: Entity;

@vertex fn vs_shadow(@location(0) position: vec4<i32>, @builtin(instance_index) index: u32) -> @builtin(position) vec4<f32> {
    let light = lights_uniform[index];
    return light.projection_view * entity_data.world * vec4<f32>(position);
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
    let shadow_depth = textureSampleCompareLevel(
        shadow_texture_array,
        shadow_sampler,
        light_local,
        i32(light_id),
        homogeneous_coords.z * proj_correction);

    return shadow_depth;
}

fn shadow_calculation(light_id: u32, bias: f32, frag_light_space: vec4<f32>, offset: vec2<f32>) -> f32 {

  let proj_correction = frag_light_space.xyz / frag_light_space.w;
  let flip_correction = vec2<f32>(0.5, -0.5);

  let projCoords = proj_correction.xy * flip_correction + vec2<f32>(0.5, 0.5);

  let shadow_depth = textureSampleCompareLevel(
    shadow_texture_array,
    shadow_sampler,
    projCoords.xy + offset,
    i32(light_id),
    proj_correction.z);

  return shadow_depth + bias;
}

/*
    Use Percentage-Closer Filtering (PCF): PCF is a widely used technique to soften the edges of
    shadows and reduce aliasing. It involves averaging multiple depth samples around the shadow
    comparison point. This can be easily implemented in your shadow sampling function in WGSL.
*/
//fn sample_pcf(uv: vec2<f32>, layer: i32, refDepth: f32, mipLevel: i32, kernelSize: i32) -> f32 {
//    var shadow: f32 = 0.0;
//
////    let step = 1.0 / float(textureDimensions(depthTexture, mipLevel).xy);
//
//    let dimensions = textureDimensions(shadow_texture_array, 0).xy;
//    let step = vec2<f32>(1.0, 1.0) / vec2<f32>(f32(dimensions.x), f32(dimensions.y));
//
//    for (var y: i32 = -kernelSize; y <= kernelSize; y += 1) {
//        for (var x: i32 = -kernelSize; x <= kernelSize; x += 1) {
//
//            let sampleUV = uv + vec2<f32>(f32(x), f32(y)) * step;
//
//            shadow += textureSampleCompareLevel(shadow_texture_array, shadow_sampler, vec3<f32>(sampleUV, f32(layer)), refDepth, mipLevel);
//        }
//    }
//    return shadow / f32((2 * kernelSize + 1) * (2 * kernelSize + 1));
//}

// Function to calculate slope-scaled depth bias based on the depth gradient
 fn calculateSlopeBias(depth: f32) -> f32 {
    // Compute partial derivatives of the depth
    let depth_dx = dpdx(depth);
    let depth_dy = dpdy(depth);

    // Calculate the maximum change in depth
    let slope = max(abs(depth_dx), abs(depth_dy));

    // Define the scale factor for the bias, adjust as necessary
    let scale: f32 = 2.0;

    // Compute the slope-scaled bias
    let slopeBias = scale * slope;
    return slopeBias;
}

@fragment fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {

    let normal = normalize(vertex.world_normal);

    var color: vec3<f32> = AMBIENT_COLOR;

    let dimensions = textureDimensions(shadow_texture_array, 0).xy;
    let texelSize = vec2<f32>(1.0, 1.0) / vec2<f32>(f32(dimensions.x), f32(dimensions.y));
    
    for (var i = 0u; i < min(num_lights, MAX_LIGHTS); i += 1u) {
        //let shadow = fetch_shadow(i, light.projection_view * vertex.world_position);

        let light = lights_uniform[i];
        let light_dir = normalize(light.position.xyz - vertex.world_position.xyz);
        var shadow_coords = light.projection_view * vertex.world_position;

        let constant_bias: f32 = 0.005; // A predefined constant bias
        var bias: f32 = max(0.05 * (1.0 - dot(normal, light_dir)), 0.005);
        var slope_bias = calculateSlopeBias(shadow_coords.z);

        bias = constant_bias + bias + slope_bias;

        var shadow = 0.0;

        for (var x = -1; x <= 1; x += 1) {
            for (var y = -1; y <= 1; y += 1) {
                let offset = vec2<f32>(f32(x), f32(y)) * texelSize;
                shadow += shadow_calculation(i, bias, shadow_coords, offset);
            }
        }

        shadow = shadow / 9; // average of neighbors

        let diffuse = max(0.0, dot(normal, light_dir));
        color += shadow * diffuse * light.color.xyz;
    }

    return vec4<f32>(color, 1.0) * entity_data.color;
}
