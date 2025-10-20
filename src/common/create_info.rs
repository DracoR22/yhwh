pub struct MeshNodeCreateInfo {
    pub mesh_name: String,
    pub material_name: String
}

pub struct GameObjectCreateInfo {
    pub name: String,
    pub model_name: String,
    pub position: cgmath::Vector3<f32>,
    pub size: cgmath::Vector3<f32>,
    pub rotation: cgmath::Matrix4<f32>,
    pub mesh_rendering_info: Vec<MeshNodeCreateInfo>
}