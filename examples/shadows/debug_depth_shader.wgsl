//#define_import_path spark::debug_depth_shader

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
};


@group(0) @binding(0) var<uniform> projection_view: mat4x4<f32>;

@group(1) @binding(0) var<uniform> model_transform: mat4x4<f32>;


// shadow texture
@group(2) @binding(0) var texture: texture_depth_2d_array;
@group(2) @binding(1) var texture_sampler: sampler;
//@group(1) @binding(1) var texture_sampler: sampler_comparison;


@vertex fn vs_main(vertex_input: VertexInput) -> VertexOutput {

    var result: VertexOutput;

//    result.position = camera.projection * camera.view * model_transform * vec4<f32>(vertex_input.position, 1.0);
    result.position = projection_view * model_transform * vec4<f32>(vertex_input.position, 1.0);
    result.tex_coords = vertex_input.tex_coords;

    return result;
}

@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
//    let texCoord = coord.xy * 0.5 + 0.5;  // Transform from [-1, 1] to [0, 1] range
//    let depth = result.z;

    let color_1 = vec4<f32>(0.5, 1.0, 0.5, 1.0);
    let color_2 = vec4<f32>(1.0, 0.5, 0.5, 1.0);
    let flip_correction = vec2<f32>(1.0, -1.0);

    let tex_coords = in.tex_coords * flip_correction + vec2<f32>(0.0, 1.0);

    var value = textureSample(texture, texture_sampler, tex_coords, 0);
//    if (value < 1.0) {
//        value = value * 0.9 ;
//    } else if (value < 0.5) {
//        value = value * 0.5 ;
//    } else if (value < 0.2) {
//        value = value * 0.2 ;
//    }

    if (value < 0.93) {
        value = 0.0;
    }


    let first = vec4<f32>(value, value, value, 1.0);

    value = textureSample(texture, texture_sampler, tex_coords, 1);
    if (value < 1.0) {
        value = value * 0.5 ;
    }
    let second = vec4<f32>(value, value, value, 1.0);

    return first; //  * color_1;
//    return second * color_2;
//    return first * color_1 + second * color_2;
}

