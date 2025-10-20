struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) tangent: vec3<f32>,
    @location(5) joints: vec4<u32>,
    @location(6) weights: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>
}

struct CameraUniform {
    view: mat4x4<f32>,
    projection: mat4x4<f32>,
    view_position: vec4<f32>,
}

struct ModelUniform {
    model_matrix: mat4x4<f32>
}

struct SkinUniform {
   joint_matrices: array<mat4x4<f32>, 512>
}

@group(0) @binding(0)
var texture_albedo: texture_2d<f32>;
@group(0) @binding(1)
var sampler_albedo: sampler;

@group(0) @binding(2)
var texture_normal_map: texture_2d<f32>;
@group(0) @binding(3)
var sampler_normal_map: sampler;

@group(1) @binding(0)
var<uniform> camera: CameraUniform;

@group(2) @binding(0)
var<uniform> model: ModelUniform;

@group(3) @binding(0)
var<uniform> skin: SkinUniform;

@vertex
fn vs_main(vert_in: VertexInput) -> VertexOutput {
   var out: VertexOutput;

   var total_position: vec4<f32> = vec4<f32>(0.0);

   for (var i = 0u; i < 4u; i++) {
    let joint_index = vert_in.joints[i];
    let weight = vert_in.weights[i];
    let joint_matrix = skin.joint_matrices[joint_index];

    total_position += weight * (joint_matrix * vec4<f32>(vert_in.position, 1.0));
   }

   out.tex_coords = vert_in.tex_coords;
   out.clip_position = camera.projection * camera.view * model.model_matrix * total_position;
   return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
  let albedo: vec4<f32> = textureSample(texture_albedo, sampler_albedo, in.tex_coords);

  return albedo;
}