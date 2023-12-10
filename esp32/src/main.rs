#![no_std]
#![no_main]
use esp_backtrace as _;

use embedded_hal::blocking::delay::DelayMs;
use esp_println::println;
{%- if ssd1306 %}
use fugit::RateExtU32;
{%- endif %}
use hal::{
    clock::ClockControl,
    entry,
    {%- if ssd1306 %}
    gpio::Pins,
    i2c::I2C,
    {%- endif %}
    peripherals::Peripherals,
    system::{SystemExt, SystemParts},
    {% if ssd1306 -%}
    Delay, IO,
    {%- else -%}
    Delay,
    {%- endif %}
};
{% if ssd1306 -%}
use ssd1306::{
    mode::DisplayConfig, rotation::DisplayRotation, size::DisplaySize128x64, I2CDisplayInterface, 
    Ssd1306,
};
{%- endif %}

#[entry]
fn main() -> ! {
    {% if ssd1306 -%}
    let Peripherals {
        SYSTEM,
        GPIO,
        IO_MUX,
        I2C0,
        ..
    } = Peripherals::take();
    {%- else -%}
    let Peripherals { SYSTEM, .. } = Peripherals::take();
    {%- endif %}
    let SystemParts { clock_control, .. } = SYSTEM.split();
    {%- if ssd1306 %}
    let IO {
        pins: Pins { gpio4, gpio5, .. },
        ..
    } = IO::new(GPIO, IO_MUX);
    {%- endif %}

    let clocks = ClockControl::max(clock_control).freeze();
    let mut delay = Delay::new(&clocks);

    esp_println::logger::init_logger_from_env();
    log::info!("Logger is setup");

    {% if ssd1306 -%}
    let mut display = Ssd1306::new(
        I2CDisplayInterface::new(I2C::new(I2C0, gpio5, gpio4, 100u32.kHz(), &clocks)),
        DisplaySize128x64,
        DisplayRotation::Rotate0,
    );
    display.init().unwrap();
    display.clear().unwrap();

    {% endif -%}
    loop {
        println!("Loop...");
        delay.delay_ms(500u32);
    }
}
