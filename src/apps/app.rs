use crate::graphics::ScreenGrid;

pub trait App {
    fn init(&mut self);
    fn update(&mut self);
    fn render(&mut self, grid: &mut ScreenGrid);
}
