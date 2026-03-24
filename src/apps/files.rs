use crate::{
    apps::app::{App, AppArgs, AppResponse, Context, InputEvents, MAX_FILE_NAME_LENGTH},
    graphics::*,
    input::{ButtonEvent, Rect},
};
use heapless::String;

pub struct FilesApp {
    selected: usize,
}

impl Default for FilesApp {
    fn default() -> Self {
        Self { selected: 0 }
    }
}

impl App for FilesApp {
    fn init(&mut self, ctx: &mut Context, _args: AppArgs) -> AppResponse {
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
        ctx.buttons.register_button(
            "EDIT",
            Rect {
                x_min: 12 * CELL_W,
                y_min: 30 * CELL_H,
                x_max: 16 * CELL_W,
                y_max: 31 * CELL_H,
            },
        );
        ctx.buttons.register_button(
            "DEL",
            Rect {
                x_min: 18 * CELL_W,
                y_min: 30 * CELL_H,
                x_max: 22 * CELL_W,
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
            if id == "EDIT" {
                if let Some(file) = ctx.fs.entries().nth(self.selected) {
                    let mut file_name: String<MAX_FILE_NAME_LENGTH> = String::new();
                    if file_name.push_str(file.name.as_str()).is_ok() {
                        return AppResponse::switch(
                            super::app::AppID::NotesApp,
                            AppArgs::OpenFile(String::from(file_name)),
                        );
                    }
                }
            }
            if id == "DEL" {
                let mut file_name: String<MAX_FILE_NAME_LENGTH> = String::new();
                if let Some(file) = ctx.fs.entries().nth(self.selected) {
                    file_name
                        .push_str(&file.name)
                        .expect("Failed to set file name.");
                }
                if !file_name.is_empty() {
                    ctx.fs.delete(&file_name).expect("Failed to delete file.");

                    // Update index
                    let len = ctx.fs.entries().count();
                    self.selected = self.selected.min(len.saturating_sub(1));
                }
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
