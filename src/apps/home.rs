use esp_hal::time::{Duration, Instant};
use log::info;

use crate::{
    apps::app::{App, Context},
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
    fn init(&mut self, ctx: &mut Context) -> bool {
        ctx.grid.clear(' ', BASE03, BASE03);
        true
    }
    fn update(&mut self, event: Option<TouchEvent>, ctx: &mut Context) -> bool {
        let mut dirty = false;
        if let Some(event) = event {
            match event {
                TouchEvent::Down { x, y } | TouchEvent::Move { x, y } => {
                    ctx.grid.put_char(x / 6, y / 10, 'X', RED, VIOLET);
                    info!("Clicked on x: {}, y: {}", x, y);
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

        dirty
    }
    fn render(&mut self, ctx: &mut Context) {
        ctx.grid.put_char(0, 0, ' ', BASE03, YELLOW);
        ctx.grid.put_char(1, 0, ' ', BASE03, ORANGE);
        ctx.grid.put_char(2, 0, ' ', BASE03, RED);
        ctx.grid.put_char(3, 0, ' ', BASE03, MAGENTA);
        ctx.grid.put_char(4, 0, ' ', BASE03, VIOLET);
        ctx.grid.put_char(5, 0, ' ', BASE03, BLUE);
        ctx.grid.put_char(6, 0, ' ', BASE03, CYAN);
        ctx.grid.put_char(7, 0, ' ', BASE03, GREEN);

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
