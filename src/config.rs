pub struct Config {
    pub wifi_ssid: String,
    pub wifi_psw: String,
}

impl Config {
    pub fn from_env() -> Self {
        let wifi_ssid = env!("WIFI_SSID").to_string();
        let wifi_psw = env!("WIFI_PSW").to_string();
        Config { wifi_ssid, wifi_psw }
    }
}
