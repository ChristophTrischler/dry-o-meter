#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]

use core::time::Duration;

use controller_esp32c6::config::Config;
use controller_esp32c6::sensor::Sensor;
use controller_esp32c6::{backend, connect_wifi, run_config_portal};
use embassy_executor::Spawner;
use embassy_futures::select::{Either, select};
use embassy_net::StackResources;
use embassy_time::{Duration as EmbassyDuration, Timer, with_timeout};
use esp_hal::clock::CpuClock;
use esp_hal::peripherals::{ADC1, GPIO2, LPWR, WIFI};
use esp_hal::rtc_cntl::Rtc;
use esp_hal::rtc_cntl::sleep::TimerWakeupSource;
use esp_hal::timer::timg::TimerGroup;
use log::{debug, error, info, warn};

const REPORT_TIMEOUT: EmbassyDuration = EmbassyDuration::from_secs(60);
const SLEEP_DURATION: Duration = Duration::from_secs(5 * 60);
const NETWORK_SEED: u64 = 0x0fed_cba9_8765_4321;

struct Platform {
    wifi: WIFI<'static>,
    adc: ADC1<'static>,
    pin: GPIO2<'static>,
    lpwr: LPWR<'static>,
}

#[esp_hal::ram(unstable(rtc_fast, persistent))]
static mut CONFIG: Config = const { Config::new() };

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

extern crate alloc;

esp_bootloader_esp_idf::esp_app_desc!();

#[allow(
    clippy::large_stack_frames,
    reason = "it's not unusual to allocate larger buffers etc. in main"
)]
#[esp_rtos::main]
async fn main(_spawner: Spawner) -> ! {
    controller_esp32c6::init_logging();
    info!("Dry-O-Meter controller booting");

    let platform = init();

    let config = unsafe { &mut *core::ptr::addr_of_mut!(CONFIG) };
    let mut rtc = Rtc::new(platform.lpwr);
    let mut sensor = Sensor::new(platform.adc, platform.pin);

    if config.set {
        info!(
            "Loaded configuration: ssid={:?}, server={:?}, uuid={:?}",
            config.ssid_str(),
            config.server_str(),
            config.uuid_str()
        );
    } else {
        info!("No configuration stored; starting setup portal");
    }

    connect_and_report(platform.wifi, config, &mut sensor).await;
    sleep(&mut rtc)
}

fn init() -> Platform {
    debug!("Initializing CPU, heap and RTOS timer");

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: 65536);

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let sw_interrupt =
        esp_hal::interrupt::software::SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);
    esp_rtos::start(timg0.timer0, sw_interrupt.software_interrupt0);

    Platform {
        wifi: peripherals.WIFI,
        adc: peripherals.ADC1,
        pin: peripherals.GPIO2,
        lpwr: peripherals.LPWR,
    }
}

#[allow(
    clippy::large_stack_frames,
    reason = "the station stack resources live for the duration of the network task"
)]
async fn connect_and_report(wifi: WIFI<'_>, config: &mut Config, sensor: &mut Sensor<'_>) {
    info!(
        "SSID: {}, PW: {}, SET: {}",
        config.ssid_str(),
        config.pw(),
        config.set
    );
    let connection = if config.set || {
        *config = Config::new();
        config.set
    } {
        debug!("Connecting directly to configured Wi-Fi");
        connect_wifi(wifi, config).await
    } else {
        debug!("Starting configuration portal");
        run_config_portal(wifi, config).await
    };

    let (mut controller, interfaces) = match connection {
        Ok(connection) => {
            info!("Wi-Fi connected");
            connection
        }
        Err(error) => {
            error!("Failed to connect Wi-Fi: {error:?}");
            panic!("Failed to connect Wi-Fi");
        }
    };

    let mut resources = StackResources::<4>::new();
    let (stack, mut runner) = embassy_net::new(
        interfaces.station,
        embassy_net::Config::dhcpv4(Default::default()),
        &mut resources,
        NETWORK_SEED,
    );

    debug!("Starting backend report cycle");
    let report = with_timeout(
        REPORT_TIMEOUT,
        backend::run(stack, config, &mut controller, sensor),
    );

    match select(runner.run(), report).await {
        Either::Second(Ok(())) => info!("Report cycle finished"),
        Either::Second(Err(_)) => {
            warn!("Report cycle timed out after {REPORT_TIMEOUT:?}");
        }
        Either::First(never) => match never {},
    }
}

fn sleep(rtc: &mut Rtc<'_>) -> ! {
    info!("Entering deep sleep for {SLEEP_DURATION:?}");

    let wakeup = TimerWakeupSource::new(SLEEP_DURATION);

    rtc.sleep_deep(&[&wakeup])
}
