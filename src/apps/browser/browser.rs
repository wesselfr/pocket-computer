use core::str::FromStr;
use heapless::{String, Vec};
use log::{error, info};

use crate::{
    apps::{
        app::{App, AppArgs, AppResponse, Context, InputEvents},
        browser::src::{
            navigation::{Resource, resolve_href_from, resolve_local_path},
            parser::{Parser, StrSlice},
            regions::{HitAction, HitRegion, MAX_REGIONS},
            render::Renderer,
        },
    },
    graphics::*,
    input::{ButtonEvent, Rect},
    tasks::TaskId,
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
    pending_request: Option<(TaskId, Resource)>,
    current_resource: Resource,
}

impl Default for BrowserApp {
    fn default() -> Self {
        Self {
            parser: Parser::new(),
            renderer: Renderer::new(),
            regions: Vec::new(),
            scroll_y: 0,
            pending_request: None,
            current_resource: Resource::Local(String::from_str("example.html").unwrap_or_default()),
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

        ctx.buttons.register_button(
            "TEST",
            Rect {
                x_min: 14 * CELL_W,
                y_min: 2 * CELL_H,
                x_max: 18 * CELL_W,
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
            if id == "TEST" {
                if let Ok(task) = ctx
                    .tasks
                    .add_http_task("192.168.178.52", 8000, "index.html")
                {
                    self.pending_request = Some((
                        task,
                        Resource::Remote {
                            host: String::from_str("192.168.178.52").unwrap_or_default(),
                            port: 8000,
                            path: String::from_str("index.html").unwrap_or_default(),
                        },
                    ));
                    info!("New Request!");
                }
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

        if let Some(pending_task) = &self.pending_request {
            if let Some(result) = ctx.tasks.take_result(pending_task.0) {
                info!("Request Finished");
                let resource = pending_task.1.clone();
                self.pending_request = None;

                match result {
                    crate::tasks::TaskResult::WithData { data } => {
                        if self.parser.parse(&data).is_err() {
                            error!("Failed to parse html page");
                            return AppResponse::none();
                        }

                        self.current_resource = resource;

                        self.scroll_y = 0;
                        return AppResponse::dirty();
                    }
                    _ => {
                        error!("Invalid data..")
                    }
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
        match resolve_href_from(href, &self.current_resource) {
            Resource::Local(path) => {
                let address = resolve_local_path(&path);

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

            Resource::Remote { host, port, path } => {
                let path = resolve_local_path(&path);
                if let Ok(task) = ctx.tasks.add_http_task(&host, port, &path) {
                    self.pending_request = Some((task, Resource::Remote { host, port, path }));
                }

                AppResponse::dirty()
            }
        }
    }
}
