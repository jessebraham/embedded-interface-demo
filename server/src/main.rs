use std::{
    sync::{Arc, Mutex},
    thread::sleep,
    time::Duration,
};

use anyhow::Result;
use embedded_hal::digital::blocking::OutputPin;
use embedded_svc::{
    http::{
        server::{registry::Registry, Body, ResponseData},
        SendHeaders,
    },
    ipv4::{Ipv4Addr, Mask, RouterConfiguration, Subnet},
    wifi::{AccessPointConfiguration, AuthMethod, Configuration as WifiConfiguration, Wifi},
};
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_svc::{
    http::server::{Configuration as ServerConfiguration, EspHttpRequest, EspHttpServer},
    log::EspLogger,
    netif::EspNetifStack,
    nvs::EspDefaultNvs,
    sysloop::EspSysLoopStack,
    wifi::EspWifi,
};
use esp_idf_sys::*;
use log::info;

use self::chip_info::ChipInfo;

mod chip_info;

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

    // Set a GPIO as an output pin, and initially set its state HIGH (as we are
    // driving an LED in an active-low configuration). One day this will use the
    // built-in WS2812 via RMT instead.
    let peripherals = Peripherals::take().unwrap();
    let mut led = peripherals.pins.gpio5.into_output()?;
    led.set_high()?;

    // Since the LED needs to be shared among handlers, we must first wrap it in a
    // Mutex to ensure we don't have any races.
    let led_mutex = Arc::new(Mutex::new(led));
    let on_mutex = led_mutex.clone();
    let off_mutex = led_mutex.clone();

    // Initialize the Wi-Fi radio and configure it as a soft access point.
    let _wifi = initialize_soft_ap()?;

    // Start the web server and register all routes/handlers.
    let mut server = EspHttpServer::new(&ServerConfiguration::default())?;
    server
        .at("/")
        .get(index_html_get_handler)?
        .at("/api/info")
        .get(system_info_get_handler)?
        .at("/api/light/on")
        .get(move |request| light_on_get_handler(request, &on_mutex))?
        .at("/api/light/off")
        .get(move |request| light_off_get_handler(request, &off_mutex))?;

    // Print the startup message, then spin for eternity so that the server does not
    // get dropped!
    print_startup_message();
    loop {
        sleep(Duration::from_secs(1));
    }
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
    wifi.set_configuration(&WifiConfiguration::AccessPoint(config))?;

    Ok(wifi)
}

fn index_html_get_handler(_request: &mut EspHttpRequest) -> Result<ResponseData> {
    let response = ResponseData::new(200)
        .content_encoding("gzip")
        .content_type("text/html")
        .body(Body::Bytes(
            include_bytes!("../resources/index.html.gz").to_vec(),
        ));

    Ok(response)
}

fn system_info_get_handler(_request: &mut EspHttpRequest) -> Result<ResponseData> {
    let chip_info = ChipInfo::new();
    let response = ResponseData::from_json(&chip_info)?;

    Ok(response)
}

fn light_on_get_handler<T>(
    _request: &mut EspHttpRequest,
    mutex: &Arc<Mutex<T>>,
) -> Result<ResponseData>
where
    T: OutputPin,
{
    if let Ok(mut led) = mutex.lock() {
        led.set_low().unwrap();
        Ok(ResponseData::new(200))
    } else {
        Ok(ResponseData::new(500))
    }
}

fn light_off_get_handler<T>(
    _request: &mut EspHttpRequest,
    mutex: &Arc<Mutex<T>>,
) -> Result<ResponseData>
where
    T: OutputPin,
{
    if let Ok(mut led) = mutex.lock() {
        led.set_high().unwrap();
        Ok(ResponseData::new(200))
    } else {
        Ok(ResponseData::new(500))
    }
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
