use anyhow::{bail, Result};
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::modem::Modem,
    wifi::{
        AuthMethod, ClientConfiguration, Configuration, EspWifi,
    },
};
use log::info;

pub struct Wifi {
    esp_wifi: EspWifi<'static>,
}

impl Wifi {
    pub fn new(modem: Modem, sysloop: EspSystemEventLoop) -> Self {
        Wifi { esp_wifi: EspWifi::new(modem, sysloop.clone(), None).unwrap() }
    }
    pub fn connect(&mut self, ssid: String, pass: String) -> Result<()> {
        let mut auth_method = AuthMethod::WPA2Personal;
        if ssid.is_empty() {
            bail!("Missing WiFi name")
        }
        if pass.is_empty() {
            auth_method = AuthMethod::None;
            info!("Wifi password is empty");
        }
        let wifi = &mut self.esp_wifi;
        // let mut wifi: BlockingWifi<&mut EspWifi<'_>> =
        // BlockingWifi::wrap(&mut esp_wifi, self.sysloop)?;

        wifi.set_configuration(&Configuration::Client(
            ClientConfiguration::default(),
        ))?;
        info!("Starting wifi...");

        wifi.start()?;

        info!("Scanning...");

        let ap_infos = wifi.scan()?;

        let ours = ap_infos.into_iter().find(|a| a.ssid == ssid.as_str());

        let channel = if let Some(ours) = ours {
            info!(
                "Found configured access point {} on channel {}",
                ssid, ours.channel
            );
            Some(ours.channel)
        } else {
            info!(
                "Configured access point {} not found during scanning, will \
                 go with unknown channel",
                ssid
            );
            None
        };

        wifi.set_configuration(&Configuration::Client(ClientConfiguration {
            ssid: ssid
                .as_str()
                .try_into()
                .expect("Could not parse the given SSID into WiFi config"),
            password: pass
                .as_str()
                .try_into()
                .expect("Could not parse the given password into WiFi config"),
            channel,
            auth_method,
            ..Default::default()
        }))?;

        info!("Connecting wifi...");

        wifi.connect()?;

        info!("Waiting for DHCP lease...");

        let ip_info = wifi.sta_netif().get_ip_info()?;

        info!("Wifi DHCP info: {:?}", ip_info);

        Ok(())
    }
}
