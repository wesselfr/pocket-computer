use crate::{graphics::ScreenGrid, input::TouchEvent};

pub struct Context<'a> {
    pub grid: &'a mut ScreenGrid<'a>,
}

pub trait App {
    fn init(&mut self, ctx: &mut Context) -> bool;
    fn update(&mut self, event: Option<TouchEvent>, ctx: &mut Context) -> bool;
    fn render(&mut self, ctx: &mut Context);
}
