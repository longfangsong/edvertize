use crate::spi_device::SPIDevice;
use core::ops::Deref;
use cortex_m_semihosting::hprintln;
use embedded_hal::blocking::spi::{Transfer, Write};
use embedded_hal::digital::v2::OutputPin;

pub struct SD<
    SPI: embedded_hal::blocking::spi::Write<u8> + embedded_hal::blocking::spi::Transfer<u8>,
    T: OutputPin,
>(SPIDevice<SPI, T>);

impl<
        SPI: embedded_hal::blocking::spi::transfer::Default<u8>
            + embedded_hal::blocking::spi::write::Default<u8>,
        T: OutputPin,
    > SD<SPI, T>
{
    fn send_command(&mut self, command: u8, arg: u32, crc: u8) -> Option<u8> {
        self.0.select();
        self.0
            .spi
            .write(&[
                command | 0x40u8,
                (arg >> 24) as u8,
                (arg >> 16) as u8,
                (arg >> 8) as u8,
                arg as u8,
                crc,
            ])
            .ok()?;
        let mut result_buffer = [0xffu8];
        while result_buffer[0] == 0xff {
            self.0.spi.transfer(&mut result_buffer).ok()?;
        }
        self.0.deselect();
        self.0.spi.write(&[0xffu8]).ok()?;
        Some(result_buffer[0])
    }
    pub fn new(spi: SPI, chip_select_pin: T) -> Self {
        let mut result = SPIDevice {
            spi,
            chip_select_pin,
        };
        result.select();
        for _ in 0..10 {
            result.spi.write(&[0xff]).ok().unwrap();
        }
        result.deselect();
        let mut result = SD(result);
        // idle
        let mut response = result.send_command(0x00, 0, 0x95).unwrap();
        while response != 0x1 {
            response = result.send_command(0x00, 0, 0x95).unwrap();
        }
        // SD2.0?
        response = result.send_command(0x08, 0x1aa, 0x87).unwrap();
        if response == 1 {
            unimplemented!();
        } else {
            // init
            result.send_command(0x37, 0, 0x01).unwrap();
            response = result.send_command(0x29, 0, 0x01).unwrap();
            if response <= 1 {
                while response != 0 {
                    result.send_command(0x37, 0, 0x01).unwrap();
                    response = result.send_command(0x29, 0, 0x01).unwrap();
                }
            } else {
                unimplemented!();
            }
        }
        result
    }
    fn read_data(&mut self, buffer: &mut [u8]) {
        self.0.select();
        let mut ready_buffer = [0xffu8];
        while ready_buffer[0] != 0xfe {
            self.0.spi.transfer(&mut ready_buffer);
        }
        self.0.spi.transfer(buffer);
        let mut crc_buffer = [0xffu8, 0xffu8];
        self.0.spi.transfer(&mut crc_buffer);
        self.0.deselect();
    }
    pub fn read_sector(&mut self, sector: u64, result: &mut [u8]) {
        let response = self.send_command(0x11, (sector << 9) as u32, 0x01).unwrap();
        if response != 0 {
            panic!();
        }
        self.read_data(result);
        hprintln!("Read success");
    }
    pub fn read_start(&mut self, address: u64) {
        let response = self.send_command(0x12, address as u32, 0x01).unwrap();
        if response != 0 {
            panic!();
        }
    }
    pub fn read_next(&mut self, result: &mut [u8]) {
        self.read_data(result);
    }
    pub fn read_end(&mut self) {
        self.send_command(0x0c, 0, 0x01).unwrap();
    }
}
