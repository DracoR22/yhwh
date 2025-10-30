#[derive(Debug)]
pub enum WgpuContextError {
    RequestDeviceError(wgpu::RequestDeviceError),
    NoAdapterFound
}

pub enum MeshNodesError {
    ModelNotFound,
    MeshNotFound,
    MaterialNotFound
}

pub enum CharacterControllerError {
    ControllerNotFound,
    ControllerHandleNotFound,
    ControllerBodyNotFound
}