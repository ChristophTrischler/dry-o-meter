use core::fmt::Write as _;
use core::net::Ipv4Addr;
use core::str::FromStr;

use embassy_net::Stack;
use embassy_net::tcp::TcpSocket;
use embassy_time::{Duration, Timer};
use embedded_io_async::Write;
use esp_radio::wifi::WifiController;
use log::{debug, info, warn};

use crate::config::{Config as DeviceConfig, UUID_LEN};
use crate::sensor::{Reading, Sensor};

const HTTP_TIMEOUT: Duration = Duration::from_secs(10);
const CLAIM_RETRY_DELAY: Duration = Duration::from_secs(5);
const CLAIM_PATH: &str = "/api/controllers/uuid/claim-latest";

#[derive(Clone, Copy)]
struct Endpoint {
    host: Ipv4Addr,
    port: u16,
}

impl Endpoint {
    fn parse(server: &[u8]) -> Option<Self> {
        let server = core::str::from_utf8(server).ok()?.trim();
        let server = server.strip_prefix("http://").unwrap_or(server);
        let authority = server.split('/').next()?;

        if authority.is_empty() {
            return None;
        }

        let (host, port) = match authority.rsplit_once(':') {
            Some((host, port)) => (host, port.parse().ok()?),
            None => (authority, 80),
        };

        let host = match Ipv4Addr::from_str(host) {
            Ok(host) => host,
            Err(_) => {
                warn!("Server host is not a valid IPv4 address");
                return None;
            }
        };

        Some(Self { host, port })
    }
}

async fn request(
    stack: Stack<'_>,
    endpoint: Endpoint,
    method: &str,
    path: &str,
    body: Option<&str>,
    response: &mut [u8],
) -> Option<usize> {
    let mut rx_buffer = [0u8; 1024];
    let mut tx_buffer = [0u8; 1024];
    let mut socket = TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer);
    socket.set_timeout(Some(HTTP_TIMEOUT));

    debug!(
        "HTTP request: method={method}, path={path}, target={}:{}, body={} bytes",
        endpoint.host,
        endpoint.port,
        body.map_or(0, str::len)
    );

    if socket
        .connect((endpoint.host, endpoint.port))
        .await
        .is_err()
    {
        warn!("Failed to connect to backend {}", endpoint.host);
        return None;
    }

    let mut header = heapless::String::<256>::new();
    let _ = write!(
        header,
        "{method} {path} HTTP/1.1\r\n\
         Host: {}:{}\r\n",
        endpoint.host, endpoint.port
    );

    if let Some(body) = body {
        let _ = write!(
            header,
            "Content-Type: application/json\r\nContent-Length: {}\r\n",
            body.len()
        );
    }

    let _ = write!(header, "Connection: close\r\n\r\n");

    socket.write_all(header.as_bytes()).await.ok()?;

    if let Some(body) = body {
        socket.write_all(body.as_bytes()).await.ok()?;
    }

    socket.flush().await.ok()?;

    let mut len = 0;

    while len < response.len() {
        match socket.read(&mut response[len..]).await {
            Ok(0) => break,
            Ok(read) => len += read,
            Err(_) => {
                warn!("Failed to read backend response");
                break;
            }
        }
    }

    socket.close();
    let _ = socket.flush().await;

    debug!("Backend response: {len} bytes");
    (len > 0).then_some(len)
}

fn is_success(response: &[u8]) -> bool {
    response.starts_with(b"HTTP/1.1 2")
}

async fn claim_uuid(stack: Stack<'_>, endpoint: Endpoint) -> Option<[u8; UUID_LEN]> {
    debug!("Claiming controller UUID from backend");

    let mut response = [0u8; 512];
    let len = match request(stack, endpoint, "GET", CLAIM_PATH, None, &mut response).await {
        Some(len) => len,
        None => {
            warn!("UUID claim request failed");
            return None;
        }
    };

    if !is_success(&response[..len]) {
        warn!("UUID claim returned a non-success response");
        return None;
    }

    let Some(response) = core::str::from_utf8(&response[..len]).ok() else {
        warn!("UUID claim response is not valid UTF-8");
        return None;
    };

    let Some(uuid) = parse_uuid(response) else {
        warn!("Failed to parse UUID from claim response");
        return None;
    };

    info!("Claimed controller UUID {uuid}");

    uuid.as_bytes().try_into().ok()
}

fn parse_uuid(response: &str) -> Option<&str> {
    let key = response.find("\"uuid\"")? + 6;
    let value = &response[key..];
    let value = &value[value.find(':')? + 1..];
    let value = value.trim_start().strip_prefix('"')?;

    Some(&value[..value.find('"')?])
}

async fn post_reading(
    stack: Stack<'_>,
    endpoint: Endpoint,
    uuid: &[u8],
    reading: Reading,
    rssi: Option<i16>,
) -> bool {
    let Ok(uuid) = core::str::from_utf8(uuid) else {
        warn!("Stored UUID is not valid UTF-8");
        return false;
    };

    debug!(
        "Posting reading: humidity={}, raw={}, rssi={rssi:?}",
        reading.humidity, reading.raw
    );

    let mut path = heapless::String::<80>::new();
    let _ = write!(path, "/api/controllers/uuid/{uuid}/readings");

    let mut body = heapless::String::<96>::new();
    let _ = write!(body, "{{\"humidity\":{}", reading.humidity);

    if let Some(rssi) = rssi {
        let _ = write!(body, ",\"rssi\":{rssi}");
    }

    let _ = write!(body, "}}");

    let mut response = [0u8; 512];
    let Some(len) = request(stack, endpoint, "POST", &path, Some(&body), &mut response).await
    else {
        warn!("Reading post request failed");
        return false;
    };

    if !is_success(&response[..len]) {
        warn!("Reading post returned a non-success response");
        return false;
    }

    info!("Posted humidity reading {}", reading.humidity);

    true
}

pub async fn run(
    stack: Stack<'_>,
    config: &mut DeviceConfig,
    controller: &mut WifiController<'_>,
    sensor: &mut Sensor<'_>,
) {
    debug!("Waiting for station network configuration");
    stack.wait_config_up().await;
    info!("Station network is up");

    let Some(endpoint) = Endpoint::parse(config.server()) else {
        warn!(
            "Cannot report readings: invalid server address {:?}",
            config.server_str()
        );
        return;
    };

    debug!(
        "Using backend {}:{} for server {:?}",
        endpoint.host,
        endpoint.port,
        config.server_str()
    );

    if config.uuid_len == 0 {
        let mut attempt = 0u32;

        while config.uuid_len == 0 {
            attempt += 1;
            debug!("Attempting to claim controller UUID (attempt {attempt})");

            if let Some(uuid) = claim_uuid(stack, endpoint).await {
                config.set_uuid(&uuid);
            } else {
                warn!("No controller UUID received; retrying in {CLAIM_RETRY_DELAY:?}");
                Timer::after(CLAIM_RETRY_DELAY).await;
            }
        }
    } else {
        debug!("Using stored controller UUID {}", config.uuid_str());
    }

    let reading = sensor.read().await;
    let rssi = controller
        .rssi()
        .ok()
        .and_then(|rssi| i16::try_from(rssi).ok());

    debug!(
        "Collected reading: raw={}, humidity={}, rssi={rssi:?}",
        reading.raw, reading.humidity
    );

    if !post_reading(stack, endpoint, config.uuid(), reading, rssi).await {
        warn!("Failed to post reading");
    }
}
