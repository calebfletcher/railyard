#![no_main]
#![no_std]

use defmt_rtt as _;
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

        let p = embassy_stm32::init(Default::default());

        // Setup clocks and systick timer for delays
        let clocks = embassy_stm32::rcc::clocks(&p.RCC);
        let sysclk = clocks.sys.to_hertz().unwrap();
        info!("running with sysclk: {}", sysclk);
        Mono::start(cx.core.SYST, sysclk.0);

        // Start health blinking led
        let health_led = embassy_stm32::gpio::Output::new(p.PB5, Level::High, Speed::Low);
        blink_led::spawn(health_led).ok();

        (Shared {}, Local {})
    }

    /// Blink one of the LEDs to show the system is alive
    #[task(priority = 1)]
    async fn blink_led(_: blink_led::Context<'_>, mut led: embassy_stm32::gpio::Output<'static>) {
        info!("startttt");
        loop {
            led.set_high();
            info!("high 1");
            Mono::delay(500.millis()).await;
            info!("high 2");
            led.set_low();
            Mono::delay(500.millis()).await;
        }
    }

    #[idle]
    fn idle(_: idle::Context<'_>) -> ! {
        loop {}
    }
}
