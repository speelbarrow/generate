use crate::Irqs;
use cyw43::{JoinOptions, NetDriver, PowerManagementMode};
use cyw43_pio::{PioSpi, RM2_CLOCK_DIVIDER};
use defmt::error;
use embassy_executor::Spawner;
use embassy_futures::select::{Either, select};
use embassy_net::{Config, StackResources};
use embassy_rp::{
    Peri,
    gpio::{Level, Output},
    peripherals::{DMA_CH0, PIN_23, PIN_24, PIN_25, PIN_29, PIO0, TRNG},
    pio::Pio,
    trng::Trng,
};
use embassy_time::Timer;
use static_cell::StaticCell;

#[embassy_executor::task]
async fn cyw43_task(
    runner: cyw43::Runner<'static, cyw43::SpiBus<Output<'static>, PioSpi<'static, PIO0, 0>>>,
) -> ! {
    runner.run().await
}

#[embassy_executor::task]
async fn net_task(mut runner: embassy_net::Runner<'static, NetDriver<'static>>) -> ! {
    runner.run().await
}

const SSID: &str = include_str!("../SSID");
const PASS: &[u8] = include_bytes!("../PASS");
#[embassy_executor::task]
pub async fn wifi(
    dma: Peri<'static, DMA_CH0>,
    pin23: Peri<'static, PIN_23>,
    pin24: Peri<'static, PIN_24>,
    pin25: Peri<'static, PIN_25>,
    pin29: Peri<'static, PIN_29>,
    pio0: Peri<'static, PIO0>,
    trng: Peri<'static, TRNG>,
    spawner: Spawner,
) {
    let mut pio = Pio::new(pio0, Irqs);
    let (device, mut control) = {
        let (device, control, runner) = cyw43::new(
            {
                use cyw43::State;
                static STATE: StaticCell<State> = StaticCell::new();
                STATE.init(State::new())
            },
            Output::new(pin23, Level::Low),
            PioSpi::new(
                &mut pio.common,
                pio.sm0,
                RM2_CLOCK_DIVIDER,
                pio.irq0,
                Output::new(pin25, Level::High),
                pin24,
                pin29,
                embassy_rp::dma::Channel::new(dma, Irqs),
            ),
            /*
            probe-rs download 43439A0.bin --binary-format bin --chip RP235x --base-address 0x10100000

            unsafe {
                core::mem::transmute(core::slice::from_raw_parts(
                    0x10100000 as *const u8,
                    0x386a5,
                ))
            },
            */
            cyw43::aligned_bytes!("../43439A0.bin"),
            cyw43::aligned_bytes!("../nvram_rp2040.bin"),
        )
        .await;

        match cyw43_task(runner) {
            Ok(task) => spawner.spawn(task),
            Err(error) => {
                error!(
                    "failed to spawn cyw43 task: {}\nwifi stack is disabled",
                    error
                );
                return;
            }
        }
        (device, control)
    };
    control
        .init(
            /*
            probe-rs download 43439A0_clm.bin --binary-format bin --chip RP235x --base-address 0x10140000

            unsafe { core::slice::from_raw_parts(0x10140000 as *const u8, 0x3d8) },
            */
            cyw43::aligned_bytes!("../43439A0_clm.bin"),
        )
        .await;
    control
        .set_power_management(PowerManagementMode::PowerSave)
        .await;

    // LED OFF: no connection has been established
    control.gpio_set(0, false).await;

    loop {
        if let Err(error) = control.join(SSID, JoinOptions::new(PASS)).await {
            error!(
                "failed to initiate connection to '{}': {}\n[retrying in 5 seconds]",
                SSID, error
            );
            Timer::after_secs(5).await;
        } else {
            break;
        }
    }

    // LED blinking: WiFi connection established, pending WAN connection
    match select(
        async {
            let mut rng = Trng::new(trng, Irqs, {
                let mut r = embassy_rp::trng::Config::default();

                r.sample_count = 128;

                r
            });
            let mut random = async || {
                let mut buffer = [0; 8];
                rng.fill_bytes(&mut buffer).await;
                u64::from_le_bytes(buffer)
            };
            static RESOURCES: StaticCell<StackResources<3>> = StaticCell::new();
            let (stack, runner) = embassy_net::new(
                device,
                Config::dhcpv4(Default::default()),
                RESOURCES.init(StackResources::new()),
                random().await,
            );
            match net_task(runner) {
                Ok(task) => {
                    spawner.spawn(task);
                    stack.wait_link_up().await;
                    stack.wait_config_up().await;
                    true
                }
                Err(error) => {
                    error!(
                        "failed to spawn net task: {}\nwifi stack is disabled",
                        error
                    );
                    false
                }
            }
        },
        async {
            let mut state = true;
            loop {
                state = !state;
                control.gpio_set(0, state).await;
                Timer::after_millis(100).await;
            }
        },
    )
    .await
    {
        /*
        LED solid: connected to internet
        (if net task failed to spawn, LED is off instead)
        */
        Either::First(b) => control.gpio_set(0, b).await,
    }
}
