use core::cell::RefCell;

use crate::touch::TouchCalibration;

pub enum SystemCmd {
    StartCalibration,
    ApplyCalibration(TouchCalibration),
    SetBrightness(u8),
}

pub struct SystemSettings {
    pub user_brightness: u8,
    pub effective_brightness: u8,
    pub sleep_time: u64,
    pub idle_time: u64,
}

impl Default for SystemSettings {
    fn default() -> Self {
        SystemSettings {
            user_brightness: 100,
            effective_brightness: 100,
            sleep_time: 60,
            idle_time: 10,
        }
    }
}

#[derive(Copy, Clone)]
pub struct SettingsView<'a> {
    inner: &'a RefCell<SystemSettings>,
}

impl<'a> SettingsView<'a> {
    pub fn new(settings: &'a RefCell<SystemSettings>) -> SettingsView<'a> {
        SettingsView { inner: settings }
    }
    pub fn read<R>(&self, f: impl FnOnce(&SystemSettings) -> R) -> R {
        let s = self.inner.borrow();
        f(&s)
    }
}
