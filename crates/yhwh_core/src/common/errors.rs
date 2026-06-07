#[derive(Debug)]

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