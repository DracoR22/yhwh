struct VertexInput {
    @location(0) position: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec3<f32>,
};

struct CameraUniform {
    view: mat4x4<f32>,
    projection: mat4x4<f32>,
    view_position: vec4<f32>,
}

@group(1) @binding(0)
var<uniform> camera: CameraUniform;

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    out.tex_coords = model.position;

    var rot_view = camera.view;
    rot_view[3] = vec4<f32>(0.0, 0.0, 0.0, 1.0);

    let pos: vec4<f32> = camera.projection * rot_view * vec4<f32>(model.position, 1.0);

    out.clip_position = pos.xyww;

    return out;
}

@group(0) @binding(0)
var t_cubemap: texture_cube<f32>;
@group(0) @binding(1)
var s_cubemap: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(t_cubemap, s_cubemap, in.tex_coords);
}