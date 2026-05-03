#![no_main]
#![no_std]

use defmt::info;
use defmt_rtt as _;
use embedded_hal::{delay::DelayNs, digital::OutputPin};
{% if arch == "risc" %}use panic_halt as _;
{% elsif arch == "arm" %}use panic_probe as _;
{% else %}#[cfg(target_arch = "riscv32")]
use panic_halt as _;
#[cfg(target_arch = "arm")]
use panic_probe as _;
{% endif %}use rp235x_hal::{
    block::ImageDef, clocks::init_clocks_and_plls, gpio::Pins, pac::Peripherals, Sio, Timer,
    Watchdog,
};

#[unsafe(link_section = ".start_block")]
#[used]
pub static IMAGE_DEF: ImageDef = ImageDef::secure_exe();

const XTAL_FREQ_HZ: u32 = 12_000_000u32;

#[rp235x_hal::entry]
fn main() -> ! {
    let mut peripherals = Peripherals::take().unwrap();

    let mut watchdog = Watchdog::new(peripherals.WATCHDOG);
    let clocks = init_clocks_and_plls(
        XTAL_FREQ_HZ,
        peripherals.XOSC,
        peripherals.CLOCKS,
        peripherals.PLL_SYS,
        peripherals.PLL_USB,
        &mut peripherals.RESETS,
        &mut watchdog,
    )
    .unwrap();
    let mut timer = Timer::new_timer0(peripherals.TIMER0, &mut peripherals.RESETS, &clocks);

    let sio = Sio::new(peripherals.SIO);
    let pins = Pins::new(
        peripherals.IO_BANK0,
        peripherals.PADS_BANK0,
        sio.gpio_bank0,
        &mut peripherals.RESETS,
    );

    let mut led = pins.gpio0.into_push_pull_output();
    loop {
        info!("led off!");
        _ = led.set_low();
        timer.delay_ms(500);
        
        info!("led on!");
        _ = led.set_high();
        timer.delay_ms(500);
    }
}
