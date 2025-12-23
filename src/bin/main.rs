#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use embedded_graphics::pixelcolor::{Rgb555, Rgb565};
use embedded_graphics::prelude::DrawTarget;
use esp_hal::clock::CpuClock;
use esp_hal::delay::Delay;
use esp_hal::gpio::{Input, InputConfig, Io, Level, Output, OutputConfig};
use esp_hal::spi::master::{Config, Spi};
use esp_hal::time::{Duration, Instant, Rate};
use esp_hal::{DriverMode, main};
use esp_println::println;

use esp_test::touch::xpt2046_read_axis;
use mipidsi::interface::{Generic8BitBus, ParallelInterface};
use mipidsi::options::Orientation;
use mipidsi::{Builder, models::ST7789, options::ColorOrder};

use esp_test::graphics::*;

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    println!("ERROR: {}", info.message());
    loop {}
}

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

#[main]
fn main() -> ! {
    // generator version: 1.0.1
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);
    let output_config = OutputConfig::default();
    let _io = Io::new(peripherals.IO_MUX);

    // screen io ports
    let lcd_d0 = Output::new(peripherals.GPIO48, Level::Low, output_config);
    let lcd_d1 = Output::new(peripherals.GPIO47, Level::Low, output_config);
    let lcd_d2 = Output::new(peripherals.GPIO39, Level::Low, output_config);
    let lcd_d3 = Output::new(peripherals.GPIO40, Level::Low, output_config);
    let lcd_d4 = Output::new(peripherals.GPIO41, Level::Low, output_config);
    let lcd_d5 = Output::new(peripherals.GPIO42, Level::Low, output_config);
    let lcd_d6 = Output::new(peripherals.GPIO45, Level::Low, output_config);
    let lcd_d7 = Output::new(peripherals.GPIO46, Level::Low, output_config);

    let lcd_wr = Output::new(peripherals.GPIO8, Level::High, output_config);
    // let _lcd_rd = Output::new(peripherals.GPIO9, Level::High, output_config);
    let lcd_dc = Output::new(peripherals.GPIO7, Level::Low, output_config);
    let _lcd_cs = Output::new(peripherals.GPIO6, Level::Low, output_config);
    let mut _lcd_backlight = Output::new(peripherals.GPIO38, Level::High, output_config);
    let _lcd_reset = Output::new(peripherals.GPIO5, Level::High, output_config);

    // Power control (must be ON)
    let _lcd_pwr_en = Output::new(peripherals.GPIO10, Level::High, output_config);
    let _lcd_pwr_on = Output::new(peripherals.GPIO14, Level::High, output_config);

    // Display interface
    let bus = Generic8BitBus::new((
        lcd_d0, lcd_d1, lcd_d2, lcd_d3, lcd_d4, lcd_d5, lcd_d6, lcd_d7,
    ));
    let interface = ParallelInterface::new(bus, lcd_dc, lcd_wr);

    let mut delay = Delay::new();

    let mut display = Builder::new(ST7789, interface)
        // .reset_pin(lcd_reset)
        .color_order(ColorOrder::Rgb) // this board uses BGR order
        .display_size(240, 320) // 240x320 panel
        .orientation(Orientation::new())
        .init(&mut delay)
        .unwrap();

    let mut screen_buffer = [Cell::default(); (240 / 6) * (320 / 10)];
    let mut screen_grid = ScreenGrid::new(240 / 6, 320 / 10, &mut screen_buffer);

    // --- Touch SPI pins ---
    let sclk = peripherals.GPIO1;
    let miso = peripherals.GPIO4;
    let mosi = peripherals.GPIO3;

    // XPT2046 chip select (manual CS)
    let mut t_cs = Output::new(peripherals.GPIO2, Level::High, output_config);

    // IRQ line: active LOW when pressed
    let t_irq = Input::new(
        peripherals.GPIO9,
        InputConfig::default().with_pull(esp_hal::gpio::Pull::Up),
    );

    // SPI for touch (<= 2.5 MHz, Mode0)
    let mut touch_spi = Spi::new(
        peripherals.SPI2,
        Config::default()
            .with_frequency(Rate::from_mhz(2))
            .with_mode(esp_hal::spi::Mode::_0),
    )
    .unwrap()
    .with_sck(sclk)
    .with_miso(miso)
    .with_mosi(mosi);

    let mut touch_calibration = None;

    let mut flicker = false;
    let mut count: u16 = 0;

    screen_grid.clear(' ', BASE03, BASE03);
    loop {
        let delay_start = Instant::now();
        if touch_calibration.is_none() {
            touch_calibration = Some(calibrate_touch(
                &t_irq,
                &mut touch_spi,
                &mut t_cs,
                &mut screen_grid,
                &mut display,
            ));
            screen_grid.clear(' ', BASE03, BASE03);
        }
        while delay_start.elapsed() < Duration::from_millis(500) {
            println!("Hello world!");
            flicker = !flicker;
            count += 1;

            if t_irq.is_low() {
                // Take a couple of samples to smooth noise if you like
                if let (Ok(x_raw), Ok(y_raw)) = (
                    xpt2046_read_axis(&mut touch_spi, &mut t_cs, 0xD0),
                    xpt2046_read_axis(&mut touch_spi, &mut t_cs, 0x90),
                ) {
                    println!("raw touch: x={} y={}", x_raw, y_raw);

                    // Later: map raw 0..4095 to 0..239 / 0..319
                    // let x = map(x_raw as i32, X_MIN, X_MAX, 0, 239);
                    // let y = map(y_raw as i32, Y_MIN, Y_MAX, 0, 319);
                }
            }

            screen_grid.put_char(0, 0, ' ', BASE03, YELLOW);
            screen_grid.put_char(1, 0, ' ', BASE03, ORANGE);
            screen_grid.put_char(2, 0, ' ', BASE03, RED);
            screen_grid.put_char(3, 0, ' ', BASE03, MAGENTA);
            screen_grid.put_char(4, 0, ' ', BASE03, VIOLET);
            screen_grid.put_char(5, 0, ' ', BASE03, BLUE);
            screen_grid.put_char(6, 0, ' ', BASE03, CYAN);
            screen_grid.put_char(7, 0, ' ', BASE03, GREEN);

            if flicker {
                screen_grid.write_str(0, 3, "Hello Rust!", BASE03, RED);
            } else {
                screen_grid.write_str(0, 3, "Hello Rust!", BASE2, BASE03);
            }

            for i in 0..(count / 5).min(32) {
                screen_grid.write_str(0, 4 + i, "another one!", BASE1, BASE03);
            }
            if count / 5 > 5 {
                count = 0;
                screen_grid.clear(' ', BASE03, BASE03);
            }

            render_grid(&mut display, &screen_grid).unwrap();
            delay.delay_millis(500);
        }
    }
}

struct TouchCalibration {
    pub min_x: u16,
    pub min_y: u16,
    pub max_x: u16,
    pub max_y: u16,
}

fn calibrate_touch<D: DrawTarget<Color = Rgb565>, DM: DriverMode>(
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
                println!("raw touch: x={} y={}", x_raw, y_raw);

                // Later: map raw 0..4095 to 0..239 / 0..319
                // let x = map(x_raw as i32, X_MIN, X_MAX, 0, 239);
                // let y = map(y_raw as i32, Y_MIN, Y_MAX, 0, 319);
            }
            if now.elapsed().as_millis() >= 600 && !old_input {
                // Go to next calibration step.
                calibration_step += 1;
                old_input = true;
                // if calibration_step >= 4 {
                //     calibration_step = 0;
                // }
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
                screen_grid.put_char(1, 1, ' ', color, color);
                screen_grid.put_char(1, 2, ' ', color, color);
                screen_grid.put_char(2, 1, ' ', color, color);
                screen_grid.put_char(2, 2, ' ', color, color);
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
                screen_grid.put_char(37, 29, ' ', color, color);
                screen_grid.put_char(38, 29, ' ', color, color);
                screen_grid.put_char(37, 30, ' ', color, color);
                screen_grid.put_char(38, 30, ' ', color, color);
            }
            _ => {}
        }

        render_grid(display, &screen_grid);
        delay.delay_millis(200);
    }
    calibration
}
