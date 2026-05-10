struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>
}

@group(0) @binding(0)
var ssao_color_texture: texture_2d<f32>;
@group(0) @binding(1)
var ssao_color_sampler: sampler;


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
    let texel_size = 1.0 / vec2<f32>(textureDimensions(ssao_color_texture, 0));
    var result = 0.0;

    for (var x = -2; x <= 2; x++) {
        for (var y = -2; y <= 2; y++) {
            let offset = vec2<f32>(f32(x), f32(y)) * texel_size;
            result += textureSample(ssao_color_texture, ssao_color_sampler, in.tex_coords + offset).r;
        }
    }
    return result / 25.0;
}
