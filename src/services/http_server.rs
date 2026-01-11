use anyhow::{Ok, Result};
use esp_idf_hal::io::Write;
use esp_idf_svc::http::server::{Configuration, EspHttpServer};
use log::info;

pub struct HTTPServer<'a> {
    pub server: EspHttpServer<'a>,
}

impl HTTPServer<'_> {
    pub fn new() -> Self {
        HTTPServer {
            server: EspHttpServer::new(&Configuration::default()).unwrap(),
        }
    }

    pub fn start(&mut self) -> Result<()> {
        self.server.fn_handler(
            "/",
            esp_idf_svc::http::server::Method::Get,
            |request| {
                let mut response = request.into_ok_response()?;
                response.write_all("Hello, World!".as_bytes())?;
                Ok(())
            },
        )?;

        info!("HTTP Server started on port 80");
        Ok(())
    }
}
