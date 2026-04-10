struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) tangent: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) tangent: vec3<f32>,
    @location(3) bitangent: vec3<f32>,
    @location(4) world_position: vec4<f32>
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

@group(1) @binding(0)
var<uniform> camera: CameraUniform;

@group(2) @binding(0)
var<uniform> model: ModelUniform;

@group(3) @binding(0)
var<storage, read> lights: array<LightUniform>;

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

fn get_direct_lighting(light_pos: vec3<f32>, light_color: vec3<f32>, light_strength: f32, light_radius: f32, world_pos: vec3<f32>, view_pos: vec3<f32>, normal: vec3<f32>, base_color: vec3<f32>, metallic: f32, roughness: f32) -> vec3<f32> {
    // let l = normalize(light_pos - world_pos);
    // let v = normalize(view_pos - world_pos);

    // let distance = length(light_pos - world_pos);
    // let nd = distance / light_radius;
    // let attenuation = 1.0 / (nd * nd + 1.0);
    // let radiance = light_color * attenuation * light_strength;
    // let ndl = max(dot(normal, l), 0.0);

    // let brdf = microfacet_brdf(l, v, normal, base_color, metallic, 1.0, roughness) * radiance * ndl;
    // return brdf;

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

@vertex
fn vs_main(vert_in: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    let world_position: vec4<f32> = model.model_matrix * vec4<f32>(vert_in.position, 1.0);

    out.clip_position = camera.projection * camera.view * world_position;

    out.tex_coords = vert_in.tex_coords;
    out.normal = normalize((model.normal_matrix * vec4<f32>(vert_in.normal, 0.0)).xyz);
    out.tangent = normalize((model.normal_matrix * vec4<f32>(vert_in.tangent, 0.0)).xyz);
    out.bitangent = normalize(cross(out.normal, out.tangent));

    out.world_position = world_position;

    return out;
}

@fragment
fn fs_main(frag_in: VertexOutput) -> @location(0) vec4<f32> {
    let base_color = textureSample(base_color_texture, base_color_sampler, frag_in.tex_coords);
    var normal_map = textureSample(normal_texture, normal_sampler, frag_in.tex_coords).rgb;
    let rma = textureSample(rma_texture, rma_sampler, frag_in.tex_coords).rgb;

    normal_map = mix(normal_map, vec3<f32>(0.5, 0.5, 1.0), 0.7);
    
    let tbn = mat3x3(normalize(frag_in.tangent), normalize(frag_in.bitangent), normalize(frag_in.normal));
    normal_map = normal_map * 2.0 - 1.0;
    normal_map = normalize(normal_map);

    let normal = normalize(tbn *(normal_map));

    let gamma_base_color = pow(base_color.rgb, vec3<f32>(2.2));
    let roughness = rma.r;
    let metallic = rma.g;

    var direct_lighting = vec3<f32>(0.0);

    for (var i = 0u; i < 8; i = i + 1u) {
        let light = lights[i];

        let light_position = light.position;
        let light_color = light.color;
        let light_strength = light.strength;
        let light_radius = light.radius * 2;

        direct_lighting += get_direct_lighting(light_position, light_color, light_strength, light_radius, frag_in.world_position.xyz, camera.view_position.xyz, normal.xyz, gamma_base_color.rgb, metallic, roughness);

        let to_light = light_position - frag_in.world_position.xyz;
        let dist = length(to_light);
        let nd = dist / light_radius;
        let light_dir = to_light / dist;
        let view_dir = normalize(camera.view_position.xyz - frag_in.world_position.xyz);
        let att = 1.0 / (nd * nd + 1.0) * light_strength;

        direct_lighting += vec3<f32>(roughness * roughness * 0.01 * att) * light_color;
    }


    return vec4<f32>(direct_lighting, 1.0);
}