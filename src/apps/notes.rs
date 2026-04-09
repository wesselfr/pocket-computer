use heapless::{String, Vec, format};
use log::error;

use crate::{
    apps::app::{App, AppArgs, AppResponse, Context, InputEvents, MAX_FILE_NAME_LENGTH},
    graphics::*,
    input::{ButtonEvent, KeyboardEvent, Rect, VirtualKeyboard},
};

pub struct NotesApp {
    current_file: String<MAX_FILE_NAME_LENGTH>,
    text: Vec<u8, 255>,
    cursor_index: usize,
    dirty: bool,
    keyboard: VirtualKeyboard,
}

impl Default for NotesApp {
    fn default() -> Self {
        Self {
            current_file: String::new(),
            text: Vec::new(),
            cursor_index: 0,
            dirty: false,
            keyboard: VirtualKeyboard::new(),
        }
    }
}

impl NotesApp {
    fn push_text(&mut self, text: &str) {
        for c in text.as_bytes() {
            let _ = self.text.insert(self.cursor_index, *c);
            self.cursor_index += 1;
        }
    }
    fn push_char(&mut self, c: char) {
        let _ = self.text.insert(self.cursor_index, c as u8);
        self.cursor_index += 1;
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
            " < ",
            Rect {
                x_min: 0,
                y_min: 26 * CELL_H,
                x_max: 3 * CELL_W,
                y_max: 27 * CELL_H,
            },
        );
        ctx.buttons.register_button(
            " > ",
            Rect {
                x_min: 4 * CELL_W,
                y_min: 26 * CELL_H,
                x_max: 7 * CELL_W,
                y_max: 27 * CELL_H,
            },
        );
        ctx.buttons.register_button(
            "BEGIN",
            Rect {
                x_min: 8 * CELL_W,
                y_min: 26 * CELL_H,
                x_max: 13 * CELL_W,
                y_max: 27 * CELL_H,
            },
        );
        ctx.buttons.register_button(
            "END",
            Rect {
                x_min: 14 * CELL_W,
                y_min: 26 * CELL_H,
                x_max: 17 * CELL_W,
                y_max: 27 * CELL_H,
            },
        );

        match args {
            AppArgs::None => {
                // Load text file if it exsist
                // DEBUG: To be removed soon.
                if let Some(data) = ctx.fs.read("notes.txt") {
                    self.text = Vec::from_slice(data).expect("Failed to load note");
                    self.cursor_index = self.text.len();
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
                    self.cursor_index = self.text.len();

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
                self.cursor_index = 0;
                ctx.grid.clear(' ', BASE03, BASE03);
                self.dirty = true;
                return AppResponse::dirty();
            }
            // Cursor logic
            if id == " < " {
                if self.cursor_index > 0 {
                    self.cursor_index -= 1;
                    // HACK: Force redraw to prevent ghost cursor.
                    ctx.grid.clear(' ', BASE03, BASE03);
                }
            }
            if id == " > " {
                if self.cursor_index < self.text.len() {
                    self.cursor_index += 1;
                    // HACK: Force redraw to prevent ghost cursor.
                    ctx.grid.clear(' ', BASE03, BASE03);
                }
            }
            if id == "BEGIN" {
                self.cursor_index = 0;
                // HACK: Force redraw to prevent ghost cursor.
                ctx.grid.clear(' ', BASE03, BASE03);
            }
            if id == "END" {
                self.cursor_index = self.text.len();
                // HACK: Force redraw to prevent ghost cursor.
                ctx.grid.clear(' ', BASE03, BASE03);
            }
        }

        if let Some(touch_event) = input.touch {
            let (event, is_dirty) = self.keyboard.update(&touch_event);
            match event {
                None => {}
                Some(KeyboardEvent::Insert(char)) => {
                    self.push_char(char);
                    self.dirty = true;
                }
                Some(KeyboardEvent::Backspace) => {
                    if self.cursor_index > 0 && !self.text.is_empty() {
                        self.text.remove(self.cursor_index - 1);
                        self.cursor_index -= 1;
                        // self.text.pop();
                    }
                    ctx.grid.clear(' ', BASE03, BASE03);
                    self.dirty = true;
                }
                Some(KeyboardEvent::Space) => {
                    self.push_text(" ");
                    self.dirty = true;
                }
                Some(KeyboardEvent::Enter) => {
                    self.push_text("\n");
                    self.dirty = true;

                    // Force redraw if cursor is not at end
                    if self.cursor_index != self.text.len() {
                        ctx.grid.clear(' ', BASE03, BASE03);
                    }
                }
            }

            if is_dirty {
                return AppResponse::dirty();
            }
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

        self.keyboard.draw(ctx.grid);

        let mut x = 0;
        let mut y = 0;
        for (index, ch) in self.text.iter().enumerate() {
            let is_cursor = index == self.cursor_index;

            if *ch == b'\n' {
                if is_cursor {
                    ctx.grid.put_char(x, 4 + y, ' ', BASE3, BASE01);
                }

                y += 1;
                x = 0;
                continue;
            }

            if is_cursor {
                ctx.grid.put_char(x, 4 + y, *ch as char, BASE3, BASE01);
            } else {
                ctx.grid.put_char(x, 4 + y, *ch as char, BASE3, BASE03);
            }

            x += 1;

            if x >= SCREEN_W / CELL_W {
                y += 1;
                x = 0;
            }
            if 4 + y >= SCREEN_H / CELL_H {
                break;
            }
        }
        if self.cursor_index == self.text.len() {
            ctx.grid.put_char(x, 4 + y, ' ', BASE3, BASE01);
        }
    }
    fn get_name(&self) -> &'static str {
        "NOTES"
    }
}
