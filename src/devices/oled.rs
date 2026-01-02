use embedded_graphics::{
    mono_font::{
        ascii::FONT_8X13,
        MonoTextStyle,
    },
    pixelcolor::BinaryColor,
    prelude::*,
    text::Text,
};
use ssd1306::{
    mode::BufferedGraphicsMode, prelude::I2CInterface, size::DisplaySize128x32,
    Ssd1306,
};

type MyOLED = Ssd1306<
    I2CInterface<esp_idf_hal::i2c::I2cDriver<'static>>,
    DisplaySize128x32,
    BufferedGraphicsMode<DisplaySize128x32>,
>;

pub struct Oled {
    display: MyOLED,
}

impl Oled {
    pub fn new(hw: MyOLED) -> Self {
        Oled { display: hw }
    }

    pub fn show(&mut self, lines: (&str, &str)) -> anyhow::Result<()> {
        self.display.clear_buffer();
        let style = MonoTextStyle::new(&FONT_8X13, BinaryColor::On);
        Text::new(lines.0, Point::new(0, 15), style)
            .draw(&mut self.display)
            .map_err(|e| anyhow::anyhow!("{:?}", e))?;
        Text::new(lines.1, Point::new(0, 30), style)
            .draw(&mut self.display)
            .map_err(|e| anyhow::anyhow!("{:?}", e))?;
        self.display.flush().map_err(|e| anyhow::anyhow!("{:?}", e))?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.display.clear_buffer();
        let _ = self.display.flush();
    }
}
