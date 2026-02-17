struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) tex_coords: vec2<f32>
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>
}

struct BlurUniform {
    direction: vec2<f32>, // horizontal: (1, 0) vertical: (0, 1)
    sample_distance: f32,
    _pad: f32
}

@group(0) @binding(0)
var image_texture: texture_2d<f32>;
@group(0) @binding(1)
var image_sampler: sampler;

@group(1) @binding(0)
var<uniform> params: BlurUniform;

const weight: array<f32, 5> = array<f32, 5>(0.227027, 0.1945946, 0.1216216, 0.054054, 0.016216);  

@vertex
fn vs_main(@builtin(vertex_index) vi: u32,vert_in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    // out.clip_position = vec4<f32>(vert_in.position.x, vert_in.position.y, 0.0, 1.0);
    // out.tex_coords = vert_in.tex_coords;

        out.tex_coords = vec2<f32>(
        f32((vi << 1u) & 2u),
        f32(vi & 2u),
    );
    out.clip_position = vec4<f32>(out.tex_coords * 2.0 - 1.0, 0.0, 1.0);
    // We need to invert the y coordinate so the image
    // is not upside down
    out.tex_coords.y = 1.0 - out.tex_coords.y;

    return out;
}

@fragment
fn fs_main(frag_in: VertexOutput) -> @location(0) vec4<f32> {
    let texSize = vec2<f32>(textureDimensions(image_texture));
    let tex_offset = 1.0 / texSize;

    var result: vec3<f32> = textureSample(image_texture, image_sampler, frag_in.tex_coords).rgb * weight[0];

    for (var i: i32 = 1; i < 5; i++) {
        let offset = params.direction * tex_offset * f32(i) * params.sample_distance;
        //let offset = vec2<f32>(1.0, 0.0) * tex_offset * f32(i) * params.sample_distance;
        result += textureSample(image_texture, image_sampler, frag_in.tex_coords + offset).rgb * weight[i];
        result += textureSample(image_texture, image_sampler, frag_in.tex_coords - offset).rgb * weight[i];
    }
    
    return vec4<f32>(result, 1.0);
}