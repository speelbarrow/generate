#![no_main]
#![no_std]

{% if wifi %}use cyw43_pio::{DEFAULT_CLOCK_DIVIDER, PioSpi};
use defmt::{info, unwrap};
{% else %}use defmt::info;
{% endif %}use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_rp::{
    binary_info::{
        EntryAddr,
        consts::{ID_RP_PROGRAM_NAME, TAG_RASPBERRY_PI},
        rp_cargo_version, rp_program_build_attribute, rp_program_description,
    },
    {% if wifi %}bind_interrupts,
    {% endif %}gpio::{Level, Output},
{% if wifi %}    peripherals::{DMA_CH0, PIO0},
    pio::{InterruptHandler, Pio},
{% endif %}};
use embassy_time::Timer;
{% if arch == "risc" %}use panic_halt as _;
{% elsif arch == "arm" %}use panic_probe as _;
{% else %}#[cfg(target_arch = "riscv32")]
use panic_halt as _;
#[cfg(target_arch = "arm")]
use panic_probe as _;
{% endif %}{% if wifi %}use static_cell::StaticCell;
{% endif %}
#[unsafe(link_section = ".bi_entries")]
#[used]
pub static PICOTOOL_ENTRIES: [EntryAddr; 4] = [
    // Importing `env` and trying to call it causes it to break, so full path is used instead.
    embassy_rp::binary_info::env!(TAG_RASPBERRY_PI, ID_RP_PROGRAM_NAME, "CARGO_BIN_NAME"),
    rp_program_description!(c""),
    rp_cargo_version!(),
    rp_program_build_attribute!(),
];
{% if wifi %}
bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => InterruptHandler<PIO0>;
});

#[embassy_executor::task]
async fn cyw43_task(
    runner: cyw43::Runner<'static, Output<'static>, PioSpi<'static, PIO0, 0, DMA_CH0>>,
) -> ! {
    runner.run().await
}
{% endif %}
#[embassy_executor::main]
async fn main({% if wifi %}spawner{% else %}_s{% endif %}: Spawner) {
    let peripherals = embassy_rp::init(Default::default());
    {% if wifi %} 
    let mut pio = Pio::new(peripherals.PIO0, Irqs);
    let (_, mut control, runner) = cyw43::new(
        {
            use cyw43::State;
            static STATE: StaticCell<State> = StaticCell::new();
            STATE.init(State::new())
        },
        Output::new(peripherals.PIN_23, Level::Low),
        PioSpi::new(
            &mut pio.common,
            pio.sm0,
            DEFAULT_CLOCK_DIVIDER,
            pio.irq0,
            Output::new(peripherals.PIN_25, Level::High),
            peripherals.PIN_24,
            peripherals.PIN_29,
            peripherals.DMA_CH0,
        ),
        include_bytes!("{% endif %}{% if wifi and lib == "both" %}../{% endif %}{% if wifi %}../43439A0.bin"),
    )
    .await;
    unwrap!(spawner.spawn(cyw43_task(runner)));
    control.init(include_bytes!("{% endif %}{% if wifi and lib == "both" %}../{% endif %}{% if wifi %}../43439A0_clm.bin")).await;
    {% else %}let mut led = Output::new(peripherals.PIN_2, Level::Low);
    {% endif %}
    loop {
        info!("led on!");
        {% if wifi %}control.gpio_set(0, true).await{% else %}led.set_high(){% endif %};
        Timer::after_millis(500).await;

        info!("led off!");
        {% if wifi %}control.gpio_set(0, false).await{% else %}led.set_low(){% endif %};
        Timer::after_millis(500).await;
    }
}
