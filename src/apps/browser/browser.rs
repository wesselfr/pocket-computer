use heapless::{String, Vec};
use log::error;

use crate::{
    apps::{
        app::{App, AppArgs, AppResponse, Context, InputEvents},
        browser::src::{
            navigation::{Resource, resolve_href, resolve_local_path},
            parser::{Parser, StrSlice},
            regions::{HitAction, HitRegion, MAX_REGIONS},
            render::Renderer,
        },
    },
    graphics::*,
    input::{ButtonEvent, Rect},
    touch::TouchEvent,
};

const HTML_EXAMPLE: &str = r#"
<h1>Pocket Computer</h1>
<p>This is a tiny browser experiment.</p>
<p>Check out <a href="/about" id="AWESOME">about</a>.</p>
<p>List test:</p>
<ul>
<li>First item</li>
<li>Second Item</li>
<li>Thrid Item</li>
</ul>
<br>
<blockquote>This is a quote!</blockquote>"#;

const HTML_EXAMPLE_2: &str = r#"
<h1>ABOUT PAGE</h1>
<p>Welcome to the about page</p>
<p>Click here to go <a href="/example">back</a>.</p>"#;

pub struct BrowserApp {
    parser: Parser,
    renderer: Renderer,
    regions: Vec<HitRegion, MAX_REGIONS>,
    scroll_y: u16,
}

impl Default for BrowserApp {
    fn default() -> Self {
        Self {
            parser: Parser::new(),
            renderer: Renderer::new(),
            regions: Vec::new(),
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
        if !ctx.fs.exists("about.html") {
            ctx.fs
                .create("about.html", HTML_EXAMPLE_2.as_bytes())
                .expect("Failed to write example.html");
        }

        let res = self.parser.parse(
            ctx.fs
                .read("example.html")
                .expect("Failed to load example.html"),
        );

        if res.is_err() {
            error!("Failed to parse html.");
        }

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

        if let Some(TouchEvent::Down { x, y }) = input.touch {
            let x = x.div_ceil(CELL_W);
            let y = y
                .div_ceil(CELL_H)
                .saturating_add(self.scroll_y)
                .saturating_sub(4);
            for region in &self.regions {
                if region.rect.inside(x, y) {
                    let HitAction::Link { href } = region.action;
                    return self.navigate(href, ctx);
                }
            }
        }

        AppResponse::none()
    }
    fn render(&mut self, ctx: &mut Context) {
        let mut page = SubGrid::new(0, 4, SCREEN_W, SCREEN_H - 1, ctx.grid);
        page.set_scroll(0, self.scroll_y);
        self.regions.clear();
        self.renderer
            .render(self.parser.get_dom(), &mut self.regions, &mut page);
    }
    fn get_name(&self) -> &'static str {
        "POCKET_BROWSER"
    }
}

impl BrowserApp {
    fn navigate(&mut self, href: StrSlice, ctx: &mut Context) -> AppResponse {
        let href = self.parser.get_dom().resolve(href);
        match resolve_href(href) {
            Resource::Local(path) => {
                let address = resolve_local_path(path);

                let Some(bytes) = ctx.fs.read(&address) else {
                    error!("Failed to load local html page: {}", address);
                    return AppResponse::none();
                };

                // FIX: This will still overwrite the current active page on failure.
                if self.parser.parse(bytes).is_err() {
                    error!("Failed to parse html page: {}", address);
                    return AppResponse::none();
                }

                self.scroll_y = 0;
                AppResponse::dirty()
            }

            Resource::Remote { host, path } => {
                error!("Remote navigation not implemented yet: {}{}", host, path);
                AppResponse::none()
            }
        }
    }
}
