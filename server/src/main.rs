use std::{convert::TryInto, sync::Arc};

use anyhow::Result;
use esp_idf_svc::{
    log::EspLogger,
    netif::EspNetifStack,
    nvs::EspDefaultNvs,
    sysloop::EspSysLoopStack,
};
use esp_idf_sys::*;
use log::info;

// WiFI soft AP configuration.
// To disable authentication use an empty string as the password.
const WIFI_SSID: &str = "ESP32-C3 Soft AP";
const WIFI_PASS: &str = "Password123";
const WIFI_CHAN: u8 = 6;
const WIFI_CONN: u8 = 3;

fn main() -> Result<()> {
    link_patches();
    EspLogger::initialize_default();

    #[allow(unused)]
    let default_nvs = Arc::new(EspDefaultNvs::new()?);
    #[allow(unused)]
    let netif_stack = Arc::new(EspNetifStack::new()?);
    #[allow(unused)]
    let sys_loop_stack = Arc::new(EspSysLoopStack::new()?);

    init_soft_ap()?;

    Ok(())
}

fn init_soft_ap() -> Result<()> {
    info!("WiFi soft AP started");
    info!("SSID: {}\tPASSWORD: {}", WIFI_SSID, WIFI_PASS);

    Ok(())
}

fn build_wifi_config() -> Result<wifi_config_t> {
    // Configure the WiFi peripheral as a soft AP, using the specificed
    // SSID, password, channel, and max connections.
    let mut ap = wifi_ap_config_t::default();
    ap.ssid = string_to_array(WIFI_SSID)?;
    ap.ssid_len = WIFI_SSID.len() as u8;
    ap.password = string_to_array(WIFI_PASS)?;
    ap.channel = WIFI_CHAN;
    ap.max_connection = WIFI_CONN;

    ap.authmode = if WIFI_PASS.len() == 0 {
        wifi_auth_mode_t_WIFI_AUTH_OPEN
    } else {
        wifi_auth_mode_t_WIFI_AUTH_WPA_WPA2_PSK
    };

    Ok(wifi_config_t { ap })
}

fn string_to_array<const N: usize>(s: &str) -> Result<[u8; N]> {
    // In order for `try_into` to convert from `&[u8]` to `[u8; N]`, we must first
    // ensure that the string's length is N. We accomplish this by padding the end
    // of the string with null ('\0') bytes.
    let padded = format!("{:\0<width$}", s, width = N);
    let array: [u8; N] = padded.as_bytes().try_into()?;

    Ok(array)
}
