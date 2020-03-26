#![no_std]
#![no_main]
#![feature(alloc_error_handler)]

extern crate alloc;
extern crate panic_semihosting;

mod clock;
mod image;
mod memory;
mod spi_device;

use crate::image::Image;
use crate::spi_device::epd::EPD;
use crate::spi_device::sd::SD;
use alloc::rc::Rc;
use core::cell::RefCell;
use cortex_m::asm;
use cortex_m_rt::entry;
use cortex_m_semihosting::hprintln;
use embedded_hal::blocking::delay::DelayMs;
use embedded_hal::digital::v2::InputPin;
use stm32f1xx_hal::afio::AfioExt;
use stm32f1xx_hal::delay::Delay;
use stm32f1xx_hal::gpio::{gpioa, gpiob, GpioExt, Output, PushPull};
use stm32f1xx_hal::pac;
use stm32f1xx_hal::rcc::RccExt;
use stm32f1xx_hal::spi::{Mode, Phase, Polarity, Spi};
use stm32f1xx_hal::time::U32Ext;

#[entry]
fn main() -> ! {
    let clocks = clock::init();
    memory::init();
    let cp = unsafe { cortex_m::Peripherals::steal() };
    let dp = unsafe { pac::Peripherals::steal() };
    let mut delay = Delay::new(cp.SYST, clocks);
    let mut rcc = dp.RCC.constrain();

    let mut gpioa = dp.GPIOA.split(&mut rcc.apb2);
    let mut gpioc = dp.GPIOC.split(&mut rcc.apb2);
    let mut afio = dp.AFIO.constrain(&mut rcc.apb2);
    let mut key1 = gpioc.pc5.into_pull_up_input(&mut gpioc.crl);
    let pins = (
        gpioa.pa5.into_alternate_push_pull(&mut gpioa.crl),
        gpioa.pa6.into_floating_input(&mut gpioa.crl),
        gpioa.pa7.into_alternate_push_pull(&mut gpioa.crl),
    );
    let spi_mode = Mode {
        polarity: Polarity::IdleLow,
        phase: Phase::CaptureOnFirstTransition,
    };
    let mut spi = Spi::spi1(
        dp.SPI1,
        pins,
        &mut afio.mapr,
        spi_mode,
        32.mhz(),
        clocks,
        &mut rcc.apb2,
    );
    let mut chip_select_pin: gpioa::PA3<Output<PushPull>> =
        gpioa.pa3.into_push_pull_output(&mut gpioa.crl);
    delay.delay_ms(1u32);
    let sd = Rc::new(RefCell::new(SD::new(spi, chip_select_pin)));
    hprintln!("sd inited");
    let mut gpiob = dp.GPIOB.split(&mut rcc.apb2);
    let pins = (
        gpiob.pb13.into_alternate_push_pull(&mut gpiob.crh),
        gpiob.pb14.into_floating_input(&mut gpiob.crh),
        gpiob.pb15.into_alternate_push_pull(&mut gpiob.crh),
    );
    let mut chip_select_pin: gpiob::PB12<Output<PushPull>> =
        gpiob.pb12.into_push_pull_output(&mut gpiob.crh);
    let spi_mode = Mode {
        polarity: Polarity::IdleLow,
        phase: Phase::CaptureOnFirstTransition,
    };
    let mut spi2 = Spi::spi2(dp.SPI2, pins, spi_mode, 4.mhz(), clocks, &mut rcc.apb1);
    let reset_pin: gpiob::PB11<Output<PushPull>> = gpiob.pb11.into_push_pull_output(&mut gpiob.crh);
    let data_command_pin: gpiob::PB10<Output<PushPull>> =
        gpiob.pb10.into_push_pull_output(&mut gpiob.crh);
    let busy_pin = gpiob.pb9.into_floating_input(&mut gpiob.crh);
    let mut epd = EPD::new(spi2, reset_pin, chip_select_pin, data_command_pin, busy_pin);
    hprintln!("epd inited");
    let mut current = true;
    loop {
        if current {
            let image = Image::new(0, sd.clone());
            epd.display(image.black_iter());
            hprintln!("displayed");
        } else {
            let image = Image::new(48_128, sd.clone());
            epd.display(image.black_iter());
            hprintln!("displayed2");
        }
        current = !current;
        for _ in 0..60000 {
            if key1.is_low().unwrap() {
                break;
            }
            delay.delay_ms(1u32);
        }
    }
}
