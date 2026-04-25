use crate::{
    graphics::{
        BASE01, BASE02, BASE3, CELL_H, CELL_W, GridTarget, SCREEN_W, ScreenGrid,
        screen_pos_to_grid_pos,
    },
    touch::TouchEvent,
};
use core::u16;
use heapless::index_map::FnvIndexMap;

pub type ButtonId = &'static str;

#[derive(PartialEq)]
pub enum ButtonEvent {
    Down(ButtonId),
    Up(ButtonId),
}

#[derive(Debug)]
pub struct Rect {
    pub x_min: u16,
    pub y_min: u16,
    pub x_max: u16,
    pub y_max: u16,
}

impl Rect {
    pub fn new(x_min: u16, y_min: u16, x_max: u16, y_max: u16) -> Self {
        Self {
            x_min,
            y_min,
            x_max,
            y_max,
        }
    }
    pub fn from_width_height(x_pos: u16, y_pos: u16, width: u16, height: u16) -> Self {
        Self {
            x_min: x_pos,
            y_min: y_pos,
            x_max: x_pos + width,
            y_max: y_pos + height,
        }
    }
    pub fn from_grid_pos(x_min: u16, y_min: u16, x_max: u16, y_max: u16) -> Self {
        Self {
            x_min: x_min * CELL_W,
            y_min: y_min * CELL_H,
            x_max: x_max * CELL_W,
            y_max: y_max * CELL_H,
        }
    }
    pub fn inside(&self, x: u16, y: u16) -> bool {
        x >= self.x_min && x <= self.x_max && y >= self.y_min && y <= self.y_max
    }
}

pub struct ButtonManager {
    pub active_button: Option<ButtonId>,
    pub buttons: FnvIndexMap<ButtonId, Rect, 16>,
    dirty: bool,
}

impl ButtonManager {
    pub fn new() -> Self {
        Self {
            active_button: None,
            buttons: FnvIndexMap::<ButtonId, Rect, 16>::new(),
            dirty: false,
        }
    }
    pub fn register_button(&mut self, name: ButtonId, rect: Rect) {
        self.buttons
            .insert(name, rect)
            .expect("Failed to add button");
        self.dirty = true;
    }
    pub fn register_default_buttons(&mut self) {
        self.register_button(
            "BACK",
            Rect {
                x_min: 0,
                y_min: 0,
                x_max: 24,
                y_max: 20,
            },
        );
    }
    pub fn clear(&mut self) {
        self.buttons.clear();
    }
    pub fn update(&mut self, touch_event: &TouchEvent) -> Option<ButtonEvent> {
        match touch_event {
            TouchEvent::Down { x, y } | TouchEvent::Move { x, y } => {
                for (id, rect) in &self.buttons {
                    if rect.inside(*x, *y) {
                        self.dirty = self.active_button != Some(*id);
                        self.active_button = Some(*id);
                        return Some(ButtonEvent::Down(*id));
                    }
                }
            }
            TouchEvent::Up => {
                if let Some(previous_button) = self.active_button {
                    self.active_button = None;
                    self.dirty = true;
                    return Some(ButtonEvent::Up(previous_button));
                }
            }
        }

        None
    }
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }
    pub fn draw_buttons(&mut self, grid: &mut dyn GridTarget) {
        for button in &self.buttons {
            let min = screen_pos_to_grid_pos(button.1.x_min, button.1.y_min);
            let max = screen_pos_to_grid_pos(button.1.x_max, button.1.y_max);

            let (fg, bg) = if let Some(active) = self.active_button {
                if active == *button.0 {
                    (BASE01, BASE3)
                } else {
                    (BASE3, BASE01)
                }
            } else {
                (BASE3, BASE01)
            };

            grid.draw_box(min.0, min.1, max.0 - min.0, max.1 - min.1, bg);
            grid.write_str(min.0, min.1, button.0, fg, bg);
        }

        self.dirty = false;
    }
}

// TODO: Move this to a widgets file
const KEYBOARD_PAGES: [&str; 8] = [
    "abcdef", "ghijkl", "mnopqr", "stuvwx", "yz", "!?@#$%^", "&*()-+=", "_/|\\<>",
];

const KEY_WIDTH: u16 = 5;

pub enum KeyboardEvent {
    Insert(char),
    Backspace,
    Space,
    Enter,
}

pub struct VirtualKeyboard {
    page_index: usize,
    active_key: Option<usize>,
    shift: bool,
    dirty: bool,
    active_button: Option<usize>,
    buttons: [(&'static str, Rect); 6],
}

impl VirtualKeyboard {
    pub fn new() -> Self {
        Self {
            page_index: 0,
            active_key: None,
            shift: false,
            dirty: false,
            active_button: None,
            buttons: [
                (" <", Rect::from_grid_pos(0, 30, 4, 31)),
                ("  >", Rect::from_grid_pos(7, 30, 11, 31)),
                ("BACKSPACE", Rect::from_grid_pos(11, 30, 21, 31)),
                ("SPACE", Rect::from_grid_pos(21, 30, 27, 31)),
                ("ENTER", Rect::from_grid_pos(27, 30, 33, 31)),
                ("SHIFT", Rect::from_grid_pos(33, 30, 39, 31)),
            ],
        }
    }
    pub fn update(&mut self, touch_event: &TouchEvent) -> (Option<KeyboardEvent>, bool) {
        match touch_event {
            TouchEvent::Down { x, y } | TouchEvent::Move { x, y } => {
                let x_offset = self.get_x_offset();
                for (i, _ch) in KEYBOARD_PAGES[self.page_index].chars().enumerate() {
                    let rect = Rect::from_width_height(
                        (x_offset + (i as u16 * KEY_WIDTH)) * CELL_W,
                        28 * CELL_H,
                        3 * CELL_W,
                        1 * CELL_H,
                    );
                    if rect.inside(*x, *y) {
                        self.dirty = self.active_key != Some(i);
                        self.active_key = Some(i);
                        self.active_button = None;

                        // Early return, only register keypress on TouchEvent::Up
                        return (None, self.dirty);
                    }
                }

                for (i, button) in self.buttons.iter().enumerate() {
                    if button.1.inside(*x, *y) {
                        self.dirty = self.active_button != Some(i);
                        self.active_key = None;
                        self.active_button = Some(i);

                        // Early return, only register keypress on TouchEvent::Up
                        return (None, self.dirty);
                    }
                }
            }
            TouchEvent::Up => {
                if let Some(active_key) = self.active_key {
                    self.active_key = None;
                    let c = KEYBOARD_PAGES[self.page_index]
                        .chars()
                        .nth(active_key)
                        .expect("Failed to get key");
                    if self.shift {
                        return (Some(KeyboardEvent::Insert(c.to_ascii_uppercase())), true);
                    } else {
                        return (Some(KeyboardEvent::Insert(c)), true);
                    }
                }

                match self.active_button {
                    // <
                    Some(0) => {
                        self.active_button = None;
                        self.prev_page();
                        self.dirty = true;
                    }
                    // >
                    Some(1) => {
                        self.active_button = None;
                        self.next_page();
                        self.dirty = true;
                    }
                    // BACKSPACE
                    Some(2) => {
                        self.active_button = None;
                        return (Some(KeyboardEvent::Backspace), true);
                    }
                    // SPACE
                    Some(3) => {
                        self.active_button = None;
                        return (Some(KeyboardEvent::Space), true);
                    }
                    // ENTER
                    Some(4) => {
                        self.active_button = None;
                        return (Some(KeyboardEvent::Enter), true);
                    }
                    // SHIFT
                    Some(5) => {
                        self.active_button = None;
                        self.shift();
                        self.dirty = true;
                    }
                    _ => {}
                }
            }
        }
        return (None, self.dirty);
    }

    pub fn draw(&mut self, grid: &mut dyn GridTarget) {
        grid.draw_box(0, 28, SCREEN_W / CELL_W, 3, BASE02);

        let x_offset = self.get_x_offset();
        for (i, ch) in KEYBOARD_PAGES[self.page_index].chars().enumerate() {
            let is_active = self.active_key == Some(i);

            let (fg, bg) = if is_active {
                (BASE01, BASE3)
            } else {
                (BASE3, BASE01)
            };

            grid.draw_box(x_offset + (i as u16 * KEY_WIDTH), 28, 3, 1, bg);
            grid.put_char(
                x_offset + 1 + (i as u16 * KEY_WIDTH),
                28,
                if self.shift {
                    ch.to_ascii_uppercase()
                } else {
                    ch
                },
                fg,
                bg,
            );
        }

        grid.write_str(
            4,
            30,
            &heapless::format!(3;"{}/{}", self.page_index + 1, KEYBOARD_PAGES.len())
                .unwrap_or_default(),
            BASE3,
            BASE01,
        );

        for (i, button) in self.buttons.iter().enumerate() {
            let min = screen_pos_to_grid_pos(button.1.x_min, button.1.y_min);
            let max = screen_pos_to_grid_pos(button.1.x_max, button.1.y_max);

            let is_active = self.active_button == Some(i);

            let (fg, bg) = if is_active {
                (BASE01, BASE3)
            } else {
                (BASE3, BASE01)
            };

            grid.draw_box(min.0, min.1, max.0 - min.0, max.1 - min.1, bg);
            grid.write_str(min.0, min.1, button.0, fg, bg);
        }
        self.dirty = false;
    }
    fn get_x_offset(&self) -> u16 {
        const KEY_WIDTH: u16 = 5;
        let num_keys = KEYBOARD_PAGES[self.page_index].chars().count();
        let total_width = num_keys as u16 * KEY_WIDTH;

        ((SCREEN_W / CELL_W) - total_width as u16) / 2
    }
    pub fn next_page(&mut self) {
        if self.page_index < KEYBOARD_PAGES.len() - 1 {
            self.page_index += 1
        };
    }
    pub fn prev_page(&mut self) {
        if self.page_index > 0 {
            self.page_index -= 1;
        }
    }
    pub fn shift(&mut self) {
        self.shift = !self.shift;
    }
}
