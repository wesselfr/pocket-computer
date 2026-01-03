use crate::touch::TouchEvent;
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
}

impl ButtonManager {
    pub fn new() -> Self {
        Self {
            active_button: None,
            buttons: FnvIndexMap::<ButtonId, Rect, 16>::new(),
        }
    }
    pub fn register_button(&mut self, name: ButtonId, rect: Rect) {
        self.buttons
            .insert(name, rect)
            .expect("Failed to add button");
    }
    pub fn update(&mut self, touch_event: TouchEvent) -> Option<ButtonEvent> {
        match touch_event {
            TouchEvent::Down { x, y } | TouchEvent::Move { x, y } => {
                for (id, rect) in &self.buttons {
                    if rect.inside(x, y) {
                        self.active_button = Some(*id);
                        return Some(ButtonEvent::Down(*id));
                    }
                }
            }
            TouchEvent::Up => {
                if let Some(previous_button) = self.active_button {
                    self.active_button = None;
                    return Some(ButtonEvent::Up(previous_button));
                }
            }
        }

        None
    }
}
