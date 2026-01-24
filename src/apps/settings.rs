use esp_hal::time::{Duration, Instant};

use crate::{
    apps::app::{App, AppResponse, Context, InputEvents},
    graphics::*,
    input::{ButtonEvent, Rect},
    system::SystemCmd,
};

pub const GIT_HASH: &str = match option_env!("GIT_HASH") {
    Some(v) => v,
    None => "unknown",
};

pub struct SettingsApp {
    last_input_events: InputEvents,
    last_update: Instant,
    screen_brightness: u8,
}

impl Default for SettingsApp {
    fn default() -> Self {
        Self {
            last_input_events: InputEvents {
                touch: None,
                button: None,
            },
            last_update: Instant::now(),
            screen_brightness: 100,
        }
    }
}

impl App for SettingsApp {
    fn init(&mut self, ctx: &mut Context) -> AppResponse {
        ctx.grid.clear(' ', BASE03, BASE03);

        ctx.buttons.clear();
        ctx.buttons.register_default_buttons();

        ctx.buttons.register_button(
            "BRIGHTNESS_DOWN",
            Rect {
                x_min: 0,
                y_min: 11 * CELL_H,
                x_max: 15 * CELL_W,
                y_max: 12 * CELL_H,
            },
        );
        ctx.buttons.register_button(
            "BRIGHTNESS_UP",
            Rect {
                x_min: 0,
                y_min: 13 * CELL_H,
                x_max: 15 * CELL_W,
                y_max: 14 * CELL_H,
            },
        );

        AppResponse::dirty()
    }
    fn update(&mut self, input: InputEvents, _ctx: &mut Context) -> AppResponse {
        if let Some(ButtonEvent::Up(id)) = input.button {
            if id == "BRIGHTNESS_UP" {
                if self.screen_brightness <= 90 {
                    self.screen_brightness += 10;
                }
                return AppResponse::system(SystemCmd::SetBrightness(self.screen_brightness));
            }
            if id == "BRIGHTNESS_DOWN" {
                if self.screen_brightness > 10 {
                    self.screen_brightness -= 10;
                }
                return AppResponse::system(SystemCmd::SetBrightness(self.screen_brightness));
            }
        };

        if self.last_input_events != input {
            self.last_input_events = input;
            self.last_update = Instant::now();
            return AppResponse::dirty();
        }
        if self.last_update.elapsed() >= Duration::from_secs(1) {
            self.last_update = Instant::now();
            return AppResponse::dirty();
        }
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
        let mut epoch_time = Instant::now().duration_since_epoch().as_secs();
        let hours = {
            let val = epoch_time / 3600;
            epoch_time = epoch_time % 3600;
            val
        };
        let minutes = {
            let val = epoch_time / 60;
            epoch_time = epoch_time % 60;
            val
        };
        let seconds = epoch_time;
        ctx.grid.write_str(
            0,
            5,
            &heapless::format!(128; "Uptime: {:02}:{:02}:{:02}",hours, minutes, seconds)
                .unwrap_or_default(),
            BASE3,
            BASE03,
        );
        ctx.grid.write_str(
            0,
            6,
            &heapless::format!(64; "Cpu: {}", esp_hal::chip!()).unwrap_or_default(),
            BASE3,
            BASE03,
        );

        ctx.grid.write_str(0, 8, "> DEBUG:", BASE3, BASE02);
        let touch = if let Some(touch) = &self.last_input_events.touch {
            match touch {
                crate::touch::TouchEvent::Down { x, y } => {
                    &heapless::format!(128; "Down (x: {}, y: {})", x, y).unwrap_or_default()
                }
                crate::touch::TouchEvent::Move { x, y } => {
                    &heapless::format!(128; "Move (x: {}, y: {})", x, y).unwrap_or_default()
                }
                crate::touch::TouchEvent::Up => "Up",
            }
        } else {
            "None"
        };
        ctx.grid.write_str(0, 9, touch, BASE3, BASE03);

        ctx.grid.write_str(
            0,
            12,
            &heapless::format!(32; "Brightness: {:03}", self.screen_brightness).unwrap_or_default(),
            BASE3,
            BASE03,
        );
    }
    fn get_name(&self) -> &'static str {
        "SETTINGS"
    }
}
