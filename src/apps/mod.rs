use crate::apps::{
    app::{App, AppCmd, AppID},
    color::ColorApp,
    home::HomeApp,
};

pub mod app;
pub mod color;
pub mod home;

pub enum AppState {
    Home(HomeApp),
    Color(ColorApp),
}

impl AppState {
    fn app_mut(&mut self) -> &mut dyn App {
        match self {
            AppState::Home(app) => app,
            AppState::Color(app) => app,
        }
    }
    pub fn switch(&self, app: AppID) -> AppState {
        match app {
            AppID::HomeApp => AppState::Home(HomeApp::default()),
            AppID::ColorPicker => AppState::Color(ColorApp::default()),
        }
    }
}

impl App for AppState {
    fn init(&mut self, ctx: &mut app::Context) -> AppCmd {
        self.app_mut().init(ctx)
    }
    fn update(
        &mut self,
        event: Option<crate::touch::TouchEvent>,
        ctx: &mut app::Context,
    ) -> AppCmd {
        self.app_mut().update(event, ctx)
    }
    fn render(&mut self, ctx: &mut app::Context) {
        self.app_mut().render(ctx);
    }
}
