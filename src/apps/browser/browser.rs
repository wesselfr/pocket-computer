use log::info;

use crate::{
    apps::{
        app::{App, AppArgs, AppResponse, Context, InputEvents},
        browser::src::{parser::Parser, render::Renderer},
    },
    graphics::*,
    input::{ButtonEvent, Rect},
};

const HTML_EXAMPLE: &str = r#"
<h1>Pocket Computer</h1>
<p>This is a tiny browser experiment.</p>
<p>Check out <a href="/about">about</a>.</p>
<p>List test:</p>
<ul>
<li>First item</li>
<li>Second Item</li>
<li>Thrid Item</li>
</ul>
<br>
<blockquote>This is a quote!</blockquot"#;

pub struct BrowserApp {
    parser: Parser,
    renderer: Renderer,
    scroll_y: u16,
}

impl Default for BrowserApp {
    fn default() -> Self {
        Self {
            parser: Parser::new(),
            renderer: Renderer::new(),
            scroll_y: 0,
        }
    }
}

impl App for BrowserApp {
    fn init(&mut self, ctx: &mut Context, _args: AppArgs) -> AppResponse {
        ctx.grid.clear(' ', BASE03, BASE03);

        ctx.buttons.clear();
        ctx.buttons.register_default_buttons();

        ctx.buttons.register_button(
            "STYLE",
            Rect {
                x_min: 0,
                y_min: 2 * CELL_H,
                x_max: 5 * CELL_W,
                y_max: 4 * CELL_H,
            },
        );

        ctx.buttons.register_button(
            "UP",
            Rect {
                x_min: 6 * CELL_W,
                y_min: 2 * CELL_H,
                x_max: 8 * CELL_W,
                y_max: 4 * CELL_H,
            },
        );
        ctx.buttons.register_button(
            "DOWN",
            Rect {
                x_min: 9 * CELL_W,
                y_min: 2 * CELL_H,
                x_max: 13 * CELL_W,
                y_max: 4 * CELL_H,
            },
        );

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
        if let Some(ButtonEvent::Up(id)) = input.button {
            if id == "STYLE" {
                self.renderer.dark_mode = !self.renderer.dark_mode;
                return AppResponse::dirty();
            }
            if id == "UP" {
                if self.scroll_y > 0 {
                    self.scroll_y -= 1;
                }
            }
            if id == "DOWN" {
                self.scroll_y += 1;
            }
        }
        AppResponse::none()
    }
    fn render(&mut self, ctx: &mut Context) {
        let mut page = SubGrid::new(0, 4, SCREEN_W, SCREEN_H - 1, ctx.grid);
        page.set_scroll(0, self.scroll_y);
        self.renderer.render(self.parser.get_dom(), &mut page);
    }
    fn get_name(&self) -> &'static str {
        "POCKET_BROWSER"
    }
}
