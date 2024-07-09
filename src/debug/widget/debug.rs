use crate::debug::gui::DebugMessage;


pub trait DebugItem
{
    fn draw(&mut self, ui: &mut egui::Ui, console: &mut Vec<DebugMessage>);
}