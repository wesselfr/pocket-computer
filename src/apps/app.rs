use crate::{graphics::ScreenGrid, input::ButtonManager, touch::TouchEvent};

pub struct Context<'a> {
    pub grid: &'a mut ScreenGrid<'a>,
    pub buttons: &'a mut ButtonManager,
}

pub trait App {
    fn init(&mut self, ctx: &mut Context) -> AppCmd;
    fn update(&mut self, event: Option<TouchEvent>, ctx: &mut Context) -> AppCmd;
    fn render(&mut self, ctx: &mut Context);
}

// TODO: Generate this enum using a macro.
#[derive(PartialEq)]
pub enum AppID {
    HomeApp,
    ColorPicker,
}

#[derive(PartialEq)]
pub enum AppCmd {
    None,
    Dirty,
    SwitchApp(AppID),
}
