use serde::{Deserialize, Serialize};

#[derive(PartialEq)]
pub enum GameState {
    Playing,
    Editor
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum LightType {
    Point,
    Directional,
    Spot
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum MeshRenderingMode {
    Pbr,
    Emissive,
    Glass,
    Flame,
    Fire
}