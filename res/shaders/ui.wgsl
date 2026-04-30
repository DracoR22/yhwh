struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) tex_coords: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>
}

struct UIUniform {
    model_matrix: mat4x4<f32>,
    ortho_projection: mat4x4<f32>
}

@group(0) @binding(0)
var ui_texture: texture_2d<f32>;
@group(0) @binding(1)
var ui_sampler: sampler;

@group(1) @binding(0)
var<uniform> ui: UIUniform;

@vertex
fn vs_main(vert_in: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    out.clip_position = ui.ortho_projection * ui.model_matrix * vec4<f32>(vert_in.position, 0.0, 1.0);
    out.tex_coords = vert_in.tex_coords;

    return out;
}

@fragment
fn fs_main(frag_in: VertexOutput) -> @location(0) vec4<f32> {
    let color = textureSample(ui_texture, ui_sampler, frag_in.tex_coords);
    return color;
}