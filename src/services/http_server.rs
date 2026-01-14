use anyhow::{Ok, Result};
use esp_idf_hal::io::Write;
use esp_idf_svc::http::server::{Configuration, EspHttpServer, Method};
use log::info;

pub struct HTTPServer<'a> {
    pub server: EspHttpServer<'a>,
    snd: crossbeam_channel::Sender<String>,
}

impl HTTPServer<'_> {
    pub fn new(snd: crossbeam_channel::Sender<String>) -> Self {
        HTTPServer {
            server: EspHttpServer::new(&Configuration::default()).unwrap(),
            snd,
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

        info!("HTTP Server started on port 80");
        Ok(())
    }
}
