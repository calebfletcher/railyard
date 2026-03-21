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

embassy_stm32::bind_interrupts!(struct Irqs {
    I2C2 => embassy_stm32::i2c::EventInterruptHandler<embassy_stm32::peripherals::I2C2>, embassy_stm32::i2c::ErrorInterruptHandler<embassy_stm32::peripherals::I2C2>;
    DMA1_CHANNEL1 => embassy_stm32::dma::InterruptHandler<embassy_stm32::peripherals::DMA1_CH1>;
    DMA1_CHANNEL2_3  => embassy_stm32::dma::InterruptHandler<embassy_stm32::peripherals::DMA1_CH2>;
});

#[rtic::app(device = embassy_stm32::pac, peripherals = false, dispatchers = [SPI1, SPI2])]
mod app {

    use si5340::Si5340;

    use super::*;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        clk_i2c: embassy_stm32::i2c::I2c<
            'static,
            embassy_stm32::mode::Async,
            embassy_stm32::i2c::mode::Master,
        >,
    }

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

        let clk_i2c = embassy_stm32::i2c::I2c::new(
            p.I2C2,
            p.PA11,
            p.PA12,
            p.DMA1_CH1,
            p.DMA1_CH2,
            Irqs,
            embassy_stm32::i2c::Config::default(),
        );

        check_i2c::spawn().ok();

        (Shared {}, Local { clk_i2c })
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

    #[task(local = [clk_i2c])]
    async fn check_i2c(cx: check_i2c::Context<'_>) {
        let i2c = cx.local.clk_i2c;
        let mut _device = Si5340::new_i2c(i2c, si5340::Address::from_pins(true, false));
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
