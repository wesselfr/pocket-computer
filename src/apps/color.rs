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
        ctx.buttons.clear();
        ctx.buttons.register_default_buttons();
        ctx.buttons.register_button(
            "NEXT",
            crate::input::Rect {
                x_min: 120,
                y_min: 120,
                x_max: 172,
                y_max: 140,
            },
        );

        AppCmd::Dirty
    }

    fn update(&mut self, event: Option<TouchEvent>, ctx: &mut Context) -> AppCmd {
        if let Some(event) = event {
            if let Some(button_event) = ctx.buttons.update(&event) {
                match button_event {
                    ButtonEvent::Up(id) => {
                        if id == "BACK" {
                            return AppCmd::SwitchApp(crate::apps::app::AppID::HomeApp);
                        }
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
                    _ => {}
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
    fn get_name(&self) -> &'static str {
        "COLOR"
    }
}
