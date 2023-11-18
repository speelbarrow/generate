#![no_std]
#![no_main]
use esp_backtrace as _;

use embedded_hal::blocking::delay::DelayMs;
use esp_println::println;
use hal::{clock::ClockControl, entry, peripherals::Peripherals, system::SystemExt, Delay};

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take();
    let system = peripherals.SYSTEM.split();

    let clocks = ClockControl::max(system.clock_control).freeze();
    let mut delay = Delay::new(&clocks);

    esp_println::logger::init_logger_from_env();
    log::info!("Logger is setup");

    loop {
        println!("Loop...");
        delay.delay_ms(500u32);
    }
}
