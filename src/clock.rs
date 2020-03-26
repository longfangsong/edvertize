use stm32f1xx_hal::flash::FlashExt;
use stm32f1xx_hal::gpio::{GpioExt, Output, PushPull};
use stm32f1xx_hal::pac;
use stm32f1xx_hal::rcc::{Clocks, RccExt};
use stm32f1xx_hal::time::U32Ext;

pub fn init() -> Clocks {
    let dp = unsafe { pac::Peripherals::steal() };
    dp.FLASH.acr.write(|w| w.prftbe().set_bit());
    let rcc = dp.RCC.constrain();
    let mut flash = dp.FLASH.constrain();
    rcc.cfgr
        .use_hse(8.mhz())
        .sysclk(72.mhz())
        .pclk1(36.mhz())
        .freeze(&mut flash.acr)
}
