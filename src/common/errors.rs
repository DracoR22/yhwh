pub enum MeshNodesError {
    ModelNotFound,
    MeshNotFound,
    MaterialNotFound
}

#[derive(Debug)]
pub enum WgpuContextError {
    RequestDeviceError(wgpu::RequestDeviceError),
    NoAdapterFound
}