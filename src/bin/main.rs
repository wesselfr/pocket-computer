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
use esp_hal::ledc::channel::{ChannelHW, ChannelIFace};
use esp_hal::ledc::timer::*;
use esp_hal::ledc::{Ledc, LowSpeed};
use esp_hal::main;
use esp_hal::spi::master::{Config, Spi};
use esp_hal::time::{Duration, Instant, Rate};

use log::{error, info};
use mipidsi::interface::{Generic8BitBus, ParallelInterface};
use mipidsi::options::Orientation;
use mipidsi::{Builder, models::ST7789, options::ColorOrder};
use pocket_computer::apps::AppState;
use pocket_computer::apps::home::HomeApp;
use pocket_computer::input::{ButtonEvent, ButtonManager};
use pocket_computer::log::init_log;
use pocket_computer::system::SystemCmd;
use pocket_computer::touch::{TouchCalibration, TouchPoller};

use pocket_computer::apps::app::{App, AppCmd, Context, InputEvents};
use pocket_computer::graphics::*;

fn set_backlight_u8(ch: &mut esp_hal::ledc::channel::Channel<'_, LowSpeed>, value: u8) {
    let value = if value == 100 { 99 } else { value };
    let res = ch.set_duty(value);
    match res {
        Ok(_) => {}
        Err(e) => error!("{:?}", e),
    }

    ch.configure_hw().expect("Failed to configure..");
}

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

    let mut screen_buffer = [Cell::default(); ((SCREEN_W / CELL_W) * (SCREEN_H / CELL_H)) as usize];
    let mut screen_grid = ScreenGrid::new(SCREEN_W / CELL_W, SCREEN_H / CELL_H, &mut screen_buffer);

    // --- Touch SPI pins ---
    let sclk = peripherals.GPIO1;
    let miso = peripherals.GPIO4;
    let mosi = peripherals.GPIO3;

    // XPT2046 chip select (manual CS)
    let t_cs = Output::new(peripherals.GPIO2, Level::High, output_config);

    // IRQ line: active LOW when pressed
    let t_irq = Input::new(
        peripherals.GPIO9,
        InputConfig::default().with_pull(esp_hal::gpio::Pull::Up),
    );

    // SPI for touch (<= 2.5 MHz, Mode0)
    let touch_spi = Spi::new(
        peripherals.SPI2,
        Config::default()
            .with_frequency(Rate::from_mhz(2))
            .with_mode(esp_hal::spi::Mode::_0),
    )
    .unwrap()
    .with_sck(sclk)
    .with_miso(miso)
    .with_mosi(mosi);

    // TODO: Load or calibrate touch here.
    let touch_calibration = TouchCalibration::default();
    let mut touch_poller = TouchPoller::new(touch_calibration, t_irq, touch_spi, t_cs);

    let mut button_manager = ButtonManager::new();
    button_manager.register_default_buttons();

    // Timers
    let mut last_screen_refresh = Instant::now();
    let mut last_input = Instant::now();
    let mut last_render_time = 0;

    let mut active_app = AppState::Home(HomeApp::default());
    let mut ctx = Context {
        grid: &mut screen_grid,
        buttons: &mut button_manager,
    };

    // HACK: BACKLIGHT TEST
    let mut ledc = Ledc::new(peripherals.LEDC);
    ledc.set_global_slow_clock(esp_hal::ledc::LSGlobalClkSource::APBClk);

    info!("LEDC");

    let mut lstimer0 = ledc.timer::<LowSpeed>(esp_hal::ledc::timer::Number::Timer0);
    lstimer0
        .configure(esp_hal::ledc::timer::config::Config {
            duty: esp_hal::ledc::timer::config::Duty::Duty10Bit,
            clock_source: esp_hal::ledc::timer::LSClockSource::APBClk,
            frequency: Rate::from_khz(20),
        })
        .expect("Failed to create Timer..");

    info!("TIMER");

    let mut channel0: esp_hal::ledc::channel::Channel<'_, LowSpeed> =
        esp_hal::ledc::channel::Channel::new(
            esp_hal::ledc::channel::Number::Channel0,
            _lcd_backlight,
        );

    info!("CHANNEL");

    channel0
        .configure(esp_hal::ledc::channel::config::Config {
            timer: &lstimer0,
            duty_pct: 100,
            drive_mode: esp_hal::gpio::DriveMode::PushPull,
        })
        .unwrap();

    let mut brightness = 100;
    active_app.init(&mut ctx);
    set_backlight_u8(&mut channel0, brightness);
    loop {
        let touch_event = touch_poller.poll();
        if touch_event.is_some() {
            last_input = Instant::now();
        }
        let button_event = if let Some(touch_event) = &touch_event {
            ctx.buttons.update(touch_event)
        } else {
            None
        };

        let mut dirty = false;
        // Check navigation buttons
        if let Some(ButtonEvent::Up(id)) = button_event {
            if id == "BACK" {
                active_app = active_app.switch(pocket_computer::apps::app::AppID::HomeApp);
                dirty = active_app.init(&mut ctx).app == AppCmd::Dirty;
            }
        };

        let response = active_app.update(
            InputEvents {
                touch: touch_event,
                button: button_event,
            },
            &mut ctx,
        );

        dirty |= match response.app {
            AppCmd::None => false,
            AppCmd::Dirty => true,
            AppCmd::SwitchApp(app) => {
                active_app = active_app.switch(app);
                active_app.init(&mut ctx).app == AppCmd::Dirty
            }
        };

        if let Some(cmd) = response.system {
            match cmd {
                SystemCmd::SetBrightness(val) => {
                    brightness = val;
                    set_backlight_u8(&mut channel0, brightness);
                    lstimer0.update_hw();
                }
                _ => {}
            }
        }

        let dirty = dirty || ctx.buttons.is_dirty();

        if dirty {
            let render_time = Instant::now();
            active_app.render(&mut ctx);
            draw_status_bars(&mut ctx.grid, active_app.get_name(), last_render_time);
            ctx.buttons.draw_buttons(ctx.grid);
            render_grid(&mut display, &mut ctx.grid).unwrap();

            last_render_time = render_time.elapsed().as_millis();
            info!("Rendering took: {} ms", last_render_time);
            last_screen_refresh = Instant::now();
        }

        let idle = last_input.elapsed() > Duration::from_secs(10)
            || last_screen_refresh.elapsed() > Duration::from_secs(5);

        if idle {
            set_backlight_u8(&mut channel0, brightness / 2);
        } else {
            set_backlight_u8(&mut channel0, brightness);
        }

        delay.delay_millis(if idle { 250 } else { 33 });
    }
}
