use anyhow::Result;
use esp_idf_hal::{
    gpio::{AnyIOPin, Gpio10, Gpio5, Gpio6, Gpio7, Gpio9},
    i2c::{I2cConfig, I2cDriver, I2C0},
    i2s::{
        config::{DataBitWidth, StdConfig}, I2sDriver,
    },
    units::FromValueType,
};

use crate::devices::{max98357a::Max98357a, oled::Oled};
use esp_idf_svc::hal::prelude::Peripherals;
use ssd1306::{prelude::*, I2CDisplayInterface, Ssd1306};

pub struct Board {
    i2c: Option<I2cDriver<'static>>,
    pub oled: Option<Oled>,
    pub max98357a: Option<Max98357a>,
}

impl Board {
    pub fn new() -> Self {
        Board { oled: None, i2c: None, max98357a: None }
    }

    pub fn init(&mut self, peripherals: Peripherals) -> Result<()> {
        let i2c0 = peripherals.i2c0;
        let gpio9 = peripherals.pins.gpio9;
        let gpio10 = peripherals.pins.gpio10;

        self.init_i2c(i2c0, gpio9, gpio10)?;
        self.init_oled()?;

        let i2s0 = peripherals.i2s0;
        let gpio5 = peripherals.pins.gpio5;
        let gpio6 = peripherals.pins.gpio6;
        let gpio7 = peripherals.pins.gpio7;
        self.init_max98357a(i2s0, gpio5, gpio6, gpio7)?;
        Ok(())
    }

    fn init_i2c(
        &mut self,
        i2c0: I2C0,
        gpio9: Gpio9,
        gpio10: Gpio10,
    ) -> Result<()> {
        if self.i2c.is_none() {
            self.i2c = Some(I2cDriver::new(
                i2c0,
                gpio9,
                gpio10,
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
        gpio5: Gpio5,
        gpio6: Gpio6,
        gpio7: Gpio7,
    ) -> Result<()> {
        if self.max98357a.is_none() {
            // 1) 选择采样率/位深（先用最常见的 44.1k / 16bit）
            let std_cfg = StdConfig::philips(44_100, DataBitWidth::Bits16); // stereo, MCLK=256*fs 的默认配置
                                                                            // 你也可以先用 22_050/16bit，降低压力
            let bclk = gpio5;
            let ws = gpio6; // LRC/WS
            let dout = gpio7; // MAX98357 的 DIN ← 这里是 ESP 的 DOUT
            let mclk: Option<AnyIOPin> = None; // MAX98357 通常不需要 MCLK

            let mut i2s =
                I2sDriver::new_std_tx(i2s0, &std_cfg, bclk, dout, mclk, ws)?;

            // 4) 启用 TX（开始产生 BCLK/WS）
            i2s.tx_enable()?;
            self.max98357a = Some(Max98357a::new(i2s));
        }
        Ok(())
    }
}
