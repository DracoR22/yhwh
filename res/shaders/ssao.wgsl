struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>
}

struct CameraUniform {
    view: mat4x4<f32>,
    projection: mat4x4<f32>,
    view_position: vec4<f32>,
};

struct SSAOUniform {
    kernel_samples: array<vec4<f32>, 16>,
};

@group(0) @binding(0)
var position_texture: texture_2d<f32>;
@group(0) @binding(1)
var position_sampler: sampler;

@group(0) @binding(2)
var normal_texture: texture_2d<f32>;
@group(0) @binding(3)
var normal_sampler: sampler;

@group(0) @binding(4)
var noise_texture: texture_2d<f32>;
@group(0) @binding(5)
var noise_sampler: sampler;

@group(1) @binding(0)
var<uniform> camera: CameraUniform;

@group(2) @binding(0)
var<uniform> ssao: SSAOUniform;

const KERNEL_SIZE: i32 = 16;
const RADIUS: f32 = 0.5;
const BIAS: f32 = 0.025;
const NOISE_SCALE = vec2<f32>(1920.0 / 4.0, 1080.0 / 4.0);

@vertex
fn vs_main(@builtin(vertex_index) vi: u32) -> VertexOutput {
    var out: VertexOutput;

    out.tex_coords = vec2<f32>(f32((vi << 1u) & 2u), f32(vi & 2u));
    out.clip_position = vec4<f32>(out.tex_coords * 2.0 - 1.0, 0.0, 1.0);
    out.tex_coords.y = 1.0 - out.tex_coords.y;

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) f32 {
    let world_position = textureSample(position_texture, position_sampler, in.tex_coords).xyz;
    let world_normal = textureSample(normal_texture, normal_sampler, in.tex_coords).xyz;
    let normal = normalize((camera.view * vec4<f32>(world_normal, 0.0)).xyz);

    let random_vec = textureSample(noise_texture, noise_sampler, in.tex_coords * NOISE_SCALE).xyz;

    let position = (camera.view * vec4<f32>(world_position, 1.0)).xyz;

    let tangent = normalize(random_vec - normal * dot(random_vec, normal));
    let bitangent = cross(normal, tangent);
    let tbn = mat3x3<f32>(tangent, bitangent, normal);

    var occlusion = 0.0;
    for (var i = 0; i < KERNEL_SIZE; i++) {
        var sample_position = tbn * ssao.kernel_samples[i].xyz;
        sample_position = position + sample_position * RADIUS;

        let offset = camera.projection * vec4<f32>(sample_position, 1.0);
        let projected = offset.xyz / offset.w;
        // let sample_uv = projected.xy * 0.5 + 0.5; // opengl coords
        let sample_uv = projected.xy * vec2<f32>(0.5, -0.5) + 0.5; // wgpu coords

        // if (sample_uv.x < 0.0 || sample_uv.x > 1.0 || sample_uv.y < 0.0 || sample_uv.y > 1.0) {
        //   continue;
        // }
        
        let sample_world_position = textureSample(position_texture, position_sampler, sample_uv).xyz;
        let sample_depth =  (camera.view * vec4<f32>(sample_world_position, 1.0)).z;
        let range_check = smoothstep(0.0, 1.0, RADIUS / abs(position.z - sample_depth));

        if (sample_depth >= sample_position.z + BIAS) {
            occlusion += range_check;
        } else {
            occlusion += 0.0;
        }
    }

    occlusion = 1.0 - (occlusion / f32(KERNEL_SIZE));
    
    let strength = 3.0;
    occlusion = pow(occlusion, strength);
    
    return occlusion;
}