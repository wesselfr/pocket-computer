#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use esp_hal::clock::CpuClock;
use esp_hal::delay::Delay;
use esp_hal::gpio::{Io, Level, Output, OutputConfig};
use esp_hal::main;
use esp_hal::time::{Duration, Instant};
use esp_println::println;

use embedded_graphics::{
    draw_target::DrawTarget,
    mono_font::{MonoTextStyle, ascii::FONT_6X10},
    pixelcolor::Rgb565,
    prelude::*,
    text::Text,
};
use mipidsi::interface::{Generic8BitBus, ParallelInterface};
use mipidsi::{Builder, models::ST7789, options::ColorOrder};

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
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
    let _lcd_rd = Output::new(peripherals.GPIO9, Level::High, output_config);
    let lcd_dc = Output::new(peripherals.GPIO7, Level::Low, output_config);
    let _lcd_cs = Output::new(peripherals.GPIO6, Level::Low, output_config);
    let _lcd_backlight = Output::new(peripherals.GPIO38, Level::High, output_config);
    let _lcd_reset = Output::new(peripherals.GPIO5, Level::High, output_config);
    // let _lcd_power_on = Output::new(peripherals.GPIO14, Level::High, output_config);

    // Power control (must be ON)
    let _lcd_pwr_en = Output::new(peripherals.GPIO10, Level::High, output_config);
    let _lcd_pwr_on = Output::new(peripherals.GPIO14, Level::High, output_config);

    let bus = Generic8BitBus::new((
        lcd_d0, lcd_d1, lcd_d2, lcd_d3, lcd_d4, lcd_d5, lcd_d6, lcd_d7,
    ));
    let interface = ParallelInterface::new(bus, lcd_dc, lcd_wr);

    let mut delay = Delay::new();

    let mut display = Builder::new(ST7789, interface)
        // .reset_pin(lcd_reset)
        .color_order(ColorOrder::Bgr) // this board uses BGR order
        .display_size(240, 320) // 240x320 panel
        .init(&mut delay)
        .unwrap();

    // Test: clear to black
    display.clear(Rgb565::WHITE).unwrap();

    // Create a new character style
    let style = MonoTextStyle::new(&FONT_6X10, Rgb565::WHITE);

    let mut flicker = false;

    loop {
        let delay_start = Instant::now();
        while delay_start.elapsed() < Duration::from_millis(500) {
            println!("Hello world!");
            flicker = !flicker;
            if flicker {
                display.clear(Rgb565::WHITE).unwrap();
            } else {
                display.clear(Rgb565::BLACK).unwrap();
                Text::new("Hello Rust!", Point::new(20, 30), style)
                    .draw(&mut display)
                    .unwrap();
            }
            delay.delay_millis(500);
        }
    }

    // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/esp-hal-v1.0.0/examples/src/bin
}
