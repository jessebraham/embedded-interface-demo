use std::sync::{Arc, Condvar, Mutex};

use anyhow::Result;
use embedded_svc::{
    httpd::{registry::Registry, *},
    ipv4::{Ipv4Addr, Mask, RouterConfiguration, Subnet},
    wifi::{AccessPointConfiguration, AuthMethod, Configuration, Wifi},
};
use esp_idf_hal::chip_info::ChipInfo;
use esp_idf_svc::{
    httpd::ServerRegistry,
    log::EspLogger,
    netif::EspNetifStack,
    nvs::EspDefaultNvs,
    sysloop::EspSysLoopStack,
    wifi::EspWifi,
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
const DHCP_GTWY: Ipv4Addr = Ipv4Addr::new(10, 0, 0, 1);

fn main() -> Result<()> {
    link_patches();
    EspLogger::initialize_default();

    // Initialize the networking services.
    let _wifi = initialize_soft_ap()?;
    print_startup_message();

    let server = ApplicationServer::new();
    server.start()?;

    Ok(())
}

fn initialize_soft_ap() -> Result<EspWifi> {
    let mut config = AccessPointConfiguration::default();

    // Wi-Fi soft AP configuration.
    config.ssid = WIFI_SSID.to_string();
    config.channel = WIFI_CHAN;
    config.max_connections = WIFI_CONN as u16;

    if !WIFI_PASS.is_empty() {
        config.auth_method = AuthMethod::WPAWPA2Personal;
        config.password = WIFI_PASS.to_string();
    }

    // DHCP configuration.
    config.ip_conf = Some(RouterConfiguration {
        subnet: Subnet {
            gateway: DHCP_GTWY,
            mask: Mask(24),
        },
        dhcp_enabled: true,
        dns: None,
        secondary_dns: None,
    });

    // Initialize the required ESP-IDF services.
    let default_nvs = Arc::new(EspDefaultNvs::new()?);
    let netif_stack = Arc::new(EspNetifStack::new()?);
    let sys_loop_stack = Arc::new(EspSysLoopStack::new()?);

    // Initialize the Wi-Fi peripheral using the above configuration.
    let mut wifi = EspWifi::new(netif_stack, sys_loop_stack, default_nvs)?;
    wifi.set_configuration(&Configuration::AccessPoint(config))?;

    Ok(wifi)
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
    info!("Web server listening at: http://{}", DHCP_GTWY);
    info!("--------------------------------------------------------------");
    info!("");
}

// ---------------------------------------------------------------------------
// Web Server

macro_rules! handler {
    ($uri:expr, $method:ident, $handler:expr) => {
        Handler::new($uri, Method::$method, $handler)
    };
}

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
