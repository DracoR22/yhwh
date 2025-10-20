use crate::egui_renderer::windows::scene_hierarchy::SceneHierarchyWindow;

pub struct UiManager {
    pub scene_hierarchy_window: SceneHierarchyWindow
}

impl UiManager {
    pub fn new() -> Self {
        Self {
            scene_hierarchy_window: SceneHierarchyWindow::new()
        }
    }
}