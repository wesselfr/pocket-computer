use crate::{
    apps::app::{App, AppCmd, Context},
    graphics::*,
    input::TouchEvent,
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

    fn update(&mut self, event: Option<crate::input::TouchEvent>, _ctx: &mut Context) -> AppCmd {
        if let Some(event) = event {
            match event {
                TouchEvent::Down { x, y } | TouchEvent::Move { x, y } => {
                    if x < 10 || y < 10 {
                        return AppCmd::SwitchApp(crate::apps::app::AppID::HomeApp);
                    }
                }
                TouchEvent::Up => {}
            }
        }
        AppCmd::None
    }
    fn render(&mut self, _ctx: &mut Context) {}
}
