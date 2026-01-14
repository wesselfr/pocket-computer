use crate::{
    apps::app::{App, AppCmd, Context},
    graphics::*,
    touch::TouchEvent,
};

pub struct HomeApp {}

impl Default for HomeApp {
    fn default() -> Self {
        Self {}
    }
}

impl App for HomeApp {
    fn init(&mut self, ctx: &mut Context) -> AppCmd {
        ctx.grid.clear(' ', BASE03, BASE03);
        ctx.buttons.clear();

        ctx.buttons.register_button(
            "TEST",
            crate::input::Rect {
                x_min: 0,
                y_min: 60,
                x_max: 80,
                y_max: 80,
            },
        );
        ctx.buttons.register_button(
            "COLOR",
            crate::input::Rect {
                x_min: 0,
                y_min: 90,
                x_max: 80,
                y_max: 110,
            },
        );
        ctx.buttons.register_button(
            "SNAKE",
            crate::input::Rect {
                x_min: 0,
                y_min: 120,
                x_max: 80,
                y_max: 140,
            },
        );

        AppCmd::Dirty
    }
    fn update(&mut self, event: Option<TouchEvent>, ctx: &mut Context) -> AppCmd {
        if let Some(event) = event {
            if let Some(button_event) = ctx.buttons.update(&event) {
                match button_event {
                    crate::input::ButtonEvent::Up(id) => {
                        if id == "TEST" {
                            return AppCmd::SwitchApp(crate::apps::app::AppID::TestApp);
                        }
                        if id == "COLOR" {
                            return AppCmd::SwitchApp(crate::apps::app::AppID::ColorPicker);
                        }
                        if id == "SNAKE" {
                            return AppCmd::SwitchApp(crate::apps::app::AppID::SnakeApp);
                        }
                    }
                    _ => {}
                }
            }
        }

        AppCmd::None
    }
    fn render(&mut self, ctx: &mut Context) {
        ctx.grid.write_str(0, 3, "Welcome!", BASE3, BASE03);
        ctx.grid
            .write_str(0, 4, "Select an app to get started.", BASE2, BASE03);
    }
    fn get_name(&self) -> &'static str {
        "HOME"
    }
}
