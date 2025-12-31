use esp_println::println;
use log::{Level, LevelFilter, Metadata, Record, SetLoggerError};

static LOGGER: Logger = Logger;

struct Logger;

impl log::Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            println!("[{}] - {}", record.level(), record.args());
        }
    }

    fn flush(&self) {}
}

pub fn init_log(level: LevelFilter) -> Result<(), SetLoggerError> {
    log::set_logger(&LOGGER).map(|()| log::set_max_level(level))
}
