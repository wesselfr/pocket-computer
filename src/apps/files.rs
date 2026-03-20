use crate::{
    apps::app::{App, AppResponse, Context, InputEvents},
    graphics::*,
    input::{ButtonEvent, Rect},
};

pub struct FilesApp {
    selected: usize,
}

impl Default for FilesApp {
    fn default() -> Self {
        Self { selected: 0 }
    }
}

impl App for FilesApp {
    fn init(&mut self, ctx: &mut Context) -> AppResponse {
        ctx.grid.clear(' ', BASE03, BASE03);

        ctx.buttons.clear();
        ctx.buttons.register_default_buttons();

        ctx.buttons.register_button(
            "UP",
            Rect {
                x_min: 0,
                y_min: 30 * CELL_H,
                x_max: 4 * CELL_W,
                y_max: 31 * CELL_H,
            },
        );
        ctx.buttons.register_button(
            "DOWN",
            Rect {
                x_min: 4 * CELL_W,
                y_min: 30 * CELL_H,
                x_max: 8 * CELL_W,
                y_max: 31 * CELL_H,
            },
        );

        AppResponse::dirty()
    }
    fn update(&mut self, input: InputEvents, ctx: &mut Context) -> AppResponse {
        if let Some(ButtonEvent::Up(id)) = input.button {
            if id == "UP" {
                self.selected = self.selected.saturating_sub(1);
            }
            if id == "DOWN" {
                let len = ctx.fs.entries().count();
                self.selected = (self.selected + 1).min(len.saturating_sub(1));
            }
            ctx.grid.clear(' ', BASE03, BASE03);
            return AppResponse::dirty();
        }
        AppResponse::none()
    }
    fn render(&mut self, ctx: &mut Context) {
        let mut total = 0;
        let mut y = 0;
        for (i, file) in ctx.fs.entries().enumerate() {
            let is_selected = self.selected == i;
            let fg = if is_selected { BASE03 } else { BASE3 };
            let bg = if is_selected { BASE3 } else { BASE03 };

            if is_selected {
                ctx.grid.write_str(1, 3 + y, ">", BASE3, BASE03);
                ctx.grid.draw_box(3, 3 + y, 17, 1, bg);
            }

            ctx.grid.write_str(3, 3 + y, &file.name, fg, bg);
            ctx.grid.write_str(
                20,
                3 + y,
                &heapless::format!(10; "{} B", file.size).unwrap_or_default(),
                fg,
                bg,
            );
            total += file.size;
            y += 1;
        }
        ctx.grid.write_str(
            20,
            3 + y,
            &heapless::format!(16; "{}/{} B", total, mem_fs::DEFAULT_STORAGE_SIZE)
                .unwrap_or_default(),
            BASE3,
            BASE03,
        );
    }
    fn get_name(&self) -> &'static str {
        "FILES"
    }
}
