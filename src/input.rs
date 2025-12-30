use crate::graphics::*;
use crate::touch::xpt2046_read_axis;
use esp_hal::delay::Delay;
use esp_hal::gpio::{Input, Output};
use esp_hal::spi::master::Spi;
use esp_hal::time::Instant;

// TODO: Move touch calibration out of input
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::DrawTarget;
use esp_hal::DriverMode;
use log::info;

pub enum TouchEvent {
    Down { x: u16, y: u16 },
    Move { x: u16, y: u16 },
    Up,
}

pub struct TouchCalibration {
    pub min_x: u16,
    pub min_y: u16,
    pub max_x: u16,
    pub max_y: u16,
}

pub struct TouchPoller<'a, DM: DriverMode> {
    calibration: TouchCalibration,
    t_irq: Input<'a>,
    touch_spi: Spi<'a, DM>,
    t_cs: Output<'a>,

    touch_down: bool,
}

impl<'a, DM: DriverMode> TouchPoller<'a, DM> {
    pub fn new(
        calibration: TouchCalibration,
        t_irq: Input<'a>,
        touch_spi: Spi<'a, DM>,
        t_cs: Output<'a>,
    ) -> Self {
        Self {
            calibration,
            t_irq,
            touch_spi,
            t_cs,
            touch_down: false,
        }
    }
    pub fn poll(&mut self) -> Option<TouchEvent> {
        if self.t_irq.is_low() {
            if let (Ok(x_raw), Ok(y_raw)) = (
                xpt2046_read_axis(&mut self.touch_spi, &mut self.t_cs, 0xD0),
                xpt2046_read_axis(&mut self.touch_spi, &mut self.t_cs, 0x90),
            ) {
                let (x, y) = map_touch(x_raw, y_raw, &self.calibration);
                if self.touch_down {
                    return Some(TouchEvent::Move { x, y });
                } else {
                    self.touch_down = true;
                    return Some(TouchEvent::Down { x, y });
                }
            }
        } else if self.touch_down {
            self.touch_down = false;
            return Some(TouchEvent::Up);
        }
        None
    }
}

fn map(raw: u16, min: u16, max: u16, out_max: u16) -> u16 {
    if max <= min {
        return 0;
    }

    let num = (raw - min) as u32 * out_max as u32;
    let den = (max - min) as u32;

    (num / den) as u16
}

fn map_touch(raw_x: u16, raw_y: u16, calibration: &TouchCalibration) -> (u16, u16) {
    let x = map(raw_x, calibration.min_x, calibration.max_x, 239);
    let y = map(raw_y, calibration.min_y, calibration.max_y, 319);
    (x, y)
}

pub fn calibrate_touch<D: DrawTarget<Color = Rgb565>, DM: DriverMode>(
    t_irq: &Input,
    mut touch_spi: &mut Spi<DM>,
    mut t_cs: &mut Output,
    screen_grid: &mut ScreenGrid,
    display: &mut D,
) -> TouchCalibration {
    let mut calibration = TouchCalibration {
        min_x: 0,
        min_y: 0,
        max_x: 0,
        max_y: 0,
    };

    let mut now = Instant::now();
    let mut old_input = false;
    let delay = Delay::new();
    let mut calibration_step = 0;
    while calibration_step < 4 {
        let state = t_irq.is_low();

        if state {
            if let (Ok(x_raw), Ok(y_raw)) = (
                xpt2046_read_axis(&mut touch_spi, &mut t_cs, 0xD0),
                xpt2046_read_axis(&mut touch_spi, &mut t_cs, 0x90),
            ) {
                info!("raw touch: x={} y={}", x_raw, y_raw);

                // TODO: Use all 4 corners for calibration
                if now.elapsed().as_millis() >= 600 && !old_input {
                    // Top Left
                    if calibration_step == 0 {
                        calibration.min_x = x_raw;
                        calibration.min_y = y_raw;
                    }
                    // Top Right
                    if calibration_step == 3 {
                        calibration.max_x = x_raw;
                        calibration.max_y = y_raw;
                    }

                    // Go to next calibration step.
                    calibration_step += 1;
                    old_input = true;
                }
            }
        } else {
            now = Instant::now();
            old_input = false;
        }

        let color = if state && !old_input { GREEN } else { BASE01 };

        screen_grid.clear(' ', BASE03, BASE03);
        screen_grid.write_str(11, 1, "TOUCH CALIBRATION", color, BASE03);

        match calibration_step {
            0 => {
                screen_grid.put_char(0, 0, ' ', color, color);
                screen_grid.put_char(0, 1, ' ', color, color);
                screen_grid.put_char(1, 0, ' ', color, color);
                screen_grid.put_char(1, 1, ' ', color, color);
            }
            1 => {
                screen_grid.put_char(37, 1, ' ', color, color);
                screen_grid.put_char(37, 2, ' ', color, color);
                screen_grid.put_char(38, 1, ' ', color, color);
                screen_grid.put_char(38, 2, ' ', color, color);
            }
            2 => {
                screen_grid.put_char(1, 29, ' ', color, color);
                screen_grid.put_char(2, 29, ' ', color, color);
                screen_grid.put_char(1, 30, ' ', color, color);
                screen_grid.put_char(2, 30, ' ', color, color);
            }
            3 => {
                screen_grid.put_char(38, 30, ' ', color, color);
                screen_grid.put_char(39, 30, ' ', color, color);
                screen_grid.put_char(38, 31, ' ', color, color);
                screen_grid.put_char(39, 31, ' ', color, color);
            }
            _ => {}
        }

        render_grid(display, &screen_grid);
        delay.delay_millis(200);
    }
    calibration
}
