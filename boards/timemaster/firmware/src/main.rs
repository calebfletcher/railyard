#![no_main]
#![no_std]

use defmt_rtt as _;
use embassy_stm32::rcc::Pll;
use panic_probe as _;

use defmt::info;
use embassy_stm32::gpio::{Level, Speed};
use rtic_monotonics::Monotonic as _;
use rtic_monotonics::fugit::ExtU32;

rtic_monotonics::systick_monotonic!(Mono, 10000);

#[rtic::app(device = embassy_stm32::pac, peripherals = false, dispatchers = [SPI1, SPI2])]
mod app {

    use super::*;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {}

    #[init]
    fn init(cx: init::Context) -> (Shared, Local) {
        info!("starting timemaster");

        let p = init_core();

        // Setup clocks and systick timer for delays
        let clocks = embassy_stm32::rcc::clocks(&p.RCC);
        let sysclk = clocks.sys.to_hertz().unwrap();
        info!("running with sysclk: {}", sysclk);
        Mono::start(cx.core.SYST, sysclk.0);

        // Start health blinking led
        let health_led = embassy_stm32::gpio::Output::new(p.PB5, Level::Low, Speed::Low);
        blink_led::spawn(health_led).ok();

        (Shared {}, Local {})
    }

    /// Blink one of the LEDs to show the system is alive
    #[task]
    async fn blink_led(_: blink_led::Context<'_>, mut led: embassy_stm32::gpio::Output<'static>) {
        loop {
            led.set_high();
            Mono::delay(100.millis()).await;
            led.set_low();
            Mono::delay(900.millis()).await;
        }
    }
}

/// Initialise the HAL with the required RCC clock configuration.
fn init_core() -> embassy_stm32::Peripherals {
    let mut config = embassy_stm32::Config::default();
    // PLL main output of (16MHz / 2) * (16 / 2) = 64MHz
    config.rcc.pll = Some(Pll {
        source: embassy_stm32::rcc::PllSource::HSI,
        prediv: embassy_stm32::rcc::PllPreDiv::DIV2,
        mul: embassy_stm32::rcc::PllMul::MUL16,
        divp: None,
        divq: None,
        divr: Some(embassy_stm32::rcc::PllRDiv::DIV2),
    });
    // SYSCLK from PLL R output
    config.rcc.sys = embassy_stm32::rcc::Sysclk::PLL1_R;
    config.rcc.ahb_pre = embassy_stm32::rcc::AHBPrescaler::DIV1;

    embassy_stm32::init(config)
}
