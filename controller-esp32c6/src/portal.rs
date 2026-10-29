use core::net::{IpAddr, Ipv4Addr, SocketAddr};

use edge_dhcp::io as dhcp_io;
use edge_dhcp::server::{Server as DhcpServer, ServerOptions};
use edge_nal::UdpBind;
use edge_nal_embassy::{Udp, UdpBuffers};
use embassy_futures::join::{join, join3};
use embassy_futures::select::{Either, select};
use embassy_net::{Ipv4Cidr, StackResources, StaticConfigV4};
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::signal::Signal;
use embassy_time::{Duration, with_timeout};
use esp_hal::peripherals::WIFI;
use esp_radio::wifi::ap::AccessPointConfig;
use esp_radio::wifi::sta::StationConfig;
use esp_radio::wifi::{
    AuthenticationMethod, Config as WifiConfig, ControllerConfig, Interfaces, WifiController,
    WifiError,
};
use heapless::Vec as HVec;
use log::{debug, info, warn};

use crate::config::Config as DeviceConfig;
use crate::http::serve_http;

const PORTAL_SSID: &str = "DryOMeter-Setup";
const PORTAL_ADDRESS: Ipv4Addr = Ipv4Addr::new(192, 168, 4, 1);
const PORTAL_URL: &str = "http://192.168.4.1/";

#[derive(Debug)]
pub enum ConfigPortalError {
    Wifi(WifiError),
    Network,
}

impl From<WifiError> for ConfigPortalError {
    fn from(value: WifiError) -> Self {
        Self::Wifi(value)
    }
}

/// Connects the station to the network stored in `config`.
pub async fn connect_wifi<'d>(
    wifi: WIFI<'d>,
    config: &DeviceConfig,
) -> Result<(WifiController<'d>, Interfaces<'d>), ConfigPortalError> {
    info!(
        "Connecting to Wi-Fi {:?} (password set: {})",
        config.ssid_str(),
        config.pw_len != 0
    );

    let initial_config = WifiConfig::Station(station_config(config));
    let controller_config = ControllerConfig::default().with_initial_config(initial_config);

    debug!("Creating Wi-Fi controller in station mode");
    let (mut controller, interfaces) = esp_radio::wifi::new(wifi, controller_config)?;

    debug!("Waiting for station association");
    controller.connect_async().await?;

    info!("Station associated with {:?}", config.ssid_str());

    Ok((controller, interfaces))
}

/// Starts an open Wi-Fi access point hosting the configuration portal.
///
/// If `config` already holds a saved network, a connection is attempted
/// immediately. Otherwise the portal must be used. Newly submitted settings
/// restart the connection attempt. The function returns once the station has
/// obtained an IP address.
///
/// The portal is reachable at `http://192.168.4.1/` while it is running and
/// asks for the station SSID, password and server address. Submitted values are
/// written into `config`.
#[allow(
    clippy::large_stack_frames,
    reason = "the portal owns its network buffers for the lifetime of the server"
)]
pub async fn run_config_portal<'d>(
    wifi: WIFI<'d>,
    config: &mut DeviceConfig,
) -> Result<(WifiController<'d>, Interfaces<'d>), ConfigPortalError> {
    info!("Starting setup portal {PORTAL_SSID:?} at {PORTAL_URL}");

    let ap_config = AccessPointConfig::default()
        .with_ssid(PORTAL_SSID)
        .with_auth_method(AuthenticationMethod::None)
        .with_max_connections(4);

    let initial_config =
        WifiConfig::AccessPointStation(StationConfig::default(), ap_config.clone());
    let controller_config = ControllerConfig::default().with_initial_config(initial_config);

    debug!("Creating Wi-Fi controller in access-point/station mode");
    let (mut controller, interfaces) = esp_radio::wifi::new(wifi, controller_config)?;

    debug!("Configuring access-point network {PORTAL_ADDRESS}/24 and station DHCP");
    let ap_net_config = embassy_net::Config::ipv4_static(StaticConfigV4 {
        address: Ipv4Cidr::new(PORTAL_ADDRESS, 24),
        gateway: None,
        dns_servers: HVec::new(),
    });
    let sta_net_config = embassy_net::Config::dhcpv4(Default::default());

    let mut ap_resources = StackResources::<8>::new();
    let (ap_stack, mut ap_runner) = embassy_net::new(
        interfaces.access_point,
        ap_net_config,
        &mut ap_resources,
        0x1234_5678_9abc_def0,
    );

    let mut sta_resources = StackResources::<4>::new();
    let (sta_stack, mut sta_runner) = embassy_net::new(
        interfaces.station,
        sta_net_config,
        &mut sta_resources,
        0x0fed_cba9_8765_4321,
    );

    let submitted = Signal::<NoopRawMutex, DeviceConfig>::new();

    let access_point = async {
        debug!(
            "Binding DHCP server to UDP port {}",
            dhcp_io::DEFAULT_SERVER_PORT
        );
        let udp_buffers = UdpBuffers::<1>::new();
        let udp = Udp::new(ap_stack, &udp_buffers);
        let mut dhcp_socket = match udp
            .bind(SocketAddr::new(
                IpAddr::V4(Ipv4Addr::UNSPECIFIED),
                dhcp_io::DEFAULT_SERVER_PORT,
            ))
            .await
        {
            Ok(socket) => socket,
            Err(_) => {
                warn!("Failed to bind DHCP server socket");
                return Err(ConfigPortalError::Network);
            }
        };

        let mut dhcp_buffer = [0u8; 1500];
        let mut gateway_buffer = [PORTAL_ADDRESS];
        let mut dhcp_server = DhcpServer::<_, 8>::new_with_et(PORTAL_ADDRESS);
        let mut dhcp_options = ServerOptions::new(PORTAL_ADDRESS, Some(&mut gateway_buffer));
        dhcp_options.captive_url = Some(PORTAL_URL);

        debug!("DHCP will advertise gateway and captive URL {PORTAL_URL} as {PORTAL_ADDRESS}");

        let dhcp = async {
            match dhcp_io::server::run(
                &mut dhcp_server,
                &dhcp_options,
                &mut dhcp_socket,
                &mut dhcp_buffer,
            )
            .await
            {
                Ok(()) => warn!("DHCP server stopped unexpectedly"),
                Err(_) => warn!("DHCP server failed"),
            }

            Err(ConfigPortalError::Network)
        };

        info!("Captive portal servers ready (DHCP, HTTP)");

        let (dhcp_result, ()) = join(dhcp, serve_http(ap_stack, &submitted)).await;

        dhcp_result
    };

    let station = async {
        if config.set {
            debug!("Saved configuration found; attempting station connection");

            if try_connect(&mut controller, sta_stack, config, &ap_config).await {
                return Ok::<(), ConfigPortalError>(());
            }
        }

        loop {
            debug!("Waiting for configuration submission");
            let new_config = submitted.wait().await;
            info!(
                "Received configuration: ssid={:?}, server={:?}, password set: {}",
                new_config.ssid_str(),
                new_config.server_str(),
                new_config.pw_len != 0
            );
            *config = new_config;

            if try_connect(&mut controller, sta_stack, config, &ap_config).await {
                return Ok::<(), ConfigPortalError>(());
            }
        }
    };

    let network = async {
        let (_, _, result) = join3(ap_runner.run(), sta_runner.run(), access_point).await;
        result
    };

    match select(network, station).await {
        Either::First(result) => {
            warn!("Portal network task stopped before configuration completed");

            result?;
            Err(ConfigPortalError::Network)
        }
        Either::Second(result) => {
            match &result {
                Ok(()) => info!("Station is ready; stopping configuration portal"),
                Err(_) => warn!("Station failed to become ready"),
            }

            result?;
            Ok((controller, interfaces))
        }
    }
}

async fn try_connect(
    controller: &mut WifiController<'_>,
    sta_stack: embassy_net::Stack<'_>,
    config: &DeviceConfig,
    ap_config: &AccessPointConfig,
) -> bool {
    debug!("Attempting station connection to {:?}", config.ssid_str());

    let wifi_config = WifiConfig::AccessPointStation(station_config(config), ap_config.clone());

    if controller.set_config(&wifi_config).is_err() {
        warn!("Wi-Fi controller rejected station configuration");
        return false;
    }

    let connected = with_timeout(Duration::from_secs(20), async {
        if controller.connect_async().await.is_err() {
            warn!("Station association failed");
            return false;
        }

        debug!("Waiting for station DHCP configuration");
        sta_stack.wait_config_up().await;

        true
    })
    .await;

    match connected {
        Ok(true) => {
            info!("Station connected to {:?}", config.ssid_str());
            true
        }
        Ok(false) => false,
        Err(_) => {
            warn!("Station connection timed out after 20 seconds");
            false
        }
    }
}

fn station_config(config: &DeviceConfig) -> StationConfig {
    let password = config.pw();
    let auth_method = if password.is_empty() {
        AuthenticationMethod::None
    } else {
        AuthenticationMethod::Wpa2Personal
    };

    StationConfig::default()
        .with_ssid(config.ssid())
        .with_password(password)
        .with_auth_method(auth_method)
}
