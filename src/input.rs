use crate::{
    graphics::{BASE3, MAGENTA, ScreenGrid, screen_pos_to_grid_pos},
    touch::TouchEvent,
};
use core::u16;
use heapless::index_map::FnvIndexMap;

pub type ButtonId = &'static str;

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
                x_max: 20,
                y_max: 20,
            },
        );
    }
    pub fn clear(&mut self) {
        self.buttons.clear();
    }
    pub fn update(&mut self, touch_event: TouchEvent) -> Option<ButtonEvent> {
        match touch_event {
            TouchEvent::Down { x, y } | TouchEvent::Move { x, y } => {
                for (id, rect) in &self.buttons {
                    if rect.inside(x, y) {
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
    pub fn draw_buttons(&mut self, grid: &mut ScreenGrid) {
        for button in &self.buttons {
            let min = screen_pos_to_grid_pos(button.1.x_min, button.1.y_min);
            let max = screen_pos_to_grid_pos(button.1.x_max, button.1.y_max);

            let (fg, bg) = if let Some(active) = self.active_button {
                if active == *button.0 {
                    (MAGENTA, BASE3)
                } else {
                    (BASE3, MAGENTA)
                }
            } else {
                (BASE3, MAGENTA)
            };

            for x in min.0..max.0 {
                for y in min.1..max.1 {
                    grid.put_char(x, y, ' ', bg, bg);
                }
            }
            grid.write_str(min.0, min.1, button.0, fg, bg);
        }

        self.dirty = false;
    }
}
