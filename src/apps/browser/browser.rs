use log::info;

use crate::{
    apps::{
        app::{App, AppArgs, AppResponse, Context, InputEvents},
        browser::src::parser::Parser,
    },
    graphics::*,
};

const HTML_EXAMPLE: &str = r#"
<h1>Pocket Computer</h1>
<p>This is a tiny browser experiment.</p>
<p>Check out <a href="/about">about</a>.</p>
"#;

pub struct BrowserApp {
    parser: Parser,
}

impl Default for BrowserApp {
    fn default() -> Self {
        Self {
            parser: Parser::new(),
        }
    }
}

impl App for BrowserApp {
    fn init(&mut self, ctx: &mut Context, _args: AppArgs) -> AppResponse {
        ctx.grid.clear(' ', BASE03, BASE03);

        ctx.buttons.clear();
        ctx.buttons.register_default_buttons();

        if !ctx.fs.exists("example.html") {
            ctx.fs
                .create("example.html", HTML_EXAMPLE.as_bytes())
                .expect("Failed to write example.html");
        }
        let res = self.parser.parse(
            ctx.fs
                .read("example.html")
                .expect("Failed to load example.html"),
        );

        AppResponse::dirty()
    }
    fn update(&mut self, input: InputEvents, ctx: &mut Context) -> AppResponse {
        AppResponse::none()
    }
    fn render(&mut self, ctx: &mut Context) {
        self.parser.render(ctx.grid);
    }
    fn get_name(&self) -> &'static str {
        "POCKET_BROWSER"
    }
}
