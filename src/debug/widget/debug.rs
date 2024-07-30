pub trait DebugItem {
    fn draw(&mut self, ui: &mut egui::Ui);
}
