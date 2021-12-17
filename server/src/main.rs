use std::{
    convert::TryInto,
    ffi::CString,
    sync::{Arc, Condvar, Mutex},
};

use anyhow::Result;
use embedded_svc::httpd::registry::Registry;
use esp_idf_svc::{
    httpd::ServerRegistry,
    log::EspLogger,
    netif::EspNetifStack,
    nvs::EspDefaultNvs,
    sysloop::EspSysLoopStack,
};
use esp_idf_sys::{c_types::c_void, *};
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

    run_server()?;

    Ok(())
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

        esp_result!(
            esp_event_handler_instance_register(
                WIFI_EVENT,
                ESP_EVENT_ANY_ID,
                Some(wifi_event_handler),
                0 as *mut c_void,
                0 as *mut esp_event_handler_instance_t,
            ),
            ()
        )?;

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

    print_startup_message();

    Ok(())
}

unsafe extern "C" fn wifi_event_handler(
    _arg: *mut c_void,
    _event_base: *const i8,
    event_id: i32,
    event_data: *mut c_void,
) {
    #[allow(non_upper_case_globals)]
    match event_id as u32 {
        wifi_event_t_WIFI_EVENT_AP_STACONNECTED => {
            let event = *event_data.cast::<wifi_event_ap_staconnected_t>();
            info!("New client:  {:#?}", event.mac);
        }
        wifi_event_t_WIFI_EVENT_AP_STADISCONNECTED => {
            let event = *event_data.cast::<wifi_event_ap_stadisconnected_t>();
            info!("Client left: {:#?}", event.mac);
        }
        _ => { /* Ignore all other events for now */ }
    }
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
    let ip_info = esp_netif_ip_info_t {
        ip: str_to_ip4_addr(DHCP_IP)?,
        gw: str_to_ip4_addr(DHCP_GW)?,
        netmask: str_to_ip4_addr(DHCP_NM)?,
    };

    unsafe {
        esp_result!(esp_netif_dhcps_stop(soft_ap), ())?;
        esp_result!(esp_netif_set_ip_info(soft_ap, &ip_info), ())?;
        esp_result!(esp_netif_dhcps_start(soft_ap), ())?;
    }

    Ok(())
}

fn str_to_ip4_addr(addr: &str) -> Result<esp_ip4_addr_t> {
    let mut bytes = CString::new(addr)?.into_bytes_with_nul();

    let mut esp_ip = esp_ip4_addr_t::default();
    unsafe {
        esp_netif_str_to_ip4(
            bytes.as_mut_ptr() as *mut i8,
            &mut esp_ip as *mut esp_ip4_addr_t,
        )
    };

    Ok(esp_ip)
}

fn print_startup_message() {
    info!("");
    info!("--------------------------------------------------------------");
    info!(
        "Wi-Fi soft AP has started, up to {} clients can connect using:",
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
// Event Handlers

// ---------------------------------------------------------------------------
// MDNS

// ---------------------------------------------------------------------------
// Web Server

fn run_server() -> Result<()> {
    // TODO: convert to HTTPS server
    let _server = ServerRegistry::new()
        .at("/")
        .get(|_| Ok("Hello from Rust!".into()))?
        .start(&Default::default())?;

    let mutex: Arc<(Mutex<Option<u32>>, Condvar)> = Arc::new((Mutex::new(None), Condvar::new()));
    let mut wait = mutex.0.lock().unwrap();

    let _cycles = loop {
        if let Some(cycles) = *wait {
            break cycles;
        } else {
            wait = mutex.1.wait(wait).unwrap();
        }
    };

    Ok(())
}
