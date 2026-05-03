#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use embassy_executor::Spawner;
use esp_hal::clock::CpuClock;
use esp_hal::gpio::OutputConfig;
use esp_hal::interrupt::software::SoftwareInterruptControl;
use esp_hal::ledc::timer::*;
use esp_hal::ledc::{Ledc, LowSpeed};
use esp_hal::time::{Instant, Rate};
use esp_hal::timer::timg::TimerGroup;
use pocket_computer::display::{DisplayDriver, DisplayPins};
use pocket_computer::power::{PowerManager, PowerMode};
use pocket_computer::storage::Storage;
use pocket_computer::tasks::{TaskContext, TaskSystem};

use core::cell::RefCell;
use log::info;
use mem_fs::MemFs;
use pocket_computer::apps::AppState;
use pocket_computer::apps::home::HomeApp;
use pocket_computer::input::{ButtonEvent, ButtonManager};
use pocket_computer::log::init_log;
use pocket_computer::system::{SettingsView, SystemCmd, SystemSettings};
use pocket_computer::touch::{TouchCalibration, TouchDriver, TouchPins, TouchPoller};

use pocket_computer::apps::app::{App, AppArgs, AppCmd, AppID, Context, InputEvents};
use pocket_computer::graphics::*;

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    log::error!("ERROR: {}", info.message());
    loop {}
}

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

#[esp_rtos::main]
async fn main(spawner: Spawner) {
    init_log(log::LevelFilter::Info).expect("Failed to initialize logger...");
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);
    let output_config = OutputConfig::default();

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let sw_int = SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);
    esp_rtos::start(timg0.timer0, sw_int.software_interrupt0);

    let mut ledc = Ledc::new(peripherals.LEDC);
    ledc.set_global_slow_clock(esp_hal::ledc::LSGlobalClkSource::APBClk);

    let mut lstimer0 = ledc.timer::<LowSpeed>(esp_hal::ledc::timer::Number::Timer0);
    lstimer0
        .configure(esp_hal::ledc::timer::config::Config {
            duty: esp_hal::ledc::timer::config::Duty::Duty10Bit,
            clock_source: esp_hal::ledc::timer::LSClockSource::APBClk,
            frequency: Rate::from_khz(20),
        })
        .expect("Failed to create Timer..");

    let mut display_driver = DisplayDriver::init(
        DisplayPins {
            d0: peripherals.GPIO48,
            d1: peripherals.GPIO47,
            d2: peripherals.GPIO39,
            d3: peripherals.GPIO40,
            d4: peripherals.GPIO41,
            d5: peripherals.GPIO42,
            d6: peripherals.GPIO45,
            d7: peripherals.GPIO46,
            wr: peripherals.GPIO8,
            dc: peripherals.GPIO7,
            backlight: peripherals.GPIO38,
            pwr_en: peripherals.GPIO10,
            pwr_on: peripherals.GPIO14,
        },
        output_config,
        &lstimer0,
    );

    let mut screen_buffer = [Cell::default(); ((SCREEN_W / CELL_W) * (SCREEN_H / CELL_H)) as usize];
    let mut screen_grid = ScreenGrid::new(SCREEN_W / CELL_W, SCREEN_H / CELL_H, &mut screen_buffer);

    let mut touch_driver = TouchDriver::new(TouchPins {
        spi: peripherals.SPI2,
        sclk: peripherals.GPIO1,
        miso: peripherals.GPIO4,
        mosi: peripherals.GPIO3,
        cs: peripherals.GPIO2,
        irq: peripherals.GPIO9,
    });

    // TODO: Load or calibrate touch here.
    let touch_calibration = TouchCalibration::default();
    let mut touch_poller = TouchPoller::new(touch_calibration, &mut touch_driver);

    let mut button_manager = ButtonManager::new();
    button_manager.register_default_buttons();

    // Timers
    let mut last_render_time = 0;

    let mut storage = Storage::new(peripherals.FLASH);
    let mut fs = if let Ok(restored_fs) = storage.restore_memfs() {
        restored_fs
    } else {
        MemFs::new()
    };

    let settings = RefCell::new(SystemSettings::default());
    let mut power_manager = PowerManager::new();

    // TODO: Init on second core.
    let mut task_system = TaskSystem::new();
    task_system.init(spawner, TaskContext { storage });

    let mut active_app = AppState::Home(HomeApp::default());
    let mut ctx = Context {
        grid: &mut screen_grid,
        buttons: &mut button_manager,
        settings: SettingsView::new(&settings),
        fs: &mut fs,
        tasks: &mut task_system,
    };

    active_app.init(&mut ctx, AppArgs::None);
    display_driver.set_backlight(settings.borrow().user_brightness);
    loop {
        let update_time = Instant::now();

        // TODO: Move to seperate core.
        ctx.tasks.update();

        let touch_event = touch_poller.poll();
        if touch_event.is_some() {
            power_manager.register_activity();
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
                active_app = active_app.switch(AppID::HomeApp);
                dirty = active_app.init(&mut ctx, AppArgs::None).app == AppCmd::Dirty;
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
            AppCmd::SwitchApp(app, args) => {
                active_app = active_app.switch(app);
                active_app.init(&mut ctx, args).app == AppCmd::Dirty
            }
        };

        if let Some(cmd) = response.system {
            match cmd {
                SystemCmd::SetBrightness(val) => {
                    let mut s = settings.borrow_mut();
                    s.user_brightness = val;
                    display_driver.set_backlight(s.user_brightness);
                    lstimer0.update_hw();
                }
                _ => {}
            }
        }

        let dirty = dirty || ctx.buttons.is_dirty();

        if dirty && power_manager.get_power_mode() != PowerMode::Sleep {
            let render_time = Instant::now();
            active_app.render(&mut ctx);
            draw_status_bars(&mut ctx.grid, active_app.get_name(), last_render_time);
            ctx.buttons.draw_buttons(ctx.grid);
            render_grid(display_driver.display_mut(), &mut ctx.grid).unwrap();

            last_render_time = render_time.elapsed().as_millis();
            info!("Rendering took: {} ms", last_render_time);
        }

        // TODO: Handle system cmd's in a uniform way
        if let Some(cmd) = power_manager.update(&settings) {
            if let SystemCmd::SetBrightness(val) = cmd {
                display_driver.set_backlight(val);
            }
        }

        let update_time = update_time.elapsed().as_millis();
        if update_time > 0 {
            info!("Total update took: {}ms", update_time);
        }
        power_manager.await_frame().await;
    }
}
