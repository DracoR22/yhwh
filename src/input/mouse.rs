pub struct Mouse {
    pub delta_x: f64,
    pub delta_y: f64,
    scroll_dx: f32,
    scroll_dy: f32,
}

impl Mouse {
    pub fn new() -> Self {
        Self {
            delta_x: 0.0,
            delta_y: 0.0,
            scroll_dx: 0.0,
            scroll_dy: 0.0
        }
    }

    pub fn handle_mouse_motion(&mut self, delta_x: f64, delta_y: f64) {
        self.delta_x = delta_x;
        self.delta_y = delta_y;
    }
}