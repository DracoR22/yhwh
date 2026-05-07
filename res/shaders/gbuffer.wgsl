struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) tangent: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) world_position: vec3<f32>,
    @location(2) tangent: vec3<f32>,
    @location(3) bitangent: vec3<f32>,
    @location(4) normal: vec3<f32>, 
}

struct FragmentOutput {
    @location(0) base_color: vec4<f32>,
    @location(1) normal: vec4<f32>,
    @location(2) rma: vec4<f32>,
    @location(3) world_pos: vec4<f32>,
}

struct CameraUniform {
    view: mat4x4<f32>,
    projection: mat4x4<f32>,
    view_position: vec4<f32>,
}

struct ModelUniform {
    model_matrix: mat4x4<f32>,
    normal_matrix: mat4x4<f32>,
    tex_scale: vec2<f32>,
    _padding_0: vec2<f32>
}

@group(0) @binding(0)
var base_color_texture: texture_2d<f32>;
@group(0) @binding(1)
var base_color_sampler: sampler;

@group(0) @binding(2)
var normal_texture: texture_2d<f32>;
@group(0) @binding(3)
var normal_sampler: sampler;

@group(0) @binding(4)
var rma_texture: texture_2d<f32>;
@group(0) @binding(5)
var rma_sampler: sampler;

@group(1) @binding(0)
var<uniform> camera: CameraUniform;

@group(2) @binding(0)
var<uniform> model: ModelUniform;

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    let world_position: vec4<f32> = model.model_matrix * vec4<f32>(in.position, 1.0);

    let n = normalize((model.normal_matrix * vec4<f32>(in.normal, 0.0)).xyz);
    var t = normalize((model.normal_matrix * vec4<f32>(in.tangent, 0.0)).xyz);
    t = normalize(t - dot(t, n) * n);
    let b = cross(n, t);

    out.tex_coords =  model.tex_scale * in.tex_coords;
    out.world_position = world_position.xyz;
    out.normal = n;
    out.tangent = t;
    out.bitangent = b;
    out.clip_position = camera.projection * camera.view * world_position;

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> FragmentOutput {
    var out: FragmentOutput;

    let base_color = textureSample(base_color_texture, base_color_sampler, in.tex_coords);
    let rma = textureSample(rma_texture, rma_sampler, in.tex_coords);
    let tangent_normal: vec3<f32> = textureSample(normal_texture, normal_sampler, in.tex_coords).xyz * 2.0 - 1.0;
    let world_normal = normalize(mat3x3<f32>(in.tangent, in.bitangent, in.normal) * tangent_normal);

    out.base_color = base_color;
    out.normal = vec4<f32>(world_normal, 1.0);
    out.rma = rma;
    out.world_pos = vec4<f32>(in.world_position, 1.0);

    return out;
}