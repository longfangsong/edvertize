use crate::spi_device::sd::SD;
use alloc::rc::Rc;
use alloc::vec::Vec;
use core::cell::RefCell;
use core::cmp::min;
use cortex_m_semihosting::hprintln;

pub struct Image<
    SPI: embedded_hal::blocking::spi::transfer::Default<u8>
        + embedded_hal::blocking::spi::write::Default<u8>,
    T: embedded_hal::digital::v2::OutputPin,
> {
    address: u64,
    sd: Rc<RefCell<SD<SPI, T>>>,
}

enum Color {
    Black,
    Red,
}

pub struct ImageIterator<
    'a,
    SPI: embedded_hal::blocking::spi::transfer::Default<u8>
        + embedded_hal::blocking::spi::write::Default<u8>,
    T: embedded_hal::digital::v2::OutputPin,
> {
    from_image: &'a Image<SPI, T>,
    current: u64,
    color: Color,
}

impl<
        'a,
        SPI: embedded_hal::blocking::spi::transfer::Default<u8>
            + embedded_hal::blocking::spi::write::Default<u8>,
        T: embedded_hal::digital::v2::OutputPin,
    > Iterator for ImageIterator<'a, SPI, T>
{
    type Item = Vec<u8>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut current_color_end = match self.color {
            Color::Black => self.from_image.address + 48_000,
            Color::Red => self.from_image.address + 96_000,
        };
        if self.current >= current_color_end {
            self.from_image.sd.borrow_mut().read_end();
            None
        } else {
            let mut result = Vec::new();
            let read_size = min(512, current_color_end - self.current);
            result.resize(read_size as _, 0xff);
            self.current += read_size;
            self.from_image.sd.borrow_mut().read_next(&mut result);
            Some(result)
        }
    }
}

impl<
        SPI: embedded_hal::blocking::spi::transfer::Default<u8>
            + embedded_hal::blocking::spi::write::Default<u8>,
        T: embedded_hal::digital::v2::OutputPin,
    > Image<SPI, T>
{
    pub fn new(address: u64, sd: Rc<RefCell<SD<SPI, T>>>) -> Self {
        Image {
            address,
            sd: sd.clone(),
        }
    }
    pub fn black_iter(&self) -> ImageIterator<'_, SPI, T> {
        self.sd.borrow_mut().read_start(self.address);
        ImageIterator {
            from_image: &self,
            current: 0,
            color: Color::Black,
        }
    }
}
