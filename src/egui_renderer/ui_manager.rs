use egui::TextureId;

use crate::{asset_manager::AssetManager, egui_renderer::windows::scene_hierarchy::SceneHierarchyWindow, wgpu_context::WgpuContext};

pub struct EguiMaterial {
    pub texture_id: TextureId,
    pub material_name: String,
    pub material_index: usize
}

pub struct UiManager {
    pub scene_hierarchy_window: SceneHierarchyWindow,
    pub materials: Vec<EguiMaterial>
}

impl UiManager {
    pub fn new() -> Self {
        Self {
            scene_hierarchy_window: SceneHierarchyWindow::new(),
            materials: Vec::new()
        }
    }

    pub fn register_textures(&mut self, ctx: &WgpuContext, renderer: &mut egui_wgpu::Renderer, asset_manager: &AssetManager) {
        for material in asset_manager.get_all_materials().iter() {
             let material_name = material.name.clone();
             let material_index = asset_manager.get_material_index_by_name(&material_name);
             if let Some(texture) = asset_manager.get_texture_by_name(&format!("{material_name}_ALB.png")) {
                let texture_id = renderer.register_native_texture(&ctx.device, &texture.view, wgpu::FilterMode::Linear);

                let egui_material = EguiMaterial {
                    material_name,
                    material_index,
                    texture_id,
                };

                self.materials.push(egui_material);
             }
        }
    }
}