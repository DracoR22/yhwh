struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) tangent: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

struct FragmentOutput {
    @location(0) color: vec4<f32>,
    @location(1) emissive: vec4<f32>
}

struct ModelUniform {
    model_matrix: mat4x4<f32>,
    normal_matrix: mat4x4<f32>,
    tex_scale: vec2<f32>,
    _padding_0: vec2<f32>
}

struct CameraUniform {
    view: mat4x4<f32>,
    projection: mat4x4<f32>,
    view_position: vec4<f32>,
}

struct GlobalUniform {
    time: f32,
    pad_0: f32,
    pad_1: f32,
    pad_2: f32
}

struct FlameParamsUniform {
    random_seed: f32,
    pad_0: f32,
    pad_1: f32,
    pad_2: f32
}

@group(0) @binding(0)
var<uniform> global: GlobalUniform;

@group(1) @binding(0)
var<uniform> camera: CameraUniform;

@group(2) @binding(0)
var<uniform> model: ModelUniform;

@group(3) @binding(0)
var<uniform> flame: FlameParamsUniform;

fn hash(n: f32) -> f32 {
    return fract(sin(n) * 43758.5453123);
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    let time = global.time;
    let seed = flame.random_seed;
    var pos = in.position;

    let height_factor = 1.0 - clamp(in.tex_coords.y, 0.0, 1.0);
    let influence = pow(height_factor, 1.2);

    let seed1 = hash(seed);
    let seed2 = hash(seed + 13.7);
    let seed3 = hash(seed + 29.1);
    let seed4 = hash(seed + 51.4);

    let sway1_speed = 4.0 + seed1 * 4.0;
    let sway2_speed = 8.0 + seed2 * 8.0;
    let sway3_speed = 6.0 + seed3 * 5.0;

    let sway1_amp = 0.04 + seed2 * 0.04;
    let sway2_amp = 0.01 + seed3 * 0.03;
    let sway3_amp = 0.008 + seed4 * 0.02;

    let sway1 = sin(time * sway1_speed + pos.y * 8.0 + seed) * sway1_amp;
    let sway2 = sin(time * sway2_speed + pos.y * 17.0 + seed2 * 10.0) * sway2_amp;
    let sway3 = cos(time * sway3_speed + pos.y * 13.0 + seed3 * 8.0) * sway3_amp;

    let turbulence = (hash(floor(time * 12.0)) - 0.5) * 0.01;

    pos.x += (sway1 + sway2 + turbulence) * influence;
    pos.z += sway3 * influence;

    let lean_speed = 0.8 + seed1 * 4.5;
    let lean_amount = 0.3 + seed2 * 0.45;
    let lean = sin(time * lean_speed + seed) * lean_amount;
    pos.x += lean * height_factor;

    let stretch_speed = 10.0 + seed1 * 7.0;;
    let stretch_amount = 0.01 + seed2 * 0.005;
    let stretch = 1.0 + sin(time * stretch_speed) * stretch_amount;
    pos.y *= mix(1.0, stretch, influence);

    let tip = pow(height_factor, 5.0);
    pos.x += sin(time * 45.0 + pos.y * 80.0) * 0.02 * tip;

    let world_position: vec4<f32> = model.model_matrix * vec4<f32>(pos, 1.0);

    out.tex_coords = in.tex_coords;
    out.clip_position = camera.projection * camera.view * world_position;

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> FragmentOutput {
    var out: FragmentOutput;

    out.color = vec4<f32>(1.0, 1.0, 1.0, 1.0);
    out.emissive = vec4<f32>(1.0, 0.6, 0.0, 1.0);

    return out;
}