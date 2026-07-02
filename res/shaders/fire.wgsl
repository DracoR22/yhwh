struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) normal: vec3<f32>
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coord: vec2<f32>,
    @location(1) tex_coords1: vec2<f32>,
    @location(2) tex_coords2: vec2<f32>,
    @location(3) tex_coords3: vec2<f32>
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
    _pad0: vec2<f32>
}

struct GlobalUniform {
    scroll_speeds: vec4<f32>,
    scales: vec4<f32>,

    distortion1: vec2<f32>,
    distortion2: vec2<f32>,
    distortion3: vec2<f32>,

    distortion_scale: f32,
    distortion_bias: f32,
    time: f32,
    _pad0: f32,

    _pad1: f32,
    _pad2: f32
}

@group(0) @binding(0)
var fire_texture: texture_2d<f32>;
@group(0) @binding(1)
var fire_sampler: sampler;

@group(0) @binding(2)
var noise_texture: texture_2d<f32>;
@group(0) @binding(3)
var noise_sampler: sampler;

@group(0) @binding(4)
var alpha_texture: texture_2d<f32>;
@group(0) @binding(5)
var alpha_sampler: sampler;

@group(1) @binding(0)
var<uniform> camera: CameraUniform;
@group(2) @binding(0)
var<uniform> globals: GlobalUniform;
@group(3) @binding(0)
var<uniform> model: ModelUniform;

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    let world_position: vec4<f32> = model.model_matrix * vec4<f32>(in.position, 1.0);
    out.clip_position = camera.projection * camera.view * world_position;

    out.tex_coord = in.tex_coords;

    out.tex_coords1 = (in.tex_coords * globals.scales.x);
    out.tex_coords1.y = out.tex_coords1.y + (globals.time * globals.scroll_speeds.x);

    out.tex_coords2 = (in.tex_coords * globals.scales.y);
    out.tex_coords2.y = out.tex_coords2.y + (globals.time * globals.scroll_speeds.y);

    out.tex_coords3 = (in.tex_coords * globals.scales.z);
    out.tex_coords3.y = out.tex_coords3.y + (globals.time * globals.scroll_speeds.z);

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var out: vec4<f32>;
    var noise_coords: vec2<f32>;

    var noise1 = textureSample(noise_texture, noise_sampler, in.tex_coords1);
    var noise2 = textureSample(noise_texture, noise_sampler, in.tex_coords2);
    var noise3 = textureSample(noise_texture, noise_sampler, in.tex_coords3);

    // Move the noise from the (0, 1) range to the (-1, +1) range
    noise1 = (noise1 - 0.5) * 2.0;
    noise2 = (noise2 - 0.5) * 2.0;
    noise3 = (noise3 - 0.5) * 2.0;

    noise1.x = noise1.x * globals.distortion1.x;
    noise1.y = noise1.y * globals.distortion1.y;

    noise2.x = noise2.x * globals.distortion2.x;
    noise2.y = noise2.y * globals.distortion2.y;

    noise3.x = noise3.x * globals.distortion3.x;
    noise3.y = noise3.y * globals.distortion3.y;

    let final_noise: vec4<f32> = noise1 + noise2 + noise3;  

    let perturb = ((in.tex_coord.y) * globals.distortion_scale) + globals.distortion_bias;
    noise_coords.x = (final_noise.x * perturb) + in.tex_coord.x;
    noise_coords.y = (final_noise.y * perturb) + (1.0 - in.tex_coord.y);
    noise_coords.y = 1.0 - noise_coords.y;

    var fire_color = textureSample(fire_texture, fire_sampler, noise_coords);
    let alpha_color = textureSample(alpha_texture, alpha_sampler, noise_coords.xy);

    fire_color.a = alpha_color.r;
    out = fire_color;

    return out;
}