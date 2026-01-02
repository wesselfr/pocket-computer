use esp_hal::time::{Duration, Instant};
use log::info;

use crate::{
    apps::app::{App, AppCmd, Context},
    graphics::*,
    input::TouchEvent,
};

pub struct HomeApp {
    flicker: bool,
    count: u16,
    last_update: Instant,
}

impl Default for HomeApp {
    fn default() -> Self {
        Self {
            flicker: false,
            count: 0,
            last_update: Instant::now(),
        }
    }
}

impl App for HomeApp {
    fn init(&mut self, ctx: &mut Context) -> AppCmd {
        ctx.grid.clear(' ', BASE03, BASE03);
        AppCmd::Dirty
    }
    fn update(&mut self, event: Option<TouchEvent>, ctx: &mut Context) -> AppCmd {
        let mut dirty = false;
        if let Some(event) = event {
            match event {
                TouchEvent::Down { x, y } | TouchEvent::Move { x, y } => {
                    ctx.grid.put_char(x / 6, y / 10, 'X', RED, VIOLET);
                    info!("Clicked on x: {}, y: {}", x, y);

                    if x < 10 || y < 10 {
                        return AppCmd::SwitchApp(crate::apps::app::AppID::ColorPicker);
                    }
                }
                TouchEvent::Up => {
                    info!("No longer touching.");
                }
            }
            dirty = true;
        }

        if self.last_update.elapsed() > Duration::from_millis(33) {
            self.flicker = !self.flicker;
            self.count += 1;
            self.last_update = Instant::now();
            dirty = true;
        }

        if dirty {
            return AppCmd::Dirty;
        }
        AppCmd::None
    }
    fn render(&mut self, ctx: &mut Context) {
        if self.flicker {
            ctx.grid.write_str(0, 3, "Hello Rust!", BASE03, RED);
        } else {
            ctx.grid.write_str(0, 3, "Hello Rust!", BASE2, BASE03);
        }

        for i in 0..(self.count / 5).min(32) {
            ctx.grid.write_str(0, 4 + i, "another one!", BASE1, BASE03);
        }
        if self.count / 5 > 5 {
            self.count = 0;
            ctx.grid.clear(' ', BASE03, BASE03);
        }
    }
}
