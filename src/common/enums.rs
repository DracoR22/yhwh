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