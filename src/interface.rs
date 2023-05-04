use imgui::Ui;

pub trait Editable {
    fn name(&self) -> &str;
    fn widget(&mut self, ui: &Ui);
}
