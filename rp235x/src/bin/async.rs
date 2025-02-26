#![no_main]
#![no_std]

{% if wifi %}use cyw43::{JoinOptions, NetDriver, PowerManagementMode};
use cyw43_pio::{DEFAULT_CLOCK_DIVIDER, PioSpi};
use defmt::{info, unwrap};
{% else %}use defmt::info;
{% endif %}use defmt_rtt as _;
use embassy_executor::Spawner;
{% if wifi %}use embassy_net::{
    Config, StackResources
};
{% endif %}use embassy_rp::{
    binary_info::{
        EntryAddr,
        consts::{ID_RP_PROGRAM_NAME, TAG_RASPBERRY_PI},
        rp_cargo_version, rp_program_build_attribute, rp_program_description,
    },
    {% if wifi %}bind_interrupts,
    {% endif %}gpio::{Level, Output},
{% if wifi %}    peripherals::{DMA_CH0, PIO0, TRNG},
    pio::{InterruptHandler as PioIH, Pio},
    trng::{InterruptHandler as TrngIH, Trng},
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

{% if wifi %}bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => PioIH<PIO0>;
    TRNG_IRQ => TrngIH<TRNG>;
});

#[embassy_executor::task]
async fn cyw43_task(
    runner: cyw43::Runner<'static, Output<'static>, PioSpi<'static, PIO0, 0, DMA_CH0>>,
) -> ! {
    runner.run().await
}

#[embassy_executor::task]
async fn net_task(mut runner: embassy_net::Runner<'static, NetDriver<'static>>) -> ! {
    runner.run().await
}

const SSID: &str = "";
const PASS: &[u8] = b"";
{% endif %}#[embassy_executor::main]
async fn main({% if wifi %}spawner{% else %}_s{% endif %}: Spawner) {
    let peripherals = embassy_rp::init(Default::default());
    {% if wifi %} 
    let mut pio = Pio::new(peripherals.PIO0, Irqs);
    let (device, mut control) = {
        let (device, control, runner) = cyw43::new(
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
            // `probe-rs download 43439A0.bin --binary-format bin --chip RP235x --base-address 0x10100000`
            // unsafe { core::slice::from_raw_parts(0x10100000 as *const u8, 230321) },
        )
        .await;
        unwrap!(spawner.spawn(cyw43_task(runner)));
        (device, control)
    };
    control.init(
        include_bytes!("{% endif %}{% if wifi and lib == "both" %}../{% endif %}{% if wifi %}../43439A0_clm.bin"),
        // `probe-rs download 43439A0_clm.bin --binary-format bin --chip RP235x --base-address 0x10140000`
        // unsafe { core::slice::from_raw_parts(0x10140000 as *const u8, 4752) },
    ).await;
    control.set_power_management(PowerManagementMode::PowerSave).await;
    control.gpio_set(0, true).await;

    unwrap!(
        control.join(SSID, JoinOptions::new(PASS)).await,
        "failed to connect to '{}'",
        SSID
    );
    
    let mut rng = Trng::new(peripherals.TRNG, Irqs, Default::default());
    let mut random = async || {
        let mut buffer = [0; 8];
        rng.fill_bytes(&mut buffer).await;
        u64::from_le_bytes(buffer)
    };

    let stack = {
        static RESOURCES: StaticCell<StackResources<3>> = StaticCell::new();
        let (stack, runner) = embassy_net::new(
            device,
            Config::dhcpv4(Default::default()),
            RESOURCES.init(StackResources::new()),
            random().await,
        );
        unwrap!(spawner.spawn(net_task(runner)));
        stack
    };

    stack.wait_link_up().await;
    stack.wait_config_up().await;
    {% else %}let mut led = Output::new(peripherals.PIN_2, Level::High);
    {% endif %}
    loop {
        info!("led off!");
        {% if wifi %}control.gpio_set(0, false).await{% else %}led.set_low(){% endif %};
        Timer::after_millis(500).await;

        info!("led on!");
        {% if wifi %}control.gpio_set(0, true).await{% else %}led.set_high(){% endif %};
        Timer::after_millis(500).await;
    }
}
