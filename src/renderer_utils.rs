use crate::{render_passes::{geometry_pass::GeometryPass, lighting_pass::LightingPass, ssao_pass::SSAOPass}, texture::Texture, wgpu_renderer::FinalTexture};

pub fn final_texture<'a>(
    mode: &FinalTexture,
    geometry_pass: &'a GeometryPass,
    lighting_pass: &'a LightingPass,
    ssao_pass: &'a SSAOPass,
) -> &'a Texture {
    match mode {
        FinalTexture::Lighting => &lighting_pass.texture,
        FinalTexture::Albedo => &geometry_pass.textures.base_color,
        FinalTexture::Normal => &geometry_pass.textures.normal,
        FinalTexture::Ssao => &ssao_pass.color_texture
    }
}