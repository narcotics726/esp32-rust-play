use anyhow::{Ok, Result};
use esp_idf_hal::io::Write;
use esp_idf_svc::http::server::{Configuration, EspHttpServer, Method};
use log::info;

pub struct HTTPServer<'a> {
    pub server: EspHttpServer<'a>,
    snd: crossbeam_channel::Sender<String>,
    snd_bounded: crossbeam_channel::Sender<Vec<u8>>,
}

impl HTTPServer<'_> {
    pub fn new(
        snd: crossbeam_channel::Sender<String>,
        snd_bounded: crossbeam_channel::Sender<Vec<u8>>,
    ) -> Self {
        HTTPServer {
            server: EspHttpServer::new(&Configuration::default()).unwrap(),
            snd,
            snd_bounded,
        }
    }

    pub fn start(&mut self) -> Result<()> {
        self.server.fn_handler("/", Method::Get, |request| {
            let mut response = request.into_ok_response()?;
            response.write_all("Hello, World!".as_bytes())?;
            Ok(())
        })?;

        let snd = self.snd.clone();
        self.server.fn_handler("/str", Method::Post, move |mut request| {
            // read the body
            const MAX_CONTENT_LENGTH: usize = 4096;
            let mut buf = Vec::new();
            let mut chunk = [0u8; 512];
            loop {
                let n = request.read(&mut chunk)?;
                if n == 0 || buf.len() + n > MAX_CONTENT_LENGTH {
                    break;
                }
                buf.extend_from_slice(&chunk[..n]);
            }

            let body_str = String::from_utf8(buf).unwrap_or_default();
            if let Err(err) = snd.send(body_str) {
                log::error!("failed to send body: {err}");
            }
            let mut response = request.into_ok_response()?;
            response.write_all(b"Received")?;
            Ok(())
        })?;

        let snd_bounded = self.snd_bounded.clone();
        self.server.fn_handler(
            "/audio",
            Method::Post,
            move |mut request| {
                // read the body chunks by chunks
                // send them to the bounded channel
                const CHUNK_SIZE: usize = 4096;
                let mut chunk = vec![0u8; CHUNK_SIZE];
                loop {
                    let n = request.read(&mut chunk)?;
                    if n == 0 {
                        break;
                    }
                    let data = chunk[..n].to_vec();
                    if let Err(err) = snd_bounded.send(data) {
                        log::error!("failed to send audio chunk: {err}");
                        break;
                    }
                }
                Ok(())
            },
        )?;

        info!("HTTP Server started on port 80");
        Ok(())
    }
}
