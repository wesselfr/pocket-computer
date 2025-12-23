use core::convert::Infallible;
use embedded_hal::digital::OutputPin;
use embedded_hal::spi::SpiBus;

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
