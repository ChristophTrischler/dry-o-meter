use core::fmt::Write as _;

use embassy_net::tcp::{Error as TcpError, State as TcpState, TcpSocket};
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::signal::Signal;
use embassy_time::{Duration, Timer};
use embedded_io_async::Write;
use log::{debug, info, warn};

use crate::config::Config as DeviceConfig;
use crate::form::config_from_form;

const HTTP_PORT: u16 = 80;
const PORTAL_URL: &str = "http://192.168.4.1/";

const CONFIG_PAGE: &str = include_str!("./html/index.html");
const SAVED_PAGE: &str = include_str!("./html/saved.html");
const ERROR_PAGE: &str = include_str!("./html/error.html");

pub(crate) async fn serve_http(
    stack: embassy_net::Stack<'_>,
    submitted: &Signal<NoopRawMutex, DeviceConfig>,
) {
    let mut rx_buffer = [0u8; 1024];
    let mut tx_buffer = [0u8; 1024];

    debug!("HTTP server listening on port {HTTP_PORT}");

    loop {
        let mut socket = TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer);
        socket.set_timeout(Some(Duration::from_secs(20)));

        if socket.accept(HTTP_PORT).await.is_err() {
            warn!("HTTP accept failed");
            continue;
        }

        debug!("Accepted HTTP connection");
        let mut request = [0u8; 1536];
        let reply = match read_request(&mut socket, &mut request).await {
            Some(len) => {
                debug!("Read HTTP request with {len} bytes");
                handle_request(&request[..len], submitted)
            }
            None => {
                warn!("Failed to read complete HTTP request");
                Reply::Page(400, "Bad Request", ERROR_PAGE)
            }
        };

        if write_response(&mut socket, reply).await.is_err() {
            warn!("Failed to write HTTP response");
        }

        socket.close();
        let _ = socket.flush().await;

        for _ in 0..20 {
            if matches!(socket.state(), TcpState::Closed | TcpState::TimeWait) {
                break;
            }

            Timer::after(Duration::from_millis(10)).await;
        }

        socket.abort();
        let _ = socket.flush().await;
    }
}

async fn read_request(socket: &mut TcpSocket<'_>, buffer: &mut [u8]) -> Option<usize> {
    let mut len = 0;

    loop {
        if let Some(header_end) = find_subslice(&buffer[..len], b"\r\n\r\n") {
            let body_start = header_end + 4;
            let body_len = content_length(&buffer[..header_end]).unwrap_or(0);
            let total = body_start.saturating_add(body_len).min(buffer.len());

            while len < total {
                let read = socket.read(&mut buffer[len..total]).await.ok()?;

                if read == 0 {
                    break;
                }

                len += read;
            }

            return Some(len);
        }

        if len == buffer.len() {
            return None;
        }

        let read = socket.read(&mut buffer[len..]).await.ok()?;

        if read == 0 {
            return None;
        }

        len += read;
    }
}

fn content_length(headers: &[u8]) -> Option<usize> {
    headers.split(|byte| *byte == b'\n').find_map(|line| {
        let line = line.strip_suffix(b"\r").unwrap_or(line);
        let colon = line.iter().position(|byte| *byte == b':')?;
        let name = &line[..colon];
        let value = &line[colon + 1..];

        if name.eq_ignore_ascii_case(b"content-length") {
            core::str::from_utf8(value).ok()?.trim().parse().ok()
        } else {
            None
        }
    })
}

fn find_subslice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() {
        return Some(0);
    }

    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

#[derive(Clone, Copy)]
enum Reply {
    Page(u16, &'static str, &'static str),
    Redirect,
}

fn handle_request(request: &[u8], submitted: &Signal<NoopRawMutex, DeviceConfig>) -> Reply {
    let Some(request) = parse_request(request) else {
        warn!("Rejected malformed HTTP request");
        return Reply::Page(400, "Bad Request", ERROR_PAGE);
    };

    debug!(
        "HTTP request: method={:?}, path={:?}, body={} bytes",
        core::str::from_utf8(request.method).unwrap_or("<invalid>"),
        core::str::from_utf8(request.path).unwrap_or("<invalid>"),
        request.body.len()
    );

    match (request.method, request.path) {
        (b"GET", b"/") => {
            debug!("Serving configuration page");
            Reply::Page(200, "OK", CONFIG_PAGE)
        }
        (b"GET", _) => {
            debug!("Redirecting captive-portal request to {PORTAL_URL}");
            Reply::Redirect
        }
        (b"POST", b"/config") => match config_from_form(request.body) {
            Some(new_config) => {
                info!(
                    "Configuration submitted: ssid={:?}, server={:?}, password set: {}",
                    new_config.ssid_str(),
                    new_config.server_str(),
                    new_config.pw_len != 0
                );
                submitted.signal(new_config);
                Reply::Page(200, "OK", SAVED_PAGE)
            }
            None => {
                warn!("Rejected invalid configuration form");
                Reply::Page(400, "Bad Request", ERROR_PAGE)
            }
        },
        _ => {
            warn!("Rejected unsupported HTTP request");
            Reply::Page(405, "Method Not Allowed", ERROR_PAGE)
        }
    }
}

struct Request<'a> {
    method: &'a [u8],
    path: &'a [u8],
    body: &'a [u8],
}

fn parse_request(buffer: &[u8]) -> Option<Request<'_>> {
    let header_end = find_subslice(buffer, b"\r\n\r\n")?;
    let headers = &buffer[..header_end];
    let body = &buffer[header_end + 4..];

    let mut lines = headers.split(|byte| *byte == b'\n');
    let request_line = lines.next()?;
    let request_line = request_line.strip_suffix(b"\r").unwrap_or(request_line);
    let mut parts = request_line.split(|byte| *byte == b' ');
    let method = parts.next()?;
    let path = parts.next()?.split(|byte| *byte == b'?').next()?;
    let version = parts.next()?;

    if !version.starts_with(b"HTTP/1.") {
        return None;
    }

    let body_len = content_length(headers).unwrap_or(0);

    Some(Request {
        method,
        path,
        body: body.get(..body_len)?,
    })
}

async fn write_response(socket: &mut TcpSocket<'_>, reply: Reply) -> Result<(), TcpError> {
    let (status, reason, body, location) = match reply {
        Reply::Page(status, reason, body) => (status, reason, body, None),
        Reply::Redirect => (302, "Found", "", Some(PORTAL_URL)),
    };

    debug!(
        "Writing HTTP response: {status} {reason}, {} bytes",
        body.len()
    );

    let mut header = heapless::String::<192>::new();
    let _ = write!(
        header,
        "HTTP/1.1 {status} {reason}\r\n\
         Content-Type: text/html; charset=utf-8\r\n\
         Content-Length: {}\r\n\
         Cache-Control: no-store\r\n\
         Connection: close\r\n",
        body.len()
    );

    if let Some(location) = location {
        let _ = write!(header, "Location: {location}\r\n");
    }

    let _ = write!(header, "\r\n");

    socket.write_all(header.as_bytes()).await?;
    socket.write_all(body.as_bytes()).await?;
    socket.flush().await?;

    Ok(())
}
