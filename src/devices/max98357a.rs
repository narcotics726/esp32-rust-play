use std::sync::{atomic::AtomicBool, Arc};

use esp_idf_hal::{
    delay::TickType,
    i2s::{I2sDriver, I2sTx},
    sys::TickType_t,
};

pub struct Max98357a {
    i2s: I2sDriver<'static, I2sTx>,
    pub is_stop: Arc<AtomicBool>,
}

impl Max98357a {
    pub fn new(i2s: I2sDriver<'static, I2sTx>) -> Self {
        Max98357a { i2s, is_stop: Arc::new(AtomicBool::new(false)) }
    }

    pub fn play_sample(
        &mut self,
        frequency: f32,
        duration_ms: u32,
    ) -> anyhow::Result<()> {
        let sample_rate_hz = 44_100_f32;
        let amp = 0.01_f32;

        // 每次写 256 帧 stereo：每帧 L+R 两个 i16
        const FRAMES: usize = 256;
        let mut buf = [0i16; FRAMES * 2]; // LRLR...

        let mut phase = 0.0_f32;
        let phase_step =
            2.0 * std::f32::consts::PI * frequency / sample_rate_hz;

        // 总共要播放多少帧（按“帧”算：一帧包含 L+R）
        let total_frames = (sample_rate_hz as u32) * duration_ms / 1000;
        let mut remaining = total_frames as usize;

        while remaining > 0 {
            if self.is_stop.load(std::sync::atomic::Ordering::Relaxed) {
                // 写一块静音，而不是 sleep
                for i in 0..FRAMES {
                    buf[i * 2] = 0;
                    buf[i * 2 + 1] = 0;
                }
                let bytes: &[u8] = unsafe {
                    core::slice::from_raw_parts(
                        buf.as_ptr() as *const u8,
                        buf.len() * 2,
                    )
                };
                self.i2s.write(
                    bytes,
                    TickType_t::from(TickType::new_millis(1000)),
                )?;
                std::thread::sleep(std::time::Duration::from_millis(30));
                continue;
            }
            let frames_now = remaining.min(FRAMES);
            if remaining - frames_now <= 0 {
                remaining += total_frames as usize; // loop
            }

            for i in 0..frames_now {
                let v = (phase.sin() * amp * i16::MAX as f32) as i16;
                // 左右声道都输出同样的波形（最简单）
                buf[i * 2] = v; // L
                buf[i * 2 + 1] = v; // R

                phase += phase_step;
                if phase > 2.0 * std::f32::consts::PI {
                    phase -= 2.0 * std::f32::consts::PI;
                }
            }

            let bytes: &[u8] = unsafe {
                core::slice::from_raw_parts(
                    buf.as_ptr() as *const u8,
                    frames_now * 2 * core::mem::size_of::<i16>(),
                )
            };

            // 用 write_all（如果有）更符合“把这块写完”
            self.i2s
                .write(bytes, TickType_t::from(TickType::new_millis(1000)))?;

            remaining -= frames_now;
            // 一般不需要 sleep：write 本身会因为 DMA 缓冲而阻塞到合适速率
            // 如果你发现 CPU 占用太高，可以 sleep 1ms
            // std::thread::sleep(std::time::Duration::from_millis(1));
        }

        Ok(())
    }
}
