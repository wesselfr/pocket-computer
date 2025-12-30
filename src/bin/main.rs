#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use esp_hal::clock::CpuClock;
use esp_hal::delay::Delay;
use esp_hal::gpio::{Input, InputConfig, Io, Level, Output, OutputConfig};
use esp_hal::main;
use esp_hal::spi::master::{Config, Spi};
use esp_hal::time::{Duration, Instant, Rate};

use esp_test::input::{TouchEvent, TouchPoller, calibrate_touch};
use esp_test::log::init_log;
use log::info;
use mipidsi::interface::{Generic8BitBus, ParallelInterface};
use mipidsi::options::Orientation;
use mipidsi::{Builder, models::ST7789, options::ColorOrder};

use esp_test::graphics::*;

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    log::error!("ERROR: {}", info.message());
    loop {}
}

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

#[main]
fn main() -> ! {
    init_log(log::LevelFilter::Info).expect("Failed to initialize logger...");
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

    let touch_calibration = calibrate_touch(
        &t_irq,
        &mut touch_spi,
        &mut t_cs,
        &mut screen_grid,
        &mut display,
    );
    let mut touch_poller = TouchPoller::new(touch_calibration, t_irq, touch_spi, t_cs);

    let mut flicker = false;
    let mut count: u16 = 0;

    // Timers
    let mut last_screen_refresh = Instant::now();
    let mut last_input = Instant::now();

    screen_grid.clear(' ', BASE03, BASE03);
    loop {
        if let Some(event) = touch_poller.poll() {
            match event {
                TouchEvent::Down { x, y } | TouchEvent::Move { x, y } => {
                    screen_grid.put_char(x / 6, y / 10, 'X', RED, VIOLET);
                    info!("Clicked on x: {}, y: {}", x, y);
                }
                TouchEvent::Up => {
                    info!("No longer touching.");
                }
            }
            last_input = Instant::now();
        }

        if last_screen_refresh.elapsed() > Duration::from_millis(33) {
            flicker = !flicker;
            count += 1;

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
            last_screen_refresh = Instant::now();
        }

        // TODO: Use a power manager here.
        if last_input.elapsed() > Duration::from_secs(10) {
            // No input detected for a while. Use low power mode.
            delay.delay_millis(1000);
        } else {
            // Default refresh rate
            delay.delay_millis(20);
        }
    }
}
