use core::convert::Infallible;
use embedded_hal::digital::OutputPin;
use embedded_hal::spi::SpiBus;

use crate::graphics::*;
use esp_hal::delay::Delay;
use esp_hal::gpio::{Input, InputConfig, Level, Output, OutputConfig};
use esp_hal::spi::master::{Config, Spi};
use esp_hal::time::{Instant, Rate};

// TODO: Move touch calibration out of input
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::DrawTarget;
use esp_hal::{Blocking, DriverMode};
use log::info;

#[derive(PartialEq)]
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

impl Default for TouchCalibration {
    fn default() -> Self {
        Self {
            min_x: 314,
            min_y: 297,
            max_x: 3707,
            max_y: 3656,
        }
    }
}

pub const X_AXIS: u8 = 0xD0;
pub const Y_AXIS: u8 = 0x90;

/// Read one axis (X or Y) from XPT2046 using the given command
/// - cmd: 0xD0 for X, 0x90 for Y (12-bit differential mode)
pub fn xpt2046_read_axis<SPI, CS, E>(spi: &mut SPI, cs: &mut CS, cmd: u8) -> Result<u16, E>
where
    SPI: SpiBus<u8, Error = E>,
    CS: OutputPin<Error = Infallible>,
{
    // we will write 3 bytes and read 3 bytes
    let write = [cmd, 0x00, 0x00];
    let mut read = [0u8; 3];

    // CS low -> start transaction
    let _ = cs.set_low();
    spi.transfer(&mut read, &write)?;
    let _ = cs.set_high();

    // 12-bit result is in the top bits of buf[1..2]
    let value = (((read[1] as u16) << 8) | read[2] as u16) >> 3;
    Ok(value)
}

pub struct TouchPins {
    pub spi: esp_hal::peripherals::SPI2<'static>,
    pub sclk: esp_hal::peripherals::GPIO1<'static>,
    pub miso: esp_hal::peripherals::GPIO4<'static>,
    pub mosi: esp_hal::peripherals::GPIO3<'static>,
    pub cs: esp_hal::peripherals::GPIO2<'static>,
    pub irq: esp_hal::peripherals::GPIO9<'static>,
}

pub struct TouchDriver<'a> {
    pub t_irq: Input<'a>,
    pub touch_spi: Spi<'a, Blocking>,
    pub t_cs: Output<'a>,
}

impl<'a> TouchDriver<'a> {
    pub fn new(p: TouchPins) -> Self {
        let t_cs = Output::new(p.cs, Level::High, OutputConfig::default());
        let t_irq = Input::new(
            p.irq,
            InputConfig::default().with_pull(esp_hal::gpio::Pull::Up),
        );

        // SPI2 for touch (<= 2.5 MHz, Mode0)
        let touch_spi: Spi<'a, Blocking> = Spi::new(
            p.spi,
            Config::default()
                .with_frequency(Rate::from_mhz(2))
                .with_mode(esp_hal::spi::Mode::_0),
        )
        .unwrap()
        .with_sck(p.sclk)
        .with_miso(p.miso)
        .with_mosi(p.mosi);

        Self {
            t_irq,
            touch_spi,
            t_cs,
        }
    }
}

pub struct TouchPoller<'a> {
    calibration: TouchCalibration,
    driver: &'a mut TouchDriver<'a>,
    touch_down: bool,
}

impl<'a> TouchPoller<'a> {
    pub fn new(calibration: TouchCalibration, driver: &'a mut TouchDriver<'a>) -> Self {
        Self {
            calibration,
            driver,
            touch_down: false,
        }
    }
    pub fn poll(&mut self) -> Option<TouchEvent> {
        if self.driver.t_irq.is_low() {
            if let (Ok(x_raw), Ok(y_raw)) = (
                xpt2046_read_axis(&mut self.driver.touch_spi, &mut self.driver.t_cs, 0xD0),
                xpt2046_read_axis(&mut self.driver.touch_spi, &mut self.driver.t_cs, 0x90),
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
    if max <= min || raw < min {
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

        render_grid(display, screen_grid);
        delay.delay_millis(200);
    }
    calibration
}
