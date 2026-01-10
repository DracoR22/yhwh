#[derive(Eq, Hash, PartialEq, Clone)]
pub enum YHWHMouseButton {
    Left,
    Right,
    Middle,
    Back,
    Forward,
    Other(u16),
}