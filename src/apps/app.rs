use crate::{
    graphics::ScreenGrid,
    input::{ButtonEvent, ButtonManager},
    touch::{TouchCalibration, TouchEvent},
};

pub struct Context<'a> {
    pub grid: &'a mut ScreenGrid<'a>,
    pub buttons: &'a mut ButtonManager,
}

pub trait App {
    fn init(&mut self, ctx: &mut Context) -> AppResponse;
    fn update(&mut self, input: InputEvents, ctx: &mut Context) -> AppResponse;
    fn render(&mut self, ctx: &mut Context);
    fn get_name(&self) -> &'static str;
}

// TODO: Generate this enum using a macro.
#[derive(PartialEq)]
pub enum AppID {
    HomeApp,
    ColorPicker,
    SnakeApp,
    TestApp,
    SettingsApp,
}

#[derive(PartialEq)]
pub enum AppCmd {
    None,
    Dirty,
    SwitchApp(AppID),
}

#[derive(PartialEq)]
pub struct InputEvents {
    pub touch: Option<TouchEvent>,
    pub button: Option<ButtonEvent>,
}

// HACK: Move out of here..
pub enum SystemCmd {
    StartCalibration,
    ApplyCalibration(TouchCalibration),
    SetBrightness(u8),
}

pub struct AppResponse {
    pub app: AppCmd,
    pub system: Option<SystemCmd>,
}

impl AppResponse {
    pub const fn none() -> Self {
        Self {
            app: AppCmd::None,
            system: None,
        }
    }
    pub const fn dirty() -> Self {
        Self {
            app: AppCmd::Dirty,
            system: None,
        }
    }
    pub const fn switch(app: AppID) -> Self {
        Self {
            app: AppCmd::SwitchApp(app),
            system: None,
        }
    }
    pub const fn system(cmd: SystemCmd) -> Self {
        Self {
            app: AppCmd::None,
            system: Some(cmd),
        }
    }
    pub const fn with_system(mut self, cmd: SystemCmd) -> Self {
        self.system = Some(cmd);
        self
    }
}
