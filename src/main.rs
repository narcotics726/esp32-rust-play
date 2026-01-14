mod board;
mod config;
mod devices;
mod services;

use anyhow::Result;
use crossbeam_channel::unbounded;
use esp_idf_svc::{eventloop::EspSystemEventLoop, hal::prelude::Peripherals};
use log::info;

use crate::config::Config;

fn format_ms(ms: u128) -> String {
    let total_centis = ms / 10; // 毫秒转成厘秒
    let centis = (total_centis % 100) as u32; // 0-99
    let total_secs = total_centis / 100;
    let secs = (total_secs % 60) as u32; // 0-59
    let mins = (total_secs / 60) as u32; // 分钟可继续滚动
    format!("{:02}:{:02}.{:02}", mins, secs, centis)
}

fn main() -> Result<()> {
    // It is necessary to call this function once. Otherwise, some patches to
    // the runtime implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    let config = Config::from_env();
    log::info!(
        "Config: wifi_ssid={}, wifi_psw={}",
        config.wifi_ssid,
        config.wifi_psw
    );

    let peripherals = Peripherals::take().unwrap();
    let sysloop = EspSystemEventLoop::take()?;

    // create an unbounded channel
    let (snd, rcv) = unbounded::<String>();

    let mut board = board::Board::new();
    board.init(peripherals, sysloop)?;

    let mut wifi = board.wifi.take().unwrap();
    let Config { wifi_ssid, wifi_psw } = config;
    wifi.connect(wifi_ssid, wifi_psw)?;
    let mut server = services::http_server::HTTPServer::new(snd);
    server.start()?;

    let mut audio = board.max98357a.take().unwrap();
    let is_stop = audio.is_stop.clone();
    let mut oled = board.oled.take().unwrap();
    let mut btn1 = board.btn1.take().unwrap();
    info!("Running...");
    std::thread::spawn(move || {
        let _ = audio.play_sample(440.0, 10000);
    });
    let mut all_msgs = String::new();

    loop {
        let elapsed_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_millis();

        if btn1.is_pressed() {
            info!("Button 1 pressed!");
            all_msgs.clear();
            let current_is_stop =
                is_stop.load(std::sync::atomic::Ordering::Relaxed);
            oled.show((
                "Button 1 pressed!",
                current_is_stop.to_string().as_str(),
            ))?;
            is_stop
                .store(!current_is_stop, std::sync::atomic::Ordering::Relaxed);
        } else {
            while let Ok(msg) = rcv.try_recv() {
                all_msgs = msg;
            }
            oled.show((&format_ms(elapsed_time), all_msgs.as_str()))?;
        }
        std::thread::sleep(std::time::Duration::from_millis(1000 / 30));
    }
}
