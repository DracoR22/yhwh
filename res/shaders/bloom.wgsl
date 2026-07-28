struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
}

struct MaskUniform {
    color: vec4<f32>,
    emissive: vec4<f32>
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
var<uniform> mask: MaskUniform;

@group(1) @binding(0)
var<uniform> camera: CameraUniform;

@group(2) @binding(0)
var<uniform> model: ModelUniform;

@vertex
fn vs_main(vert_in: VertexInput) -> VertexOutput {
   var out: VertexOutput;

   let world_position = model.model_matrix * vec4<f32>(vert_in.position, 1.0);

   out.clip_position = camera.projection * camera.view * world_position;
   return out;
}

struct FragmentOutput {
    @location(0) color: vec4<f32>,
    @location(1) emissive: vec4<f32>
}

@fragment
fn fs_main(frag_in: VertexOutput) -> FragmentOutput {
    var out: FragmentOutput;

    out.color = mask.color;
    out.emissive = mask.emissive;

    return out;
}