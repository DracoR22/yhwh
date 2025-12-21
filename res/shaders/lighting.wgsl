struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) tangent: vec3<f32>,
    @location(4) bitangent: vec3<f32>,
    @location(5) joints: vec4<u32>,
    @location(6) weights: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) world_position: vec3<f32>,
    @location(2) tangent: vec3<f32>, // tangent
    @location(3) bitangent: vec3<f32>, // bitangent
    @location(4) normal: vec3<f32>, // normal
}

struct CameraUniform {
    view: mat4x4<f32>,
    projection: mat4x4<f32>,
    view_position: vec4<f32>,
}

struct LightUniform {
    position: vec3<f32>,
    color: vec3<f32>, 
}

struct ModelUniform {
    model_matrix: mat4x4<f32>,
    normal_matrix: mat3x4<f32>,
    tex_scale: vec2<f32>,
    _padding_0: vec2<f32>
}

@group(0) @binding(0)
var t_base_color: texture_2d<f32>;
@group(0) @binding(1)
var s_base_color: sampler;

@group(0) @binding(2)
var t_normal_map: texture_2d<f32>;
@group(0) @binding(3)
var s_normal_map: sampler;

@group(0) @binding(4)
var t_rma_map: texture_2d<f32>;
@group(0) @binding(5)
var s_rma_map: sampler;

@group(1) @binding(0)
var<uniform> camera: CameraUniform;

@group(2) @binding(0)
var<uniform> model: ModelUniform;

@group(3) @binding(0)
var<uniform> light: LightUniform;

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

    return (kd * base_color / PI + specular) * ndotl;
}

@vertex
fn vs_main(vert_in: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    out.tex_coords = model.tex_scale * vert_in.tex_coords;
    
    let world_position: vec4<f32> = model.model_matrix * vec4<f32>(vert_in.position, 1.0);
    out.world_position = world_position.xyz;

    let N = normalize((model.model_matrix * vec4<f32>(vert_in.normal, 0.0)).xyz);
    var T = normalize((model.model_matrix * vec4<f32>(vert_in.tangent, 0.0)).xyz);
    T = normalize(T - dot(T, N) * N);
    let B = cross(N, T);

    out.tangent = T;
    out.bitangent = B;
    out.normal = N;

    out.clip_position = camera.projection * camera.view * world_position;

    return out;
}

fn get_spot_light_lighting(light_pos: vec3<f32>, light_color: vec3<f32>, light_strength: f32, light_radius: f32, world_pos: vec3<f32>, view_pos: vec3<f32>, normal: vec3<f32>, base_color: vec3<f32>, metallic: f32, roughness: f32) -> vec3<f32> {
    let l = normalize(light_pos - world_pos);
    let v = normalize(view_pos - world_pos);

    let distance = length(light_pos - world_pos);
    let d = length(light_pos - world_pos);
    let nd = d / light_radius;
    let attenuation = 1.0 / (nd * nd + 1.0);
    let radiance = light_color * attenuation * light_strength;

    let brdf = microfacet_brdf(l, v, normal, base_color, metallic, 1.0, roughness) * radiance;
    return brdf;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let base_texture: vec4<f32> = textureSample(t_base_color, s_base_color, in.tex_coords);

    let albedo: vec3<f32> = pow(textureSample(t_base_color, s_base_color, in.tex_coords).rgb, vec3<f32>(2.2));
    let tangent_normal: vec3<f32> = textureSample(t_normal_map, s_normal_map, in.tex_coords).xyz * 2.0 - 1.0;
    let world_normal = normalize(mat3x3<f32>(in.tangent, in.bitangent, in.normal) * tangent_normal);
    let rma = textureSample(t_rma_map, s_rma_map, in.tex_coords);

    let roughness = rma.r;
    let metallic = rma.g;
    let ao = rma.b;

    var final_color = vec3<f32>(0.0);

    // TODO: DO MULTIPLE LIGHTS!!
    let light_strength = 50.0;
    let light_radius = 10.0;
    final_color += get_spot_light_lighting(light.position, light.color, light_strength, light_radius, in.world_position, camera.view_position.xyz, world_normal, albedo, metallic, roughness);
    //final_color *= ao;
    return vec4<f32>(final_color, 1.0);

    // let ambient_strength = 0.1;
    // let ambient_color = light.color * ambient_strength;

    // let light_dir = normalize(light.position - in.world_position);
    // let diffuse_strength = max(dot(world_normal, light_dir), 0.0);
    // let diffuse_color = light.color * diffuse_strength;

    // // specular
    // let view_dir = normalize(camera.view_position.xyz - in.world_position);
    // let half_dir = normalize(view_dir + light_dir);

    // let specular_strength = pow(max(dot(world_normal, half_dir), 0.0), 32.0);
    // let specular_color = specular_strength * light.color;

    // let result = (ambient_color + diffuse_color + specular_color) * base_texture.xyz;

    // return vec4<f32>(result, 1.0);
}