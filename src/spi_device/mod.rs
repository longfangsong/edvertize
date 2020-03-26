use embedded_hal::digital::v2::OutputPin;

pub mod epd;
pub mod sd;

struct SPIDevice<
    SPI: embedded_hal::blocking::spi::Write<u8> + embedded_hal::blocking::spi::Transfer<u8>,
    T: OutputPin,
> {
    spi: SPI,
    chip_select_pin: T,
}

impl<
        SPI: embedded_hal::blocking::spi::Write<u8> + embedded_hal::blocking::spi::Transfer<u8>,
        T: OutputPin,
    > SPIDevice<SPI, T>
{
    pub fn select(&mut self) {
        self.chip_select_pin.set_low().ok().unwrap();
    }
    pub fn deselect(&mut self) {
        self.chip_select_pin.set_high().ok().unwrap();
    }
}
