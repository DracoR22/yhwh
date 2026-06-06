use cgmath::{Matrix4, SquareMatrix};

use crate::{common::{enums::MeshRenderingMode, types::{AnimatedRenderData, RenderItem}}, frustum::Frustum};

pub struct RenderDataManager {
    animated_render_data: Vec<AnimatedRenderData>,
    render_items_pbr: Vec<RenderItem>,
    render_items_glass: Vec<RenderItem>,
    render_items_emissive: Vec<RenderItem>,
    render_items_flame: Vec<RenderItem>,
    render_items_animated: Vec<RenderItem>,
    render_items_outlined: Vec<RenderItem>
}

impl RenderDataManager {
    pub fn new() -> Self {
        Self {
            animated_render_data: Vec::new(),
            render_items_pbr: Vec::new(),
            render_items_glass: Vec::new(),
            render_items_emissive: Vec::new(),
            render_items_flame: Vec::new(),
            render_items_animated: Vec::new(),
            render_items_outlined: Vec::new()
        }
    }

    pub fn submit_animated_render_data(&mut self, render_data: AnimatedRenderData) {
        self.animated_render_data.push(render_data);
    }

    pub fn submit_animated_render_items(&mut self, render_items: &Vec<RenderItem>) {
        for item in render_items.iter() {
            self.render_items_animated.push(item.clone());
        }
    }

    pub fn submit_outlined_render_items(&mut self, render_items: &Vec<RenderItem>) {
        for item in render_items.iter() {
            self.render_items_outlined.push(item.clone());
        }
    }

    pub fn submit_render_items(&mut self, render_items: &Vec<RenderItem>, frustum: &Frustum) {
        for item in render_items.iter() {
            let visible = match item.aabb {
                Some(aabb) => {
                    let world_aabb = aabb.transform(item.model_matrix);
                    frustum.intersects_aabb(&world_aabb)
                }
                None => true,
            };

            // if not aabb provided, it will always pass
            if !visible {
                continue;
            }

            match item.rendering_mode {
                MeshRenderingMode::Pbr => {
                    self.render_items_pbr.push(item.clone())
                },
                MeshRenderingMode::Emissive => {
                    self.render_items_emissive.push(item.clone())
                },
                MeshRenderingMode::Glass => {
                    self.render_items_glass.push(item.clone())
                },
                MeshRenderingMode::Flame => {
                    self.render_items_flame.push(item.clone())
                }
            }
        }
    }

    pub fn clear(&mut self) {
        self.animated_render_data.clear();
        self.render_items_pbr.clear();
        self.render_items_outlined.clear();
        self.render_items_animated.clear();
    }
}

// Getters
impl RenderDataManager {
    pub fn render_items_pbr(&self) -> &Vec<RenderItem> {
        &self.render_items_pbr
    }

    pub fn render_items_animated(&self) -> &Vec<RenderItem> {
        &self.render_items_animated
    }

    pub fn render_items_outlined(&self) -> &Vec<RenderItem> {
        &self.render_items_outlined
    }

    pub fn render_items_animated_mut(&mut self) -> &mut Vec<RenderItem> {
        &mut self.render_items_animated
    }

    pub fn animated_render_data(&self) -> &Vec<AnimatedRenderData> {
        &self.animated_render_data
    }
}