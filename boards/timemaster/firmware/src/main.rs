#![no_main]
#![no_std]

use defmt_rtt as _;
use panic_probe as _;

pub mod pac {
    pub use embassy_stm32::pac::Interrupt as interrupt;
    pub use embassy_stm32::pac::*;
}

#[rtic::app(device = pac, peripherals = false, dispatchers = [USART1])]
mod app {
    use super::*;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {}

    #[init]
    fn init(_: init::Context) -> (Shared, Local) {
        (Shared {}, Local {})
    }

    // #[task(binds = USART2)]
    // fn bar(c: bar::Context) {
    //     crate::bar_trampoline(c)
    // }

    extern "Rust" {
        #[task(binds = I2C1)]
        fn bar2(cx: bar2::Context);
    }
}

fn bar2(_: app::bar2::Context) {}
