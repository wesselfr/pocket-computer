use crate::{
    apps::app::{App, AppArgs, AppID, AppResponse, Context, InputEvents},
    graphics::*,
};

pub struct HomeApp {}

impl Default for HomeApp {
    fn default() -> Self {
        Self {}
    }
}

impl App for HomeApp {
    fn init(&mut self, ctx: &mut Context, _args: AppArgs) -> AppResponse {
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
        ctx.buttons.register_button(
            "NOTES",
            crate::input::Rect {
                x_min: 0,
                y_min: 150,
                x_max: 80,
                y_max: 170,
            },
        );
        ctx.buttons.register_button(
            "FILES",
            crate::input::Rect {
                x_min: 0,
                y_min: 180,
                x_max: 80,
                y_max: 200,
            },
        );
        ctx.buttons.register_button(
            "SETTINGS",
            crate::input::Rect {
                x_min: 0,
                y_min: 210,
                x_max: 80,
                y_max: 230,
            },
        );
        ctx.buttons.register_button(
            "BROWSER",
            crate::input::Rect {
                x_min: 0,
                y_min: 240,
                x_max: 80,
                y_max: 260,
            },
        );

        AppResponse::dirty()
    }
    fn update(&mut self, input: InputEvents, _ctx: &mut Context) -> AppResponse {
        if let Some(button_event) = input.button {
            match button_event {
                crate::input::ButtonEvent::Up(id) => {
                    if id == "TEST" {
                        return AppResponse::switch(AppID::TestApp, AppArgs::None);
                    }
                    if id == "COLOR" {
                        return AppResponse::switch(AppID::ColorPicker, AppArgs::None);
                    }
                    if id == "SNAKE" {
                        return AppResponse::switch(AppID::SnakeApp, AppArgs::None);
                    }
                    if id == "NOTES" {
                        return AppResponse::switch(AppID::NotesApp, AppArgs::None);
                    }
                    if id == "FILES" {
                        return AppResponse::switch(AppID::FilesApp, AppArgs::None);
                    }
                    if id == "SETTINGS" {
                        return AppResponse::switch(AppID::SettingsApp, AppArgs::None);
                    }
                    if id == "BROWSER" {
                        return AppResponse::switch(AppID::BrowserApp, AppArgs::None);
                    }
                }
                _ => {}
            }
        }

        AppResponse::none()
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
