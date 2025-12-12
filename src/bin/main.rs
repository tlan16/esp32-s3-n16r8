#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use bt_hci::controller::ExternalController;
use embassy_executor::Spawner;
use embassy_time::{Duration, Instant, Timer};
use esp_backtrace as _;
use esp_hal::clock::CpuClock;
use esp_hal::gpio::{Level, Output, OutputConfig};
use esp_hal::rng::Rng;
use esp_hal::timer::timg::TimerGroup;
use esp_radio::ble::controller::BleConnector;
use log::info;
use esp_radio::wifi::{ClientConfig, ModeConfig, WifiController, ScanConfig, ScanTypeConfig, WifiStaState, WifiEvent, WifiDevice};
use smoltcp::iface::{SocketSet, SocketStorage};
use smoltcp::wire::{DhcpOption};
use trouble_host::prelude::*;
use embassy_net::{DhcpConfig, StackResources,Runner, Stack};

extern crate alloc;

const CONNECTIONS_MAX: usize = 1;
const L2CAP_CHANNELS_MAX: usize = 1;

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

macro_rules! mk_static {
    ($t:ty, $val:expr) => {{
        static STATIC_CELL: static_cell::StaticCell<$t> = static_cell::StaticCell::new();
        #[deny(unused_attributes)]
        let x = STATIC_CELL.uninit().write(($val));
        x
    }};
}

#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
    // generator version: 1.0.1

    esp_println::logger::init_logger_from_env();

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: 73744);
    // COEX needs more RAM - so we've added some more
    esp_alloc::heap_allocator!(size: 64 * 1024);

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let rng = Rng::new();
    esp_rtos::start(timg0.timer0);

    info!("Embassy initialized!");

    let radio_init = &*mk_static!(
        esp_radio::Controller<'static>,
        esp_radio::init().expect("Failed to initialize Wi-Fi/BLE controller")
    );
    let (wifi_controller, interfaces) =
        esp_radio::wifi::new(&radio_init, peripherals.WIFI, Default::default())
            .expect("Failed to initialize Wi-Fi controller");
    let wifi_interface = interfaces.sta;

    let rng = Rng::new();
    let net_seed = rng.random() as u64 | ((rng.random() as u64) << 32);
    let tls_seed = rng.random() as u64 | ((rng.random() as u64) << 32);

    let mut dhcp_config = DhcpConfig::default();
    let config = embassy_net::Config::dhcpv4(dhcp_config);


    let (stack, runner) = embassy_net::new(
        wifi_interface,
        config,
        mk_static!(StackResources<3>, StackResources::<3>::new()),
        net_seed,
    );

    spawner.spawn(connection(wifi_controller)).ok();
    spawner.spawn(net_task(runner)).ok();

    wait_for_connection(stack).await;


    // configure_wifi(&mut wifi_controller).await;
    // scan_wifi(&mut wifi_controller);
    // connect_wifi(&mut wifi_controller);

    // find more examples https://github.com/embassy-rs/trouble/tree/main/examples/esp32
    let transport = BleConnector::new(&radio_init, peripherals.BT, Default::default()).unwrap();
    let ble_controller = ExternalController::<_, 20>::new(transport);
    let mut resources: HostResources<DefaultPacketPool, CONNECTIONS_MAX, L2CAP_CHANNELS_MAX> =
        HostResources::new();
    let _stack = trouble_host::new(ble_controller, &mut resources);

    let mut led = Output::new(peripherals.GPIO43, Level::High, OutputConfig::default());

    loop {
        info!("============START============");
        Timer::after(Duration::from_millis(50)).await;

        led.toggle();

        // let ip_info = stack.get_ip_info();
        // if ip_info.is_ok() {
        //     info!("IP Address: {}", ip_info.unwrap().ip);
        // } else{
        //     info!("No IP Address assigned");
        // }
        info!("============END============");
        Timer::after(Duration::from_secs(1)).await;
    }

    // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/esp-hal-v1.0.0/examples/src/bin
}

async fn wait_for_connection(stack: Stack<'_>) {
    info!("Waiting for link to be up");
    loop {
        if stack.is_link_up() {
            break;
        }
        Timer::after(Duration::from_millis(500)).await;
    }

    info!("Waiting to get IP address...");
    loop {
        if let Some(config) = stack.config_v4() {
            info!("Got IP: {}", config.address);
            break;
        }
        Timer::after(Duration::from_millis(500)).await;
    }
}

#[derive(Debug)]
struct AppConfig {
    ssid: &'static str,
    password: &'static str,
    is_hidden: bool,
}

fn get_app_config() -> AppConfig {
    const APP_WIFI_SSID: &str = env!("APP_WIFI_SSID");
    const APP_WIFI_PASSWORD: &str = env!("APP_WIFI_PASSWORD");
    const APP_WIFI_IS_HIDDEN: &str = env!("APP_WIFI_IS_HIDDEN");

    AppConfig {
        ssid: APP_WIFI_SSID,
        password: APP_WIFI_PASSWORD,
        is_hidden: APP_WIFI_IS_HIDDEN == "true",
    }
}

pub fn create_interface(device: &mut esp_radio::wifi::WifiDevice) -> smoltcp::iface::Interface {
    // users could create multiple instances but since they only have one WifiDevice
    // they probably can't do anything bad with that
    smoltcp::iface::Interface::new(
        smoltcp::iface::Config::new(smoltcp::wire::HardwareAddress::Ethernet(
            smoltcp::wire::EthernetAddress::from_bytes(&device.mac_address()),
        )),
        device,
        timestamp(),
    )
}

// some smoltcp boilerplate
fn timestamp() -> smoltcp::time::Instant {
    smoltcp::time::Instant::from_micros(
        esp_hal::time::Instant::now()
            .duration_since_epoch()
            .as_micros() as i64,
    )
}

async fn configure_wifi(controller: &mut WifiController<'_>) {
    let app_config = get_app_config();
    controller
        .set_power_saving(esp_radio::wifi::PowerSaveMode::None)
        .unwrap();

    let client_config = ModeConfig::Client(
        ClientConfig::default()
            .with_ssid(app_config.ssid.into())
            .with_password(app_config.password.into()),
    );
    let res = controller.set_config(&client_config);
    info!("wifi_set_configuration returned {:?}", res);

    controller.start().unwrap();
    info!("is wifi started: {:?}", controller.is_started());

    // Add delay for hidden network stability
    Timer::after(Duration::from_secs(1)).await;
}

fn scan_wifi(controller: &mut WifiController<'_>) {
    use core::time::Duration;
    let app_config = get_app_config();

    info!("Start Wifi Scan");
    let scan_type = ScanTypeConfig::Active {
        min: Duration::from_millis(100),
        max: Duration::from_millis(300),
    };
    let scan_config = ScanConfig::default()
        .with_show_hidden(app_config.is_hidden)
        .with_max_none().with_scan_type(scan_type);
    let res = controller.scan_with_config(scan_config).unwrap();
    for ap in res {
        info!("{:?}", ap);
    }
}

fn connect_wifi(controller: &mut WifiController<'_>) {
    info!("{:?}", controller.capabilities());
    info!("wifi_connect {:?}", controller.connect());

    info!("Wait to get connected");
    loop {
        match controller.is_connected() {
            Ok(true) => break,
            Ok(false) => {}
            Err(err) => panic!("{:?}", err),
        }
    }
    info!("Connected: {:?}", controller.is_connected());
}


#[embassy_executor::task]
async fn connection(mut controller: WifiController<'static>) {
    let app_config = get_app_config();

    info!("start connection task");
    info!("Device capabilities: {:?}", controller.capabilities());
    loop {
        match esp_radio::wifi::sta_state() {
            WifiStaState::Connected => {
                // wait until we're no longer connected
                controller.wait_for_event(WifiEvent::StaDisconnected).await;
                Timer::after(Duration::from_millis(5000)).await
            }
            _ => {}
        }
        if !matches!(controller.is_started(), Ok(true)) {
            let client_config = ModeConfig::Client(
                ClientConfig::default()
                    .with_ssid(app_config.ssid.into())
                    .with_password(app_config.password.into()),
            );
            controller.set_config(&client_config).unwrap();
            info!("Starting wifi");
            controller.start_async().await.unwrap();
            info!("Wifi started!");

            info!("Scan");
            let scan_config = ScanConfig::default().with_show_hidden(true).with_max_none();
            let result = controller
                .scan_with_config_async(scan_config)
                .await
                .unwrap();
            for ap in result {
                info!("{:?}", ap);
            }
        }
        info!("About to connect...");

        match controller.connect_async().await {
            Ok(_) => info!("Wifi connected!"),
            Err(e) => {
                info!("Failed to connect to wifi: {:?}", e);
                Timer::after(Duration::from_millis(5000)).await
            }
        }
    }
}

#[embassy_executor::task]
async fn net_task(mut runner: Runner<'static, WifiDevice<'static>>) {
    runner.run().await
}
