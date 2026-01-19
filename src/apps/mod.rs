use crate::apps::{
    app::{App, AppID, AppResponse, InputEvents},
    color::ColorApp,
    home::HomeApp,
    settings::SettingsApp,
    snake::SnakeApp,
    test::TestApp,
};

pub mod app;
pub mod color;
pub mod home;
pub mod settings;
pub mod snake;
pub mod test;

pub enum AppState {
    Home(HomeApp),
    Color(ColorApp),
    Snake(SnakeApp),
    Test(TestApp),
    Settings(SettingsApp),
}

impl AppState {
    fn app_mut(&mut self) -> &mut dyn App {
        match self {
            AppState::Home(app) => app,
            AppState::Color(app) => app,
            AppState::Snake(app) => app,
            AppState::Test(app) => app,
            AppState::Settings(app) => app,
        }
    }
    fn app_ref(&self) -> &dyn App {
        match self {
            AppState::Home(app) => app,
            AppState::Color(app) => app,
            AppState::Snake(app) => app,
            AppState::Test(app) => app,
            AppState::Settings(app) => app,
        }
    }
    pub fn switch(&self, app: AppID) -> AppState {
        match app {
            AppID::HomeApp => AppState::Home(HomeApp::default()),
            AppID::ColorPicker => AppState::Color(ColorApp::default()),
            AppID::SnakeApp => AppState::Snake(SnakeApp::default()),
            AppID::TestApp => AppState::Test(TestApp::default()),
            AppID::SettingsApp => AppState::Settings(SettingsApp::default()),
        }
    }
}

impl App for AppState {
    fn init(&mut self, ctx: &mut app::Context) -> AppResponse {
        self.app_mut().init(ctx)
    }
    fn update(&mut self, input: InputEvents, ctx: &mut app::Context) -> AppResponse {
        self.app_mut().update(input, ctx)
    }
    fn render(&mut self, ctx: &mut app::Context) {
        self.app_mut().render(ctx);
    }
    fn get_name(&self) -> &'static str {
        self.app_ref().get_name()
    }
}
