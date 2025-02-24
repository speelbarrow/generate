#![no_main]
#![no_std]

use defmt::info;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_rp::{
    binary_info::{
        EntryAddr,
        consts::{ID_RP_PROGRAM_NAME, TAG_RASPBERRY_PI},
        rp_cargo_version, rp_program_build_attribute, rp_program_description,
    },
    gpio::{Level, Output},
};
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
    // Importing `env` and trying to call it causes it to break, so full path is used instead.
    embassy_rp::binary_info::env!(TAG_RASPBERRY_PI, ID_RP_PROGRAM_NAME, "CARGO_BIN_NAME"),
    rp_program_description!(c""),
    rp_cargo_version!(),
    rp_program_build_attribute!(),
];

#[embassy_executor::main]
async fn main(_s: Spawner) {
    let peripherals = embassy_rp::init(Default::default());
    let mut led = Output::new(peripherals.PIN_2, Level::Low);

    loop {
        info!("led on!");
        led.set_high();
        Timer::after_millis(500).await;

        info!("led off!");
        led.set_low();
        Timer::after_millis(500).await;
    }
}
