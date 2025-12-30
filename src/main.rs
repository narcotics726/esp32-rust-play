use anyhow::Result;
use embedded_graphics::{
    mono_font::{MonoTextStyle, ascii::{FONT_6X10, FONT_8X13, FONT_9X15}},
    pixelcolor::BinaryColor,
    prelude::*,
    text::Text,
};
use esp_idf_hal::{
    delay::TickType,
    i2c::{I2cConfig, I2cDriver},
    sys::TickType_t,
    units::FromValueType,
};
use esp_idf_svc::{eventloop::EspSystemEventLoop, hal::prelude::Peripherals};
use log::info;
use ssd1306::{prelude::*, I2CDisplayInterface, Ssd1306};

fn main() -> Result<()> {
    // It is necessary to call this function once. Otherwise, some patches to
    // the runtime implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    log::info!("Hello, world!");

    let peripherals = Peripherals::take().unwrap();
    let _sysloop = EspSystemEventLoop::take()?;

    let i2c = I2cDriver::new(
        peripherals.i2c0,
        peripherals.pins.gpio9,
        peripherals.pins.gpio10,
        &I2cConfig::new().baudrate(100000_u32.Hz().into()),
    )?;

    // info!("Starting I2C scan...");

    // for addr in 0x03u8..=0x77u8 {
    //     // “敲门”方式：对这个地址做一次空写（写 0 字节）
    //     // 有 ACK 就返回 Ok
    //     let res =
    //         i2c.write(addr, &[], TickType_t::from(TickType::new_millis(50)));
    //     if res.is_ok() {
    //         info!("Found device at 0x{:02X}", addr);
    //     }
    // }

    info!("Scan done.");

    // SSD1306 I2C interface（地址扫描到的是 0x3C）
    let interface = I2CDisplayInterface::new_custom_address(i2c, 0x3C);

    // 0.91" 128x32；
    let mut display: Ssd1306<_, _, _> =
        Ssd1306::new(interface, DisplaySize128x32, DisplayRotation::Rotate0)
            .into_buffered_graphics_mode();

    display.init().map_err(|e| anyhow::anyhow!("{:?}", e))?;
    display.clear_buffer();

    let style = MonoTextStyle::new(&FONT_8X13, BinaryColor::On);
    Text::new("Hello OLED!", Point::new(0, 15), style)
        .draw(&mut display)
        .map_err(|e| anyhow::anyhow!("{:?}", e))?;
    Text::new("addr=0x3C", Point::new(0, 30), style)
        .draw(&mut display)
        .map_err(|e| anyhow::anyhow!("{:?}", e))?;

    // 关键：flush 才会真的显示
    display.flush().map_err(|e| anyhow::anyhow!("{:?}", e))?;

    info!("display updated");

    loop {
        info!("Running...");
        std::thread::sleep(std::time::Duration::from_secs(5));
    }
}
