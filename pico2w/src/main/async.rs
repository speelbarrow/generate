#![no_main]
#![no_std]

{% if wifi %}mod wifi;

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
    {% if wifi %}bind_interrupts, dma,
    {% endif %}gpio::{Level, Output},
{% if wifi %}    peripherals::{DMA_CH0, PIO0, TRNG},
    pio::InterruptHandler as PioIH,
    trng::InterruptHandler as TrngIH,
{% endif %}};
use embassy_time::Timer;
{% if arch == "risc" %}use panic_halt as _;
{% elsif arch == "arm" %}use panic_probe as _;
{% else %}#[cfg(target_arch = "riscv32")]
use panic_halt as _;
#[cfg(target_arch = "arm")]
use panic_probe as _;
{% endif %}
#[unsafe(link_section = ".bi_entries")]
#[used]
pub static PICOTOOL_ENTRIES: [EntryAddr; 4] = [
    embassy_rp::binary_info::env!(TAG_RASPBERRY_PI, ID_RP_PROGRAM_NAME, "CARGO_BIN_NAME"),
    rp_program_description!(c""),
    rp_cargo_version!(),
    rp_program_build_attribute!(),
];
{% if wifi %}
bind_interrupts!(struct Irqs {
    DMA_IRQ_0 => dma::InterruptHandler<DMA_CH0>;
    PIO0_IRQ_0 => PioIH<PIO0>;
    TRNG_IRQ => TrngIH<TRNG>;
});
{% endif %}
#[embassy_executor::main]
async fn main({% if wifi %}spawner{% else %}_s{% endif %}: Spawner) {
    let peripherals = embassy_rp::init(Default::default());
{% if wifi %}
    spawner.spawn(unwrap!(wifi::wifi(
        peripherals.DMA_CH0,
        peripherals.PIN_23,
        peripherals.PIN_24,
        peripherals.PIN_25,
        peripherals.PIN_29,
        peripherals.PIO0,
        peripherals.TRNG,
        spawner,
    )));
{% endif %}
    let mut led = Output::new(peripherals.PIN_0, Level::High);
    loop {
        info!("led off!");
        led.set_low();
        Timer::after_millis(500).await;

        info!("led on!");
        led.set_high();
        Timer::after_millis(500).await;
    }
}
