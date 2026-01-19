use crate::{
    apps::app::{App, AppResponse, Context, InputEvents},
    graphics::*,
};

pub const GIT_HASH: &str = match option_env!("GIT_HASH") {
    Some(v) => v,
    None => "unknown",
};

pub struct SettingsApp {}

impl Default for SettingsApp {
    fn default() -> Self {
        Self {}
    }
}

impl App for SettingsApp {
    fn init(&mut self, ctx: &mut Context) -> AppResponse {
        ctx.grid.clear(' ', BASE03, BASE03);

        ctx.buttons.clear();
        ctx.buttons.register_default_buttons();

        AppResponse::dirty()
    }
    fn update(&mut self, _input: InputEvents, _ctx: &mut Context) -> AppResponse {
        AppResponse::none()
    }
    fn render(&mut self, ctx: &mut Context) {
        ctx.grid.write_str(0, 3, "> ABOUT:", BASE3, BASE02);
        ctx.grid.write_str(
            0,
            4,
            &heapless::format!(48; "V: {} ({})", env!("CARGO_PKG_VERSION"), GIT_HASH)
                .unwrap_or_default(),
            BASE3,
            BASE03,
        );

        ctx.grid.write_str(0, 6, "> INPUT:", BASE3, BASE02);
    }
    fn get_name(&self) -> &'static str {
        "SETTINGS"
    }
}
