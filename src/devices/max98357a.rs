use esp_idf_hal::{
    delay::TickType,
    i2s::{I2sDriver, I2sTx},
    sys::TickType_t,
};

pub struct Max98357a {
    i2s: I2sDriver<'static, I2sTx>,
}

impl Max98357a {
    pub fn new(i2s: I2sDriver<'static, I2sTx>) -> Self {
        Max98357a { i2s }
    }

    pub fn play_sample(
        &mut self,
        _frequency: f32,
        _duration_ms: u32,
    ) -> anyhow::Result<()> {
        let sample_rate_hz = 44_100_f32;
        let freq_hz = 440.0_f32; // A4
        let amp = 0.01_f32; // 0.0~1.0, 别太大避免爆音

        // 小 buffer：1024 个 sample -> 2048 bytes（i16）
        let mut buf = [0i16; 1024];

        let mut phase = 0.0_f32;
        let phase_step = 2.0 * std::f32::consts::PI * freq_hz / sample_rate_hz;

        loop {
            for s in buf.iter_mut() {
                let v = (phase.sin() * amp * i16::MAX as f32) as i16;
                *s = v;
                phase += phase_step;
                if phase > 2.0 * std::f32::consts::PI {
                    phase -= 2.0 * std::f32::consts::PI;
                }
            }

            // 把 i16 buffer 当作字节写入
            let bytes: &[u8] = unsafe {
                core::slice::from_raw_parts(
                    buf.as_ptr() as *const u8,
                    buf.len() * core::mem::size_of::<i16>(),
                )
            };

            self.i2s
                .write(bytes, TickType_t::from(TickType::new_millis(1000)))?;
        }
        Ok(())
    }
}
