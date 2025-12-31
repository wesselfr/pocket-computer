use esp_hal::time::{Duration, Instant};

use crate::{apps::app::App, graphics::*};

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
    fn init(&mut self) {
        self.last_update = Instant::now();
    }
    fn update(&mut self) {
        if self.last_update.elapsed() > Duration::from_millis(33) {
            self.flicker = !self.flicker;
            self.count += 1;
            self.last_update = Instant::now();
        }
    }
    fn render(&mut self, grid: &mut ScreenGrid) {
        grid.put_char(0, 0, ' ', BASE03, YELLOW);
        grid.put_char(1, 0, ' ', BASE03, ORANGE);
        grid.put_char(2, 0, ' ', BASE03, RED);
        grid.put_char(3, 0, ' ', BASE03, MAGENTA);
        grid.put_char(4, 0, ' ', BASE03, VIOLET);
        grid.put_char(5, 0, ' ', BASE03, BLUE);
        grid.put_char(6, 0, ' ', BASE03, CYAN);
        grid.put_char(7, 0, ' ', BASE03, GREEN);

        if self.flicker {
            grid.write_str(0, 3, "Hello Rust!", BASE03, RED);
        } else {
            grid.write_str(0, 3, "Hello Rust!", BASE2, BASE03);
        }

        for i in 0..(self.count / 5).min(32) {
            grid.write_str(0, 4 + i, "another one!", BASE1, BASE03);
        }
        if self.count / 5 > 5 {
            self.count = 0;
            grid.clear(' ', BASE03, BASE03);
        }
    }
}
