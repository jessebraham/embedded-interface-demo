use std::{
    convert::TryInto,
    ffi::CString,
    sync::{Arc, Condvar, Mutex},
};

use anyhow::Result;
use embedded_svc::httpd::{registry::Registry, *};
use esp_idf_hal::chip_info::ChipInfo;
use esp_idf_svc::{
    httpd::ServerRegistry,
    log::EspLogger,
    netif::EspNetifStack,
    nvs::EspDefaultNvs,
    sysloop::EspSysLoopStack,
};
use esp_idf_sys::*;
use json;
use log::info;

// WiFI soft AP configuration.
// To disable authentication use an empty string as the password.
const WIFI_SSID: &str = "ESP32-C3 Soft AP";
const WIFI_PASS: &str = "Password123";
const WIFI_CHAN: u8 = 6;
const WIFI_CONN: u8 = 3;

// DHCP configuration.
const DHCP_IP: &str = "10.0.0.1";
const DHCP_GW: &str = "10.0.0.1";
const DHCP_NM: &str = "255.255.255.0";

fn main() -> Result<()> {
    link_patches();
    EspLogger::initialize_default();

    let _default_nvs = Arc::new(EspDefaultNvs::new()?);
    let _netif_stack = Arc::new(EspNetifStack::new()?);
    let _sys_loop_stack = Arc::new(EspSysLoopStack::new()?);

    init_soft_ap()?;

    print_startup_message();

    let server = ApplicationServer::new();
    server.start()?;

    Ok(())
}

fn print_startup_message() {
    info!("");
    info!("--------------------------------------------------------------");
    info!(
        "Wi-Fi soft AP started, up to {} clients can connect using:",
        WIFI_CONN
    );
    info!("");
    info!("SSID:     {}", WIFI_SSID);
    info!("PASSWORD: {}", WIFI_PASS);
    info!("");
    info!("Web server listening at: http://{}", DHCP_IP);
    info!("--------------------------------------------------------------");
    info!("");
}

// ---------------------------------------------------------------------------
// Convenience Macros

macro_rules! cstr {
    ($input:expr) => {
        CString::new($input)?.into_bytes_with_nul().as_mut_ptr() as *mut i8
    };
}

macro_rules! set_ip {
    ($input:expr, $output:expr) => {
        esp_result!(esp_netif_str_to_ip4(cstr!($input), $output), ())?
    };
}

macro_rules! handler {
    ($uri:expr, $method:ident, $handler:expr) => {
        Handler::new($uri, Method::$method, $handler)
    };
}

// ---------------------------------------------------------------------------
// Wi-Fi Soft AP

fn init_soft_ap() -> Result<()> {
    let wifi_init_config = wifi_init_config_default();
    let mut wifi_config = build_wifi_config()?;

    // Such unsafe wow
    unsafe {
        let soft_ap = esp_netif_create_default_wifi_ap();
        configure_dhcp(soft_ap)?;

        esp_result!(esp_wifi_init(&wifi_init_config), ())?;
        esp_result!(esp_wifi_set_mode(wifi_mode_t_WIFI_MODE_AP), ())?;
        esp_result!(
            esp_wifi_set_config(
                wifi_interface_t_WIFI_IF_AP,
                &mut wifi_config as *mut wifi_config_t,
            ),
            ()
        )?;

        esp_result!(esp_wifi_start(), ())?;
    }

    Ok(())
}

fn wifi_init_config_default() -> wifi_init_config_t {
    // TODO: once the `WIFI_INIT_CONFIG_DEFAULT` macro has been wrapped or emulated
    //       in `esp-idf-sys`, use that instead.
    wifi_init_config_t {
        event_handler: Some(esp_event_send_internal),
        osi_funcs: unsafe { &mut g_wifi_osi_funcs as *mut wifi_osi_funcs_t },
        wpa_crypto_funcs: unsafe { g_wifi_default_wpa_crypto_funcs },
        static_rx_buf_num: CONFIG_ESP32_WIFI_STATIC_RX_BUFFER_NUM as i32,
        dynamic_rx_buf_num: CONFIG_ESP32_WIFI_DYNAMIC_RX_BUFFER_NUM as i32,
        tx_buf_type: CONFIG_ESP32_WIFI_TX_BUFFER_TYPE as i32,
        static_tx_buf_num: WIFI_STATIC_TX_BUFFER_NUM as i32,
        dynamic_tx_buf_num: WIFI_DYNAMIC_TX_BUFFER_NUM as i32,
        cache_tx_buf_num: WIFI_CACHE_TX_BUFFER_NUM as i32,
        csi_enable: WIFI_CSI_ENABLED as i32,
        ampdu_rx_enable: WIFI_AMPDU_RX_ENABLED as i32,
        ampdu_tx_enable: WIFI_AMPDU_TX_ENABLED as i32,
        amsdu_tx_enable: WIFI_AMSDU_TX_ENABLED as i32,
        nvs_enable: WIFI_NVS_ENABLED as i32,
        nano_enable: WIFI_NANO_FORMAT_ENABLED as i32,
        rx_ba_win: WIFI_DEFAULT_RX_BA_WIN as i32,
        wifi_task_core_id: WIFI_TASK_CORE_ID as i32,
        beacon_max_len: WIFI_SOFTAP_BEACON_MAX_LEN as i32,
        mgmt_sbuf_num: WIFI_MGMT_SBUF_NUM as i32,
        feature_caps: unsafe { g_wifi_feature_caps },
        sta_disconnected_pm: WIFI_STA_DISCONNECTED_PM_ENABLED != 0,
        magic: WIFI_INIT_CONFIG_MAGIC as i32,
    }
}

fn build_wifi_config() -> Result<wifi_config_t> {
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

fn configure_dhcp(soft_ap: *mut esp_netif_obj) -> Result<()> {
    let mut ip_info = esp_netif_ip_info_t::default();

    unsafe {
        set_ip!(DHCP_IP, &mut ip_info.ip);
        set_ip!(DHCP_GW, &mut ip_info.gw);
        set_ip!(DHCP_NM, &mut ip_info.netmask);

        esp_result!(esp_netif_dhcps_stop(soft_ap), ())?;
        esp_result!(esp_netif_set_ip_info(soft_ap, &ip_info), ())?;
        esp_result!(esp_netif_dhcps_start(soft_ap), ())?;
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Web Server

#[derive(Debug, Clone)]
struct ApplicationServer {
    mutex: Arc<(Mutex<Option<u32>>, Condvar)>,
}

impl ApplicationServer {
    pub fn new() -> Self {
        Self {
            mutex: Arc::new((Mutex::new(None), Condvar::new())),
        }
    }

    pub fn start(&self) -> Result<()> {
        // TODO: convert to HTTPS server
        let _server = ServerRegistry::new()
            .handler(handler!("/", Get, Self::index_html_get_handler))?
            .handler(handler!("/api/info", Get, Self::system_info_get_handler))?
            .start(&Default::default())?;

        let mut wait = self.mutex.0.lock().unwrap();

        let _cycles = loop {
            if let Some(cycles) = *wait {
                break cycles;
            } else {
                wait = self.mutex.1.wait(wait).unwrap();
            }
        };

        Ok(())
    }

    fn index_html_get_handler(_request: Request) -> Result<Response> {
        let response = Response::new(200)
            .content_encoding("gzip")
            .content_type("text/html")
            .body(Body::Bytes(
                include_bytes!("../resources/index.html.gz").to_vec(),
            ));

        Ok(response)
    }

    fn system_info_get_handler(_request: Request) -> Result<Response> {
        let info = ChipInfo::new();

        let model = info.model.unwrap().to_string();
        let features = info
            .features
            .iter()
            .map(|f| f.to_string())
            .collect::<Vec<_>>();

        let payload = json::object! {
            code: 200,
            success: true,
            data: {
                model: model,
                revision: info.revision,
                cores: info.cores,
                features: features,
            },
        };

        let response = Response::new(200)
            .content_type("application/json")
            .body(Body::Bytes(payload.to_string().as_bytes().to_vec()));

        Ok(response)
    }
}
