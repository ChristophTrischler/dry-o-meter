use alloc::string::String;
use esp_hal::Persistable;

pub const UUID_LEN: usize = 36;

#[derive(Debug)]
pub struct Config {
    pub ssid: [u8; 32], // 32 is max ssid length
    pub pw: [u8; 64],   //64 is max pw length
    pub server: [u8; 96],
    pub uuid: [u8; UUID_LEN],
    pub ssid_len: u8,
    pub pw_len: u8,
    pub server_len: u8,
    pub uuid_len: u8,
    pub set: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

impl Config {
    pub const fn new() -> Self {
        // Read SSID and PW from the environment as compile-time defaults. When the
        // values are present the device is considered configured (direct Wi-Fi
        // connect); otherwise the existing setup-portal flow is used.

        let mut set = false;

        let mut ssid = [0; 32];

        let ssid_len = if let Some(ssid_str) = option_env!("SSID") {
            let bytes = ssid_str.as_bytes();
            ssid[..bytes.len()].copy_from_slice(bytes);
            set = true;
            ssid_str.len() as u8
        } else {
            0
        };

        let mut pw = [0; 64];
        let pw_len = if let Some(pw_str) = option_env!("PW") {
            let bytes = pw_str.as_bytes();
            pw[..bytes.len()].copy_from_slice(bytes);
            pw_str.len() as u8
        } else {
            0
        };

        let mut server = [0; 96];
        let server_len = if let Some(server_str) = option_env!("SERVER") {
            let bytes = server_str.as_bytes();
            server[..bytes.len()].copy_from_slice(bytes);
            bytes.len() as u8
        } else {
            0
        };

        Self {
            ssid,
            pw,
            server,
            uuid: [0; UUID_LEN],
            ssid_len,
            pw_len,
            server_len,
            uuid_len: 0,
            set,
        }
    }

    pub fn ssid(&self) -> &[u8] {
        &self.ssid[0..(self.ssid_len as usize)]
    }

    pub fn ssid_str(&self) -> &str {
        core::str::from_utf8(self.ssid()).unwrap_or("<invalid utf-8>")
    }

    pub fn pw(&self) -> String {
        let v = &self.pw[0..(self.pw_len as usize)];

        String::from_utf8_lossy(v).into_owned()
    }

    pub fn server(&self) -> &[u8] {
        &self.server[0..(self.server_len as usize)]
    }

    pub fn server_str(&self) -> &str {
        core::str::from_utf8(self.server()).unwrap_or("<invalid utf-8>")
    }

    pub fn uuid(&self) -> &[u8] {
        &self.uuid[0..(self.uuid_len as usize)]
    }

    pub fn uuid_str(&self) -> &str {
        core::str::from_utf8(self.uuid()).unwrap_or("<invalid utf-8>")
    }

    pub fn set_uuid(&mut self, uuid: &[u8]) -> bool {
        if uuid.len() != UUID_LEN {
            log::warn!("Rejected UUID with invalid length {}", uuid.len());
            return false;
        }

        self.uuid.copy_from_slice(uuid);
        self.uuid_len = UUID_LEN as u8;

        log::debug!("Stored controller UUID {}", self.uuid_str());

        true
    }
}

// only u8 arreys
unsafe impl Persistable for Config {}
