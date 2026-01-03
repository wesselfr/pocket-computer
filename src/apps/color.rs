use crate::{
    apps::app::{App, AppCmd, Context},
    graphics::*,
    touch::TouchEvent,
};

pub struct ColorApp {}

impl Default for ColorApp {
    fn default() -> Self {
        Self {}
    }
}

impl App for ColorApp {
    fn init(&mut self, ctx: &mut Context) -> AppCmd {
        ctx.grid.clear(' ', BASE03, BASE03);
        for y in 0..32 {
            ctx.grid.put_char(0, y, ' ', BASE03, YELLOW);
            ctx.grid.put_char(1, y, ' ', BASE03, ORANGE);
            ctx.grid.put_char(2, y, ' ', BASE03, RED);
            ctx.grid.put_char(3, y, ' ', BASE03, MAGENTA);
            ctx.grid.put_char(4, y, ' ', BASE03, VIOLET);
            ctx.grid.put_char(5, y, ' ', BASE03, BLUE);
            ctx.grid.put_char(6, y, ' ', BASE03, CYAN);
            ctx.grid.put_char(7, y, ' ', BASE03, GREEN);
        }
        AppCmd::Dirty
    }

    fn update(&mut self, event: Option<TouchEvent>, ctx: &mut Context) -> AppCmd {
        if let Some(event) = event {
            if let Some(button_event) = ctx.buttons.update(event) {
                match button_event {
                    crate::input::ButtonEvent::Down(id) => {
                        if id == "BACK" {
                            return AppCmd::SwitchApp(crate::apps::app::AppID::ColorPicker);
                        }
                    }
                    _ => {}
                }
            }
        }
        AppCmd::None
    }
    fn render(&mut self, _ctx: &mut Context) {}
}
