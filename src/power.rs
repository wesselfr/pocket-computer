use crate::system::{SystemCmd, SystemSettings};
use core::cell::RefCell;
use esp_hal::time::{Duration, Instant};
use log::info;

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum PowerMode {
    Active, // High refresh rate
    Idle,   // Low refresh rate, dim screen.
    Sleep,  // Lower refresh rate, screen off.
}

pub struct PowerManager {
    mode: PowerMode,
    last_activity: Instant,
}

impl PowerManager {
    pub fn new() -> Self {
        Self {
            mode: PowerMode::Active,
            last_activity: Instant::now(),
        }
    }
    pub fn update(&mut self, settings: &RefCell<SystemSettings>) -> Option<SystemCmd> {
        let mut s = settings.borrow_mut();
        let elapsed = self.last_activity.elapsed();

        let new_mode = if elapsed > Duration::from_secs(s.sleep_time) {
            PowerMode::Sleep
        } else if elapsed > Duration::from_secs(s.idle_time) {
            PowerMode::Idle
        } else {
            PowerMode::Active
        };

        let effective = match new_mode {
            PowerMode::Active => s.user_brightness,
            PowerMode::Idle => s.user_brightness / 2,
            PowerMode::Sleep => 0,
        };

        let changed = (new_mode != self.mode) || (effective != s.effective_brightness);

        self.mode = new_mode;
        s.effective_brightness = effective;

        if changed {
            info!("Power Mode: {:?}", self.mode);
            Some(SystemCmd::SetBrightness(effective))
        } else {
            None
        }
    }
    pub fn get_power_mode(&self) -> PowerMode {
        self.mode
    }
    pub fn register_activity(&mut self) {
        self.last_activity = Instant::now();
    }
    pub async fn await_frame(&mut self) {
        match self.mode {
            PowerMode::Active => {
                embassy_time::Timer::after_millis(16).await;
            }
            PowerMode::Idle => {
                embassy_time::Timer::after_millis(200).await;
            }
            PowerMode::Sleep => {
                embassy_time::Timer::after_millis(400).await;
            }
        }
    }
}
