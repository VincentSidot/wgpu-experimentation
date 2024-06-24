
const INFO_COLOR: egui::Color32 = egui::Color32::from_rgb(0xFF, 0xFF, 0xFF);
const WARNING_COLOR: egui::Color32 = egui::Color32::from_rgb(0xFF, 0xFF, 0);
const ERROR_COLOR: egui::Color32 = egui::Color32::from_rgb(0xFF, 0, 0);

enum DebugMessage {
    Info(String),
    Warning(String),
    Error(String),
}

impl DebugMessage {

    pub fn info<T: ToString>(msg: T) -> Self {
        Self::Info(msg.to_string())
    }

    pub fn warning<T: ToString>(msg: T) -> Self {
        Self::Warning(msg.to_string())
    }

    pub fn error<T: ToString>(msg: T) -> Self {
        Self::Error(msg.to_string())
    }

    pub fn to_label(&self) -> egui::Label {
        match self {
            Self::Info(msg) => egui::Label::new(
                egui::RichText::new(msg)
                    .color(INFO_COLOR)
            ),
            Self::Warning(msg) => egui::Label::new(
                egui::RichText::new(msg)
                    .color(WARNING_COLOR)
            ),
            Self::Error(msg) => egui::Label::new(
                egui::RichText::new(msg)
                    .color(ERROR_COLOR)
            ),
        }

    }
}

pub struct Debug {
    slider: i32,
    debug_console: Vec<DebugMessage>,
}

impl Debug {

    pub fn init() -> Self {
        Self {
            slider: 0,
            debug_console: Vec::new(),
        }
    }

    pub fn info(&mut self, msg: &str) {
        self.debug_console.push(DebugMessage::info(msg));
    }

    #[allow(dead_code)]
    pub fn warning(&mut self, msg: &str) {
        self.debug_console.push(DebugMessage::warning(msg));
    }

    #[allow(dead_code)]
    pub fn error(&mut self, msg: &str) {
        self.debug_console.push(DebugMessage::error(msg));
    }

    pub fn run_ui(&mut self, ui: &egui::Context) {
        egui::Window::new("Debug Window")
            // .vscroll(true)
            .default_open(true)
            .max_width(1000.0)
            .max_height(800.0)
            .default_width(800.0)
            .resizable(true)
            // .anchor(Align2::LEFT_TOP, [0.0, 0.0])
            .show(&ui, |ui| {

                if ui.add(egui::Button::new("Info")).clicked() {
                    self.info("This is an info message");
                }
                
                if ui.add(egui::Button::new("Warning")).clicked() {
                    self.warning("This is a warning message");
                }

                if ui.add(egui::Button::new("Error")).clicked() {
                    self.error("This is an error message");
                }

                ui.label("Slider");
                ui.add(egui::Slider::new(&mut self.slider, 0..=120).text("age"));
                ui.end_row();

                // proto_scene.egui(ui);
            });

        egui::Window::new("Debug Console")
            .default_open(false)
            .show(&ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Debug Console");
                    if ui.button("Clear").on_hover_text("Clear the debug console").clicked() {
                        self.debug_console.clear();
                    };
                });
                ui.separator();
                ui.vertical(|ui| {

                    egui::ScrollArea::vertical().stick_to_bottom(true).show(ui, |ui| {
                        for msg in self.debug_console.iter() {
                            ui.add(msg.to_label());
                        }
                    });
                });
            });
    }

}