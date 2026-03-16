struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
}

struct CameraUniform {
    view: mat4x4<f32>,
    projection: mat4x4<f32>,
    view_position: vec4<f32>,
}

struct ModelUniform {
    model_matrix: mat4x4<f32>
}

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

@group(1) @binding(0)
var<uniform> model: ModelUniform;

@vertex
fn vs_main(vert_in: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    let world_position = model.model_matrix * vec4<f32>(vert_in.position, 1.0);
    out.clip_position = camera.projection * camera.view * world_position;

    return out;
}

@fragment
fn fs_main(fs_in: VertexOutput) -> @location(0) vec4<f32> {

    return vec4<f32>(1.0, 1.0, 1.0, 1.0);
}