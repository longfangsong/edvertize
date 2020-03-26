use alloc::vec::Vec;
use core::iter::Peekable;
use core::slice::Iter;
use cortex_m_semihosting::hprintln;

use crate::spi_device::SPIDevice;
use cortex_m::asm;
use cortex_m::asm::bkpt;
use cortex_m::peripheral::SYST;
use embedded_hal::blocking::spi::Write;
use embedded_hal::digital::v2::{InputPin, OutputPin};
use embedded_hal::timer::CountDown;
use stm32f1xx_hal::spi::Spi;
use stm32f1xx_hal::time::U32Ext;
use stm32f1xx_hal::timer::Timer;

pub struct EPD<
    SPI: embedded_hal::blocking::spi::transfer::Default<u8>
        + embedded_hal::blocking::spi::write::Default<u8>,
    T: OutputPin,
    U: OutputPin,
    V: OutputPin,
    W: InputPin,
> {
    base: SPIDevice<SPI, U>,
    reset_pin: T,
    data_command_pin: V,
    busy_pin: W,
}

impl<
        SPI: embedded_hal::blocking::spi::transfer::Default<u8>
            + embedded_hal::blocking::spi::write::Default<u8>,
        T: OutputPin,
        U: OutputPin,
        V: OutputPin,
        W: InputPin,
    > EPD<SPI, T, U, V, W>
{
    pub fn new(
        spi: SPI,
        reset_pin: T,
        chip_select_pin: U,
        data_command_pin: V,
        busy_pin: W,
    ) -> Self {
        let mut result = EPD {
            base: SPIDevice {
                spi,
                chip_select_pin,
            },
            reset_pin,
            data_command_pin,
            busy_pin,
        };
        result.reset();
        result.send_command(0x01);
        result.send_batch_data(&[0x07u8, 0x07u8, 0x3fu8, 0x3fu8]);
        result.send_command(0x04);
        result.sleep(100);
        while result.is_busy() {
            result.sleep(1);
        }
        result.send_command(0x00);
        result.send_data(0x0f);
        result.send_command(0x61);
        result.send_batch_data(&[0x03u8, 0x20, 0x01, 0xE0]);
        result.send_command(0x15);
        result.send_data(0x00);
        result.send_command(0x50);
        result.send_batch_data(&[0x11, 0x07]);
        result.send_command(0x60);
        result.send_data(0x22);
        result
    }
    fn reset(&mut self) {
        self.base.select();
        self.sleep(200);
        self.reset_pin.set_low();
        self.sleep(4);
        self.reset_pin.set_high();
        self.base.deselect();
        self.sleep(200);
    }
    fn send_command(&mut self, command: u8) {
        self.data_command_pin.set_low();
        self.base.select();
        self.base.spi.write(&[command]);
        self.base.deselect();
    }
    fn send_data(&mut self, data: u8) {
        self.data_command_pin.set_high();
        self.base.select();
        self.base.spi.write(&[data]);
        self.base.deselect();
    }
    fn send_batch_data(&mut self, data: &[u8]) {
        self.data_command_pin.set_high();
        self.base.select();
        self.base.spi.write(&data);
        self.base.deselect();
    }
    fn is_busy(&mut self) -> bool {
        self.send_command(0x71);
        self.busy_pin.is_low().unwrap_or(true)
    }
    pub fn display<Iter1: Iterator<Item = Vec<u8>>>(&mut self, black: Iter1) {
        self.send_command(0x10);
        black.for_each(|it| self.send_batch_data(&it));
        self.send_command(0x13);

        [{
            let mut vec = Vec::new();
            vec.resize(512, 0u8);
            vec
        }]
        .iter()
        .cycle()
        .take(93)
        .for_each(|it| self.send_batch_data(&it));
        self.send_batch_data(&{
            let mut vec = Vec::new();
            vec.resize(384, 0u8);
            vec
        });
        self.send_command(0x12);
        self.sleep(100);
        while self.is_busy() {
            self.sleep(1);
        }
    }
    pub fn halt(&mut self) {
        self.send_command(0x02);
        while self.is_busy() {
            self.sleep(1);
        }
        self.send_command(0x07);
        self.send_data(0xa5);
    }
    fn sleep(&mut self, ms: u32) {
        for _ in 0..900 * ms {
            // asm::wfi();
        }
    }
}

impl<
        SPI: embedded_hal::blocking::spi::transfer::Default<u8>
            + embedded_hal::blocking::spi::write::Default<u8>,
        T: OutputPin,
        U: OutputPin,
        V: OutputPin,
        W: InputPin,
    > Drop for EPD<SPI, T, U, V, W>
{
    fn drop(&mut self) {
        self.halt();
        self.reset_pin.set_low();
        self.data_command_pin.set_low();
    }
}
