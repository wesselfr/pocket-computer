use embedded_graphics::pixelcolor::Rgb565;
use log::info;

use crate::{
    apps::app::{App, AppCmd, Context},
    graphics::*,
    input::ButtonEvent,
    touch::TouchEvent,
};

pub struct ColorApp {
    colors: [(&'static str, Rgb565); 8],
    selected: u16,
}

impl Default for ColorApp {
    fn default() -> Self {
        Self {
            colors: [
                ("Yellow", YELLOW),
                ("Orange", ORANGE),
                ("Red", RED),
                ("Magenta", MAGENTA),
                ("Violet", VIOLET),
                ("Blue", BLUE),
                ("Cyan", CYAN),
                ("Green", GREEN),
            ],
            selected: 0,
        }
    }
}

impl App for ColorApp {
    fn init(&mut self, ctx: &mut Context) -> AppCmd {
        ctx.buttons.register_button(
            "NEXT",
            crate::input::Rect {
                x_min: 0,
                y_min: 100,
                x_max: 400,
                y_max: 400,
            },
        );

        AppCmd::Dirty
    }

    fn update(&mut self, event: Option<TouchEvent>, ctx: &mut Context) -> AppCmd {
        if let Some(event) = event {
            if let Some(button_event) = ctx.buttons.update(event) {
                match button_event {
                    ButtonEvent::Down(id) => {
                        if id == "BACK" {
                            return AppCmd::SwitchApp(crate::apps::app::AppID::HomeApp);
                        }
                    }
                    ButtonEvent::Up(id) => {
                        if id == "NEXT" {
                            info!("NEXT");
                            self.selected += 1;
                            info!("SELECTED: {}", self.selected);
                            if self.selected >= self.colors.len() as u16 {
                                self.selected = 0;
                                info!("RESET");
                            }
                            return AppCmd::Dirty;
                        }
                    }
                }
            }
        }
        AppCmd::None
    }
    fn render(&mut self, ctx: &mut Context) {
        ctx.grid.clear(' ', BASE03, BASE03);

        let x_offset = 12;
        let y_offset = 8;
        for (x, color) in self.colors.iter().enumerate() {
            for y in 0..2 {
                ctx.grid
                    .put_char(x_offset + x as u16 * 2, y_offset + y, ' ', BASE03, color.1);
                ctx.grid.put_char(
                    x_offset + x as u16 * 2 + 1,
                    y_offset + y,
                    ' ',
                    BASE03,
                    color.1,
                );
            }
            if x as u16 == self.selected {
                ctx.grid
                    .put_char(x_offset + x as u16 * 2, y_offset + 2, '/', color.1, BASE03);
                ctx.grid.put_char(
                    x_offset + x as u16 * 2 + 1,
                    y_offset + 2,
                    '\\',
                    color.1,
                    BASE03,
                );

                ctx.grid
                    .write_str(x_offset, y_offset + 4, color.0, color.1, BASE03);
                ctx.grid
                    .write_str(x_offset, y_offset + 5, color.0, BASE03, color.1);
            }
        }
    }
}
