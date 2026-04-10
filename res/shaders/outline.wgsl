struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>
}
@group(0) @binding(0)
var mask_texture: texture_2d<f32>;
@group(0) @binding(1)
var mask_sampler: sampler;

@vertex
fn vs_main(vert_in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(vert_in.position.x, vert_in.position.y, 0.0, 1.0);
    out.tex_coords = vert_in.tex_coords;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // TODO: pass as uniform!!
    let resolution = vec2<f32>(1920.0, 1080.0);
    
    let uv = vec2<f32>(in.tex_coords.x, 1.0 - in.tex_coords.y);
    let texel = vec2<f32>(1.0 / resolution.x, 1.0 / resolution.y);

    let center = textureSample(mask_texture, mask_sampler, uv).r;
    let thickness = 2.0;

    let right  = textureSample(mask_texture, mask_sampler, uv + vec2<f32>( texel.x * thickness, 0.0)).r;
    let left   = textureSample(mask_texture, mask_sampler, uv + vec2<f32>(-texel.x * thickness, 0.0)).r;
    let up     = textureSample(mask_texture, mask_sampler, uv + vec2<f32>(0.0,  texel.y * thickness)).r;
    let down   = textureSample(mask_texture, mask_sampler, uv + vec2<f32>(0.0, -texel.y * thickness)).r;

    let maxNeighbor = max(max(right, left), max(up, down));

    let outline = max(maxNeighbor - center, 0.0);

    return vec4<f32>(outline, outline, outline, 1.0);
}