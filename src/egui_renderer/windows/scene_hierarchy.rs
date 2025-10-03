pub struct SceneHierarchyWindow;

impl SceneHierarchyWindow {
    pub fn draw(ui: &egui::Context) {
        egui::Window::new("Settings")
        .resizable(true)
        .vscroll(true)
        .default_open(true)
        .show(&ui, |mut ui| {
            ui.label("Window!");
            let some_button = ui.button("SOME BUTTON");
            if some_button.clicked() {
                println!("BUTTON CLICKED!")
            }
            ui.label("Window!");
            ui.label("Window!");
            ui.label("Window!");
         });
    }
}