fn aces_tonemap(hdr: vec3<f32>) -> vec3<f32> {
    let m1 = mat3x3(
        0.59719, 0.07600, 0.02840,
        0.35458, 0.90834, 0.13383,
        0.04823, 0.01566, 0.83777,
    );
    let m2 = mat3x3(
        1.60475, -0.10208, -0.00327,
        -0.53108,  1.10813, -0.07276,
        -0.07367, -0.00605,  1.07602,
    );
    let v = m1 * hdr;
    let a = v * (v + 0.0245786) - 0.000090537;
    let b = v * (0.983729 * v + 0.4329510) + 0.238081;
    return clamp(m2 * (a / b), vec3(0.0), vec3(1.0));
}

fn uncharted2_tonemap(x: vec3<f32>) -> vec3<f32> {
    let A = 0.15;
    let B = 0.50;
    let C = 0.10;
    let D = 0.20;
    let E = 0.02;
    let F = 0.30;
    let W = 11.2;
    return ((x * (A * x + C * B) + D * E) / (x * (A * x + B) + D * F)) - E / F;
}

fn uncharted2(color: vec3<f32>) -> vec3<f32> {
  let W = 11.2;
  let exposureBias = 2.0;
  let curr = uncharted2_tonemap(exposureBias * color);
  let whiteScale = 1.0 / uncharted2_tonemap(vec3(W));
  return curr * whiteScale;
}

const AGX_LOOK = 2;

fn agx(color: vec3<f32>) -> vec3<f32> {
    var col = color;

    let matrix = mat3x3<f32>(
        vec3<f32>(0.842, 0.0784, 0.0792),  
        vec3<f32>(0.0423, 0.878, 0.0792),   
        vec3<f32>(0.0424, 0.0784, 0.879)    
    );
    col = matrix * col;
    col = clamp((log2(col) + 12.47393) / 16.5, vec3<f32>(0.0), vec3<f32>(1.0));

    let x = (((-3.11 * col + 6.42) * col - 0.378) * col - 1.44);
    let sin_approx = x - (x * x * x) / 6.0; // Taylor approx near 0
    col = 0.5 + 0.5 * clamp(sin_approx, vec3<f32>(-1.0, -1.0, -1.0), vec3<f32>(1.0, 1.0, 1.0));

    if (AGX_LOOK == 1) {
        // Golden
        let luma = dot(col, vec3<f32>(0.216, 0.7152, 0.0722));
        col = mix(vec3<f32>(luma), col * vec3<f32>(1.0, 0.9, 0.5), 0.8);
    } else if (AGX_LOOK == 2) {
        // Punchy 
        let pow_col = vec3<f32>(
            pow(col.r, 1.35),
            pow(col.g, 1.35),
            pow(col.b, 1.35)
        );
        let luma = dot(col, vec3<f32>(0.216, 0.7152, 0.0722));
        col = mix(vec3<f32>(luma), pow_col, 1.4);
    }

    return col;
}

struct VertexOutput {
    @location(0) uv: vec2<f32>,
    @builtin(position) clip_position: vec4<f32>,
};

@vertex
fn vs_main(
    @builtin(vertex_index) vi: u32,
) -> VertexOutput {
    var out: VertexOutput;
    // Generate a triangle that covers the whole screen
    out.uv = vec2<f32>(f32((vi << 1u) & 2u), f32(vi & 2u));
    out.clip_position = vec4<f32>(out.uv * 2.0 - 1.0, 0.0, 1.0);
    // We need to invert the y coordinate so the image
    // is not upside down
    out.uv.y = 1.0 - out.uv.y;

    return out;
}

@group(0) @binding(0)
var hdr_image: texture_2d<f32>;
@group(0) @binding(1)
var hdr_sampler: sampler;

@group(0) @binding(2)
var emissive_image: texture_2d<f32>;
@group(0) @binding(3)
var emissive_sampler: sampler;

@group(0) @binding(4)
var outline_image: texture_2d<f32>;
@group(0) @binding(5)
var outline_sampler: sampler;

@group(0) @binding(6)
var glass_image: texture_2d<f32>;
@group(0) @binding(7)
var glass_sampler: sampler;

@fragment
fn fs_main(vs: VertexOutput) -> @location(0) vec4<f32> {
    let hdr = textureSample(hdr_image, hdr_sampler, vs.uv).rgb;
    let emissive = textureSample(emissive_image, emissive_sampler, vs.uv).rgb;
    let outline = textureSample(outline_image, outline_sampler, vs.uv).rgb;
    let glass = textureSample(glass_image, glass_sampler, vs.uv).rgb;

    let composite_image = hdr + glass + emissive + outline;

    let exposure = 1.0;
    var sdr = aces_tonemap(composite_image * exposure);

    // let gamma: f32 = 2.2;
    // sdr = pow(sdr, vec3<f32>(1.0 / gamma));
        
    return vec4(sdr, 1.0);
}