use anyhow::Result;
use esp_idf_hal::{
    delay::TickType,
    gpio::{
        AnyIOPin, Gpio10, Gpio11, Gpio12, Gpio4, Gpio5, Gpio6, Gpio7, InputPin,
        OutputPin, PinDriver, Pull,
    },
    i2c::{I2cConfig, I2cDriver, I2C0},
    i2s::{
        config::{DataBitWidth, StdConfig},
        I2sDriver,
    },
    sys::TickType_t,
    units::FromValueType,
};
use log::info;

use crate::devices::{
    button::Button, max98357a::Max98357a, oled::Oled, wifi::Wifi,
};
use esp_idf_svc::{eventloop::EspSystemEventLoop, hal::prelude::Peripherals};
use ssd1306::{prelude::*, I2CDisplayInterface, Ssd1306};

pub struct Board {
    i2c: Option<I2cDriver<'static>>,
    pub oled: Option<Oled>,
    pub max98357a: Option<Max98357a>,
    pub btn1: Option<Button<Gpio6>>,
    pub btn2: Option<Button<Gpio7>>,
    pub wifi: Option<Wifi>,
}

impl Board {
    pub fn new() -> Self {
        Board {
            oled: None,
            i2c: None,
            max98357a: None,
            btn1: None,
            btn2: None,
            wifi: None,
        }
    }

    fn init_wifi(
        modem: esp_idf_hal::modem::Modem,
        sysloop: EspSystemEventLoop,
        wifi_slot: &mut Option<Wifi>,
    ) {
        *wifi_slot = Some(Wifi::new(modem, sysloop));
    }

    pub fn init(
        &mut self,
        peripherals: Peripherals,
        sysloop: EspSystemEventLoop,
    ) -> Result<()> {
        let modem = peripherals.modem;

        let i2c0 = peripherals.i2c0;
        let gpio4 = peripherals.pins.gpio4;
        let gpio5 = peripherals.pins.gpio5;

        self.init_i2c(i2c0, gpio4, gpio5)?;
        self.init_oled()?;

        let i2s0 = peripherals.i2s0;
        let gpio10 = peripherals.pins.gpio10;
        let gpio11 = peripherals.pins.gpio11;
        let gpio12 = peripherals.pins.gpio12;
        self.init_max98357a(i2s0, gpio10, gpio11, gpio12)?;

        let Board { btn1, btn2, wifi, .. } = self;

        Self::init_wifi(modem, sysloop.clone(), wifi);

        let gpio6 = peripherals.pins.gpio6;
        Self::init_button(gpio6, btn1)?;

        let gpio7 = peripherals.pins.gpio7;
        Self::init_button(gpio7, btn2)?;
        Ok(())
    }

    #[allow(dead_code)]
    fn i2c_addr_scan(&mut self) -> Result<()> {
        info!("Starting I2C scan...");

        for addr in 0x03u8..=0x77u8 {
            let res = self.i2c.take().unwrap().write(
                addr,
                &[],
                TickType_t::from(TickType::new_millis(50)),
            );
            if res.is_ok() {
                info!("Found device at 0x{:02X}", addr);
            }
        }

        Ok(())
    }

    fn init_i2c(
        &mut self,
        i2c0: I2C0,
        gpio4: Gpio4,
        gpio5: Gpio5,
    ) -> Result<()> {
        if self.i2c.is_none() {
            self.i2c = Some(I2cDriver::new(
                i2c0,
                gpio5,
                gpio4,
                &I2cConfig::new().baudrate(100_u32.kHz().into()),
            )?);
        }
        Ok(())
    }

    fn init_oled(&mut self) -> Result<()> {
        if self.oled.is_none() {
            let i2c = self.i2c.take().unwrap();

            let interface = I2CDisplayInterface::new_custom_address(i2c, 0x3C);
            let mut display = Ssd1306::new(
                interface,
                DisplaySize128x32,
                DisplayRotation::Rotate0,
            )
            .into_buffered_graphics_mode();

            display.init().map_err(|e| anyhow::anyhow!("{:?}", e))?;
            self.oled = Some(Oled::new(display));
        }
        Ok(())
    }

    fn init_max98357a(
        &mut self,
        i2s0: esp_idf_hal::i2s::I2S0,
        gpio10: Gpio10,
        gpio11: Gpio11,
        gpio12: Gpio12,
    ) -> Result<()> {
        if self.max98357a.is_none() {
            // 1) 选择采样率/位深（先用最常见的 44.1k / 16bit）
            let std_cfg = StdConfig::philips(44_100, DataBitWidth::Bits16); // stereo, MCLK=256*fs 的默认配置
                                                                            // 你也可以先用 22_050/16bit，降低压力
            let bclk = gpio11;
            let ws = gpio10; // LRC/WS
            let dout = gpio12; // MAX98357 的 DIN ← 这里是 ESP 的 DOUT
            let mclk: Option<AnyIOPin> = None; // MAX98357 通常不需要 MCLK

            let mut i2s =
                I2sDriver::new_std_tx(i2s0, &std_cfg, bclk, dout, mclk, ws)?;

            // 4) 启用 TX（开始产生 BCLK/WS）
            i2s.tx_enable()?;
            self.max98357a = Some(Max98357a::new(i2s));
        }
        Ok(())
    }

    fn init_button<P>(gpio: P, btn_slot: &mut Option<Button<P>>) -> Result<()>
    where
        P: InputPin + OutputPin,
    {
        let mut pin = PinDriver::input(gpio)?;
        pin.set_pull(Pull::Up)?;
        if btn_slot.is_none() {
            *btn_slot = Some(Button::new(pin));
        }
        Ok(())
    }
}
