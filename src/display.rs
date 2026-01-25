use esp_hal::ledc::LowSpeed;
use esp_hal::ledc::channel::{Channel, ChannelHW, ChannelIFace};
use esp_hal::ledc::timer::*;
use esp_hal::{
    delay::Delay,
    gpio::{Level, Output, OutputConfig},
};
use log::error;
use mipidsi::{
    Builder, Display, NoResetPin,
    interface::{Generic8BitBus, ParallelInterface},
    models::ST7789,
    options::{ColorOrder, Orientation},
};

pub struct DisplayPins {
    pub d0: esp_hal::peripherals::GPIO48<'static>,
    pub d1: esp_hal::peripherals::GPIO47<'static>,
    pub d2: esp_hal::peripherals::GPIO39<'static>,
    pub d3: esp_hal::peripherals::GPIO40<'static>,
    pub d4: esp_hal::peripherals::GPIO41<'static>,
    pub d5: esp_hal::peripherals::GPIO42<'static>,
    pub d6: esp_hal::peripherals::GPIO45<'static>,
    pub d7: esp_hal::peripherals::GPIO46<'static>,
    pub wr: esp_hal::peripherals::GPIO8<'static>,
    pub dc: esp_hal::peripherals::GPIO7<'static>,
    pub backlight: esp_hal::peripherals::GPIO38<'static>,
    pub pwr_en: esp_hal::peripherals::GPIO10<'static>,
    pub pwr_on: esp_hal::peripherals::GPIO14<'static>,
}

// Concrete type aliases hidden in this module:
type LcdBus = Generic8BitBus<
    Output<'static>,
    Output<'static>,
    Output<'static>,
    Output<'static>,
    Output<'static>,
    Output<'static>,
    Output<'static>,
    Output<'static>,
>;

type LcdInterface = ParallelInterface<
    LcdBus,
    Output<'static>, // DC
    Output<'static>, // WR
>;

type LcdDisplay = Display<LcdInterface, ST7789, NoResetPin>;

pub struct DisplayDriver<'a> {
    display: LcdDisplay,
    backlight_channel: Channel<'a, LowSpeed>,
}

impl<'a> DisplayDriver<'a> {
    pub fn init(
        pins: DisplayPins,
        output_config: OutputConfig,
        low_speed_timer: &'a Timer<'a, LowSpeed>,
    ) -> Self {
        // Data pins
        let lcd_d0 = Output::new(pins.d0, Level::Low, output_config);
        let lcd_d1 = Output::new(pins.d1, Level::Low, output_config);
        let lcd_d2 = Output::new(pins.d2, Level::Low, output_config);
        let lcd_d3 = Output::new(pins.d3, Level::Low, output_config);
        let lcd_d4 = Output::new(pins.d4, Level::Low, output_config);
        let lcd_d5 = Output::new(pins.d5, Level::Low, output_config);
        let lcd_d6 = Output::new(pins.d6, Level::Low, output_config);
        let lcd_d7 = Output::new(pins.d7, Level::Low, output_config);

        // Control pins
        let lcd_wr = Output::new(pins.wr, Level::High, output_config);
        let lcd_dc = Output::new(pins.dc, Level::Low, output_config);

        // Backlight
        let backlight = Output::new(pins.backlight, Level::High, output_config);

        // Power control (must be ON)
        let _lcd_pwr_en = Output::new(pins.pwr_en, Level::High, output_config);
        let _lcd_pwr_on = Output::new(pins.pwr_on, Level::High, output_config);

        // Build bus + interface
        let bus = Generic8BitBus::new((
            lcd_d0, lcd_d1, lcd_d2, lcd_d3, lcd_d4, lcd_d5, lcd_d6, lcd_d7,
        ));
        let interface = ParallelInterface::new(bus, lcd_dc, lcd_wr);

        // Init display
        let mut delay = Delay::new();
        let display: LcdDisplay = Builder::new(ST7789, interface)
            .color_order(ColorOrder::Rgb)
            .display_size(240, 320)
            .orientation(Orientation::new())
            .init(&mut delay)
            .unwrap();

        // Backlight controls
        let mut channel0: esp_hal::ledc::channel::Channel<'_, LowSpeed> =
            esp_hal::ledc::channel::Channel::new(
                esp_hal::ledc::channel::Number::Channel0,
                backlight,
            );

        channel0
            .configure(esp_hal::ledc::channel::config::Config {
                timer: low_speed_timer,
                duty_pct: 100,
                drive_mode: esp_hal::gpio::DriveMode::PushPull,
            })
            .unwrap();

        Self {
            display,
            backlight_channel: channel0,
        }
    }

    pub fn display_mut(&mut self) -> &mut LcdDisplay {
        &mut self.display
    }

    pub fn set_backlight(&mut self, brightness: u8) {
        let brightness = if brightness == 100 { 99 } else { brightness };
        let res = self.backlight_channel.set_duty(brightness);
        match res {
            Ok(_) => {}
            Err(e) => error!("{:?}", e),
        }

        self.backlight_channel
            .configure_hw()
            .expect("Failed to configure..");
    }
}
