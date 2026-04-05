use heapless::String;
use mem_fs::MemFs;

use crate::{
    graphics::ScreenGrid,
    input::{ButtonEvent, ButtonManager},
    system::{SettingsView, SystemCmd},
    tasks::TaskQueueHandle,
    touch::TouchEvent,
};

pub struct Context<'a> {
    pub grid: &'a mut ScreenGrid<'a>,
    pub buttons: &'a mut ButtonManager,
    pub settings: SettingsView<'a>,
    pub fs: &'a mut MemFs,
    pub tasks: &'a mut TaskQueueHandle<'a>,
}

pub trait App {
    fn init(&mut self, ctx: &mut Context, args: AppArgs) -> AppResponse;
    fn update(&mut self, input: InputEvents, ctx: &mut Context) -> AppResponse;
    fn render(&mut self, ctx: &mut Context);
    fn get_name(&self) -> &'static str;
}

// TODO: Base this value on mem_fs instead.
pub const MAX_FILE_NAME_LENGTH: usize = 255;

// TODO: Generate this enum using a macro.
#[derive(PartialEq)]
pub enum AppID {
    HomeApp,
    ColorPicker,
    SnakeApp,
    TestApp,
    SettingsApp,
    NotesApp,
    FilesApp,
    BrowserApp,
}

#[derive(PartialEq)]
pub enum AppArgs {
    None,
    NewFile(String<MAX_FILE_NAME_LENGTH>),
    OpenFile(String<MAX_FILE_NAME_LENGTH>),
}

#[derive(PartialEq)]
pub enum AppCmd {
    None,
    Dirty,
    SwitchApp(AppID, AppArgs),
}

#[derive(PartialEq)]
pub struct InputEvents {
    pub touch: Option<TouchEvent>,
    pub button: Option<ButtonEvent>,
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
    pub const fn switch(app: AppID, args: AppArgs) -> Self {
        Self {
            app: AppCmd::SwitchApp(app, args),
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
