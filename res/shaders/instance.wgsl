struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) tangent: vec3<f32>,
    @location(4) bitangent: vec3<f32>,
    @location(5) joints: vec4<u32>,
    @location(6) weights: vec4<f32>,
}

struct InstanceInput {
    @location(7) model_matrix_0: vec4<f32>,
    @location(8) model_matrix_1: vec4<f32>,
    @location(9) model_matrix_2: vec4<f32>,
    @location(10) model_matrix_3: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) world_position: vec3<f32>,
    @location(2) tbn_0: vec3<f32>, // tangent
    @location(3) tbn_1: vec3<f32>, // bitangent
    @location(4) tbn_2: vec3<f32>, // normal
};

// uniforms
struct CameraUniform {
    view: mat4x4<f32>,
    projection: mat4x4<f32>,
    view_position: vec4<f32>,
}

@group(1) @binding(0)
var<uniform> camera: CameraUniform;

@vertex
fn vs_main(model: VertexInput, instance: InstanceInput) -> VertexOutput {
    var out: VertexOutput;

     let model_matrix = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
    );

    out.tex_coords = model.tex_coords;
    
    let world_position: vec4<f32> = model_matrix * vec4<f32>(model.position, 1.0);
    out.world_position = world_position.xyz;

    let N = normalize((model_matrix * vec4<f32>(model.normal, 0.0)).xyz);
    var T = normalize((model_matrix * vec4<f32>(model.tangent, 0.0)).xyz);
    T = normalize(T - dot(T, N) * N);
    let B = cross(N, T);

    out.tbn_0 = T;
    out.tbn_1 = B;
    out.tbn_2 = N;

    out.clip_position = camera.projection * camera.view * world_position;

    return out;
}

// uniforms
@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

@group(0) @binding(2)
var t_normal: texture_2d<f32>;
@group(0) @binding(3)
var s_normal: sampler;

struct LightUniform {
    position: vec3<f32>,
    color: vec3<f32>, 
}
@group(2) @binding(0)
var<uniform> light: LightUniform;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let diffuse_texture: vec4<f32> = textureSample(t_diffuse, s_diffuse, in.tex_coords);
    var tangent_normal: vec3<f32> = textureSample(t_normal, s_normal, in.tex_coords).xyz * 2.0 - 1.0;

    let world_normal = normalize(mat3x3<f32>(in.tbn_0, in.tbn_1, in.tbn_2) * tangent_normal);
    let ambient_strength = 0.1;
    let ambient_color = light.color * ambient_strength;

    let light_dir = normalize(light.position - in.world_position);
    let diffuse_strength = max(dot(world_normal, light_dir), 0.0);
    let diffuse_color = light.color * diffuse_strength;

    // specular
    let view_dir = normalize(camera.view_position.xyz - in.world_position);
    let half_dir = normalize(view_dir + light_dir);

    let specular_strength = pow(max(dot(world_normal, half_dir), 0.0), 32.0);
    let specular_color = specular_strength * light.color;

    let result = (ambient_color + diffuse_color + specular_color) * diffuse_texture.xyz;

    return vec4<f32>(result, 1.0);
}