struct VertexOutput {
    @location(0) tex_coords: vec2<f32>,
    @builtin(position) clip_position: vec4<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) vi: u32) -> VertexOutput {
    var out: VertexOutput;

    out.tex_coords = vec2<f32>(f32((vi << 1u) & 2u), f32(vi & 2u));
    out.clip_position = vec4<f32>(out.tex_coords * 2.0 - 1.0, 0.0, 1.0);
    out.tex_coords.y = 1.0 - out.tex_coords.y;

    return out;
}

struct FragmentOutput {
    @location(0) o_Target: vec4<f32>,
}

@group(0) @binding(0) var t_Color: texture_2d<f32>;
@group(0) @binding(1) var s_Color: sampler;

@fragment
fn fs_main(@location(0) tex_coords: vec2<f32>) -> FragmentOutput {
    let o_Target = textureSampleLevel(t_Color, s_Color, tex_coords, 0.0);
    return FragmentOutput(o_Target);
}