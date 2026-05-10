struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) tex_coords: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

struct CameraUniform {
    view: mat4x4<f32>,
    projection: mat4x4<f32>,
    view_position: vec4<f32>,
}

struct LightUniform {
    position: vec3<f32>,
    _pad0: u32,
    color: vec3<f32>, 
    _pad1: u32,
    strength: f32,
    radius: f32,
    _pad2: u32,
    _pad3: u32
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

@group(0) @binding(6)
var world_position_texture: texture_2d<f32>;
@group(0) @binding(7) 
var world_position_sampler: sampler;

@group(0) @binding(8)
var ssao_texture: texture_2d<f32>;
@group(0) @binding(9)
var ssao_sampler: sampler;

@group(1) @binding(0)
var<uniform> camera: CameraUniform;

@group(2) @binding(0)
var<storage, read> lights: array<LightUniform>;
@group(2) @binding(1)
var<uniform> light_count: u32;

@group(2) @binding(2)
var shadow_maps: texture_depth_cube_array;
@group(2) @binding(3)
var shadow_sampler: sampler_comparison;

const gridSamplingDisk: array<vec3<f32>, 20> = array(
   vec3<f32>(1, 1,  1), vec3<f32>( 1, -1,  1), vec3<f32>(-1, -1,  1), vec3<f32>(-1, 1,  1),
   vec3<f32>(1, 1, -1), vec3<f32>( 1, -1, -1), vec3<f32>(-1, -1, -1), vec3<f32>(-1, 1, -1),
   vec3<f32>(1, 1,  0), vec3<f32>( 1, -1,  0), vec3<f32>(-1, -1,  0), vec3<f32>(-1, 1,  0),
   vec3<f32>(1, 0,  1), vec3<f32>(-1,  0,  1), vec3<f32>( 1,  0, -1), vec3<f32>(-1, 0, -1),
   vec3<f32>(0, 1,  1), vec3<f32>( 0, -1,  1), vec3<f32>( 0, -1, -1), vec3<f32>( 0, 1, -1)

);

fn shadow_calculation(
    light_index: u32,
    light_pos: vec3<f32>,
    light_radius: f32,
    frag_pos: vec3<f32>,
    view_pos: vec3<f32>,
    normal: vec3<f32>
) -> f32 {
    var shadow = 0.0;
    let samples = 20;

    let light_dir = frag_pos - light_pos;
    let distance = length(light_dir);
    let current_depth = distance / light_radius;

    if (distance > light_radius) {
        return 1.0;
    }

    let bias = max(0.0125 * (1.0 - dot(normal, normalize(light_dir))), 0.00125); 

    let view_distance = length(view_pos - frag_pos);
    let disk_radius = (1.0 + (view_distance / light_radius)) / 200.0;
    //let disk_radius = 0.01 * current_depth;
    //let disk_radius = 0.01 * current_depth * (distance / light_radius);

    for (var i: i32 = 0; i < samples; i = i + 1) {
        let offset = normalize(gridSamplingDisk[i]) * disk_radius;
        let sample_dir = normalize(normalize(light_dir) + offset);

        shadow += textureSampleCompare(
            shadow_maps,
            shadow_sampler,
            sample_dir,
            light_index,
            current_depth - bias
        );
    }

    shadow /= f32(samples);

    return shadow;
}

const PI = 3.14159265359;

fn fresnel_schlick(cos_theta: f32, f0: vec3<f32>) -> vec3<f32> {
    return f0 + (1.0 - f0) * pow(clamp(1.0 - cos_theta, 0.0, 1.0), 5.0);
}

fn distribution_ggx(n: vec3<f32>, h: vec3<f32>, roughness: f32) -> f32 {
    let a = roughness * roughness;
    let a2 = a * a;
    let ndoth = max(dot(n, h), 0.0);
    let ndothh2 = ndoth * ndoth;

    let num = a2;
    var denom = (ndothh2 * (a2 - 1.0) + 1.0);
    denom = PI * denom * denom;

    return num / denom;
}

fn geometry_schlick_ggx(ndotv: f32, roughness: f32) -> f32 {
    let r = (roughness + 1.0);
    let k = (r * r) / 8.0;

    let num = ndotv;
    let denom = ndotv * (1.0 - k) + k;

    return num / denom;
}

fn geometry_smith(n: vec3<f32>, v: vec3<f32>, l: vec3<f32>, roughness: f32) -> f32 {
    let ndotv = max(dot(n, v), 0.0);
    let ndotl = max(dot(n, l), 0.0);
    let ggx2 = geometry_schlick_ggx(ndotv, roughness);
    let ggx1 = geometry_schlick_ggx(ndotl, roughness);

    return ggx1 * ggx2;
}

fn microfacet_brdf(l: vec3<f32>, v: vec3<f32>, n: vec3<f32>, base_color: vec3<f32>, metallic: f32, fresnel_reflect: f32, roughness: f32) -> vec3<f32> {
    let h = normalize(v + l);
    let lo = vec3<f32>(0.0);

    var f0 = vec3<f32>(0.04 * fresnel_reflect);
    f0 = mix(f0, base_color, metallic);

    let f = fresnel_schlick(max(dot(h, v), 0.0), f0);
    let ndf = distribution_ggx(n, h, roughness);
    let g = geometry_smith(n, v, l, roughness);

    let numerator = ndf * g * f;
    let denominator = 4.0 * max(dot(n, v), 0.0) * max(dot(n, l), 0.0) + 0.0001;
    let specular = numerator / denominator;

    let ks = f;
    var kd = vec3<f32>(1.0) - ks;
    kd *= 1.0 - metallic;

    let ndotl = max(dot(n, l), 0.0);

    return (kd * base_color / PI + specular);
}

@vertex
fn vs_main(@builtin(vertex_index) vi: u32) -> VertexOutput {
    var out: VertexOutput;

    out.tex_coords = vec2<f32>(f32((vi << 1u) & 2u), f32(vi & 2u));
    out.clip_position = vec4<f32>(out.tex_coords * 2.0 - 1.0, 0.0, 1.0);
    out.tex_coords.y = 1.0 - out.tex_coords.y;

    return out;
}

fn get_spot_light_lighting(light_pos: vec3<f32>, light_color: vec3<f32>, light_strength: f32, light_radius: f32, world_pos: vec3<f32>, view_pos: vec3<f32>, normal: vec3<f32>, base_color: vec3<f32>, metallic: f32, roughness: f32) -> vec3<f32> {
    let to_light = light_pos - world_pos;
    let dist = length(to_light);
    let nd = dist / light_radius;
    let light_dir = to_light / dist;
    let view_dir = normalize(view_pos - world_pos);
    let att = 1.0 / (nd * nd + 1.0);
    let ndl = max(dot(normal, light_dir), 0.0);
    let radiance = light_color * att * light_strength;
    let brdf = microfacet_brdf(light_dir, view_dir, normal, base_color, metallic, 1.0, roughness);
    return brdf * radiance * ndl;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let base_color: vec3<f32> = textureSample(base_color_texture, base_color_sampler, in.tex_coords).rgb;
    let normal = textureSample(normal_texture, normal_sampler, in.tex_coords).rgb;
    let rma = textureSample(rma_texture, rma_sampler, in.tex_coords);
    let world_position = textureSample(world_position_texture, world_position_sampler, in.tex_coords).xyz;
    let ssao = textureSample(ssao_texture, ssao_sampler, in.tex_coords).r;

    let roughness = rma.r;
    let metallic = rma.g;
    let ao = rma.b;

    var final_color = vec3<f32>(0.0);

    for (var i: u32 = 0u; i < light_count; i = i + 1u) {
        let light = lights[i];

        let lighting = get_spot_light_lighting(light.position, light.color, light.strength, light.radius, world_position, camera.view_position.xyz, normal, base_color, metallic, roughness);
        let shadow = shadow_calculation(i, light.position, light.radius, world_position, camera.view_position.xyz, normal);

        final_color += lighting * shadow;
    }

    let ambient = 0.05 * base_color * ao * ssao;
    final_color += ambient;

    return vec4<f32>(final_color, 1.0);
}