use cgmath::{InnerSpace, Vector3};
use rand::Rng;

use crate::{texture::TextureBuilder, wgpu_context::WgpuContext};

pub struct SSAOPass {

}

impl SSAOPass{
    pub fn new(ctx: &WgpuContext) {
        let kernel_samples = generate_kernel();
        let noise = generate_noise();
        let raw_noise: Vec<u8> = bytemuck::cast_slice(&noise).to_vec();
        
        let noise_texture = TextureBuilder::from_raw(raw_noise, 4, 4, wgpu::TextureFormat::Rgba32Float)
        .build(&ctx.device, &ctx.queue);
    }
}

fn generate_noise() -> Vec<[f32; 4]> {
    let mut rng = rand::thread_rng();
    let mut noise = Vec::<[f32; 4]>::new();
    for _ in 0..16 {
        noise.push([
            rng.gen_range(-1.0..1.0),
            rng.gen_range(-1.0..1.0),
            0.0,
            0.0,
       ]);
    }

    return noise
}

fn generate_kernel() -> Vec<[f32; 4]> {
    let mut rng = rand::thread_rng();
    let mut kernel_samples = Vec::<[f32; 4]>::new();
    for i in 0..64 {
        let mut sample = Vector3::<f32>::new(
            rng.gen_range(-1.0..1.0),
            rng.gen_range(-1.0..1.0),
            rng.gen_range(0.0..1.0),
        );
        sample = sample.normalize();
        sample *= rng.gen_range(0.0..1.0);
        let mut scale = i as f32 / 64.0;
        scale = lerp(0.1, 1.0, scale * scale);
        sample *= scale;
        kernel_samples.push([sample.x, sample.y, sample.z, 0.0]);
    }

    kernel_samples
}

fn lerp(a: f32, b: f32, f: f32) -> f32 {
    return a + f * (b - a)
}