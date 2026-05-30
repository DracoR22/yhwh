use std::collections::{HashMap, HashSet};

pub struct ChunkEditorState {
    pub selected_id: i32,
    pub selected_model_index: usize,
    pub selected_mesh_index_map: HashMap<usize, usize>,
    pub add_game_object_selected: bool,
    pub objects_marked_for_removal: HashSet<usize>,
}

impl ChunkEditorState {
    pub fn new() -> Self {
        ChunkEditorState { 
                selected_id: -1,
                selected_model_index: 0,
                add_game_object_selected: false,
                selected_mesh_index_map: HashMap::new(),
                objects_marked_for_removal: HashSet::new(),
        }
    }
}