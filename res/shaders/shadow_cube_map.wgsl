struct VertexInput {
    @location(0) position: vec3<f32>
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>
}

struct ShadowUniform {
    light_matrix: mat4x4<f32>,
    light_pos_x: f32,
    light_pos_y: f32,
    light_pos_z: f32,
    far_plane: f32,
};

struct ModelUniform {
    model_matrix: mat4x4<f32>,
    normal_matrix: mat3x4<f32>,
    tex_scale: vec2<f32>,
    _padding_0: vec2<f32>
}

@group(0) @binding(0)
var<uniform> shadow: ShadowUniform;

@group(1) @binding(0)
var<uniform> model: ModelUniform;

@vertex
fn vs_main(vert_in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    let world_position = model.model_matrix * vec4<f32>(vert_in.position, 1.0);
    out.clip_position = shadow.light_matrix * world_position;
    out.world_position = world_position.xyz;

    return out;
}

@fragment
fn fs_main(frag_in: VertexOutput) -> @builtin(frag_depth) f32 {
    let light_distance = distance(frag_in.world_position, vec3<f32>(shadow.light_pos_x, shadow.light_pos_y, shadow.light_pos_z));

    let depth = light_distance / shadow.far_plane;

    return depth;
}