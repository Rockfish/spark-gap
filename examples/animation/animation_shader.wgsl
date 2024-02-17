

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec2<f32>,
    @location(2) tex_coords: vec2<f32>,
    @location(3) tangent: vec2<f32>,
    @location(4) bitangent: vec2<f32>,
    @location(5) bone_ids: vec4<i32>,
    @location(6) weights: vec4<f32>,
}

// camera
@group(0) @binding(1) var<uniform> projection: mat4x4<f32>;
@group(0) @binding(1) var<uniform> view: mat4x4<f32>;

// model transform
@group(1) @binding(0) var<uniform> model_transform: mat4x4<f32>;

// material information
@group(2) @binding(0) var diffuse_texture: texture_2d<f32>;
@group(2) @binding(1) var diffuse_sampler: sampler;

// animation transforms
@group(3) @binding(0) var<storage> bone_transforms: array<mat4x4<f32>>;
@group(3) @binding(1) var<uniform> node_transform: mat4x4<f32>;


// Vertex shader

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
};

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {

    var result: VertexOutput;

    result.position = projection * view * model_transform * vec4<f32>(model.position, 1.0);
    result.tex_coords = model.tex_coords;

    return result;
}

// Fragment shader

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var color = textureSample(diffuse_texture, diffuse_sampler, in.tex_coords);
    return color;
}

