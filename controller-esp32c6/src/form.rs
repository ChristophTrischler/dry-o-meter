use log::{debug, warn};

use crate::config::Config as DeviceConfig;

pub(crate) fn config_from_form(body: &[u8]) -> Option<DeviceConfig> {
    debug!("Parsing configuration form ({} bytes)", body.len());

    let mut decoded = [0u8; 128];
    let mut config = DeviceConfig::new();

    let len = match decode_field(body, b"ssid", &mut decoded) {
        Some(len) => len,
        None => {
            warn!("Missing or invalid SSID field");
            return None;
        }
    };

    if len == 0 || len > config.ssid.len() {
        warn!("Invalid SSID length {len}");
        return None;
    }

    config.ssid[..len].copy_from_slice(&decoded[..len]);
    config.ssid_len = len as u8;
    debug!("Parsed SSID ({} bytes)", len);

    let len = decode_field(body, b"pw", &mut decoded).unwrap_or(0);

    if len > config.pw.len() {
        warn!("Invalid Wi-Fi password length {len}");
        return None;
    }

    config.pw[..len].copy_from_slice(&decoded[..len]);
    config.pw_len = len as u8;
    debug!("Parsed Wi-Fi password ({} bytes)", len);

    let len = match decode_field(body, b"server", &mut decoded) {
        Some(len) => len,
        None => {
            warn!("Missing or invalid server field");
            return None;
        }
    };

    if len == 0 || len > config.server.len() {
        warn!("Invalid server length {len}");
        return None;
    }

    config.server[..len].copy_from_slice(&decoded[..len]);
    config.server_len = len as u8;
    debug!("Parsed server ({} bytes)", len);

    config.set = true;

    Some(config)
}

fn decode_field(body: &[u8], key: &[u8], out: &mut [u8]) -> Option<usize> {
    let value = body.split(|byte| *byte == b'&').find_map(|pair| {
        let mut parts = pair.splitn(2, |byte| *byte == b'=');
        let name = parts.next()?;

        if name == key {
            Some(parts.next().unwrap_or(&[]))
        } else {
            None
        }
    })?;

    let mut written = 0;
    let mut index = 0;

    while index < value.len() {
        let byte = match value[index] {
            b'+' => b' ',
            b'%' => {
                if index + 2 >= value.len() {
                    return None;
                }

                let high = hex_value(value[index + 1])?;
                let low = hex_value(value[index + 2])?;
                index += 2;

                (high << 4) | low
            }
            byte => byte,
        };

        if written >= out.len() {
            return None;
        }

        out[written] = byte;
        written += 1;
        index += 1;
    }

    Some(written)
}

fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}
