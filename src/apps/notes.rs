use heapless::{String, Vec, format};
use log::error;

use crate::{
    apps::app::{App, AppArgs, AppResponse, Context, InputEvents, MAX_FILE_NAME_LENGTH},
    graphics::*,
    input::{ButtonEvent, Rect},
};

pub struct NotesApp {
    current_file: String<MAX_FILE_NAME_LENGTH>,
    text: Vec<u8, 255>,
    dirty: bool,
}

impl Default for NotesApp {
    fn default() -> Self {
        Self {
            current_file: String::new(),
            text: Vec::new(),
            dirty: false,
        }
    }
}

impl NotesApp {
    fn push_text(&mut self, text: &str) {
        for c in text.as_bytes() {
            let _ = self.text.push(*c);
        }
    }
}
impl App for NotesApp {
    fn init(&mut self, ctx: &mut Context, args: AppArgs) -> AppResponse {
        ctx.grid.clear(' ', BASE03, BASE03);

        ctx.buttons.clear();
        ctx.buttons.register_default_buttons();

        ctx.buttons.register_button(
            "SAVE",
            Rect {
                x_min: 0,
                y_min: 2 * CELL_H,
                x_max: 4 * CELL_W,
                y_max: 3 * CELL_H,
            },
        );
        ctx.buttons.register_button(
            "CLEAR",
            Rect {
                x_min: 5 * CELL_W,
                y_min: 2 * CELL_H,
                x_max: 10 * CELL_W,
                y_max: 3 * CELL_H,
            },
        );

        ctx.buttons.register_button(
            "A",
            Rect {
                x_min: 0,
                y_min: 30 * CELL_H,
                x_max: 4 * CELL_W,
                y_max: 31 * CELL_H,
            },
        );
        ctx.buttons.register_button(
            "B",
            Rect {
                x_min: 4 * CELL_W,
                y_min: 30 * CELL_H,
                x_max: 8 * CELL_W,
                y_max: 31 * CELL_H,
            },
        );
        ctx.buttons.register_button(
            "BACKSPACE",
            Rect {
                x_min: 8 * CELL_W,
                y_min: 30 * CELL_H,
                x_max: 18 * CELL_W,
                y_max: 31 * CELL_H,
            },
        );
        ctx.buttons.register_button(
            "SPACE",
            Rect {
                x_min: 18 * CELL_W,
                y_min: 30 * CELL_H,
                x_max: 24 * CELL_W,
                y_max: 31 * CELL_H,
            },
        );
        ctx.buttons.register_button(
            "ENTER",
            Rect {
                x_min: 24 * CELL_W,
                y_min: 30 * CELL_H,
                x_max: 29 * CELL_W,
                y_max: 31 * CELL_H,
            },
        );

        match args {
            // DEBUG: To be removed soon.
            AppArgs::None => {
                // Load text file if it exsist
                if let Some(data) = ctx.fs.read("notes.txt") {
                    self.text = Vec::from_slice(data).expect("Failed to load note");
                }
                self.current_file.clear();
                self.current_file
                    .push_str("notes.txt")
                    .expect("Failed to set current file.");
            }
            AppArgs::OpenFile(name) => {
                // Load text file if it exsist
                if let Some(data) = ctx.fs.read(&name) {
                    self.text = Vec::from_slice(data).expect("Failed to load note");
                    self.current_file.clear();
                    self.current_file
                        .push_str(&name)
                        .expect("Failed to set current file.");
                }
            }

            _ => {}
        }

        AppResponse::dirty()
    }
    fn update(&mut self, input: InputEvents, ctx: &mut Context) -> AppResponse {
        let mut dirty = false;

        if let Some(ButtonEvent::Up(id)) = input.button {
            if id == "SAVE" {
                let res = ctx.fs.write(&self.current_file, &self.text);

                if res.is_ok() {
                    self.dirty = false;
                } else {
                    error!("Failed to save file.. {:?}", res.err().unwrap());
                }

                ctx.grid.clear(' ', BASE03, BASE03);
                return AppResponse::dirty();
            }
            if id == "CLEAR" {
                self.text.clear();
                ctx.grid.clear(' ', BASE03, BASE03);
                dirty = true;
            }
            if id == "A" {
                self.push_text("Hello ");
                dirty = true;
            }
            if id == "B" {
                self.push_text("World! ");
                dirty = true;
            }
            if id == "BACKSPACE" {
                if !self.text.is_empty() {
                    self.text.pop();
                }
                ctx.grid.clear(' ', BASE03, BASE03);
                dirty = true;
            }
            if id == "SPACE" {
                self.push_text(" ");
                dirty = true;
            }
            if id == "ENTER" {
                self.push_text("\n");
                dirty = true;
            }
        }

        if dirty {
            self.dirty = true;
            return AppResponse::dirty();
        }
        AppResponse::none()
    }
    fn render(&mut self, ctx: &mut Context) {
        // TODO: Display file name here
        if self.dirty {
            ctx.grid.center_str(
                2,
                &format!({MAX_FILE_NAME_LENGTH + 2}; "*{}*", &self.current_file)
                    .unwrap_or_default(),
                BASE3,
                BASE02,
            );
        } else {
            ctx.grid.center_str(2, &self.current_file, BASE3, BASE02);
        }

        let mut x = 0;
        let mut y = 0;
        for ch in &self.text {
            if *ch == b'\n' {
                y += 1;
                x = 0;
                continue;
            }
            ctx.grid.put_char(x, 4 + y, *ch as char, BASE3, BASE03);
            x += 1;

            if x >= SCREEN_W / CELL_W {
                y += 1;
                x = 0;
            }
            if 4 + y >= SCREEN_H / CELL_H {
                break;
            }
        }
    }
    fn get_name(&self) -> &'static str {
        "NOTES"
    }
}
