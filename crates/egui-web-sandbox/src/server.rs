//! Serve an embedded Web build. The executable needs neither Cargo nor source files.
use std::{collections::BTreeMap, error::Error, io::Cursor, net::Ipv4Addr};
use tiny_http::{Header, Method, Response};

pub type Result<T> = std::result::Result<T, Box<dyn Error + Send + Sync>>;
pub type Assets = &'static [(&'static str, &'static [u8])];

/// A loopback-only server. Binding port 0 chooses an unused port for each instance.
pub struct Server {
    http: tiny_http::Server,
    assets: BTreeMap<&'static str, &'static [u8]>,
}

impl Server {
    /// Bind without opening a browser or starting background processes.
    /// `assets` comes from `include_app!()`; only embedded resources are reachable.
    pub fn bind(assets: Assets, port: u16) -> Result<Self> {
        let mut files = BTreeMap::from([
            (
                "index.html",
                include_bytes!("../browser/index.html").as_slice(),
            ),
            (
                "launch.mjs",
                include_bytes!("../browser/launch.mjs").as_slice(),
            ),
            (
                "client.mjs",
                include_bytes!("../browser/client.mjs").as_slice(),
            ),
            ("LICENSE.txt", include_bytes!("../LICENSE").as_slice()),
        ]);
        files.extend(assets.iter().copied());
        Ok(Self {
            http: tiny_http::Server::http((Ipv4Addr::LOCALHOST, port))?,
            assets: files,
        })
    }

    pub fn url(&self) -> String {
        format!("http://{}/", self.http.server_addr())
    }

    /// Serve on this thread until `stop()` or process exit. Use one request loop per server.
    pub fn run(&self) {
        for request in self.http.incoming_requests() {
            if !matches!(request.method(), Method::Get | Method::Head) {
                let _ = request.respond(Response::empty(405));
                continue;
            }
            let path = request.url().split('?').next().unwrap_or("/");
            let name = if path == "/" {
                "index.html"
            } else {
                path.trim_start_matches('/')
            };
            let Some(bytes) = self.assets.get(name) else {
                let _ = request.respond(Response::empty(404));
                continue;
            };
            let content_type = match name.rsplit('.').next() {
                Some("html") => "text/html; charset=utf-8",
                Some("js" | "mjs") => "text/javascript; charset=utf-8",
                Some("wasm") => "application/wasm",
                _ => "application/octet-stream",
            };
            let response = Response::new(
                200.into(),
                vec![
                    Header::from_bytes("Content-Type", content_type).unwrap(),
                    Header::from_bytes("Cache-Control", "no-store").unwrap(),
                ],
                Cursor::new(*bytes),
                Some(bytes.len()),
                None,
            );
            // A disconnected browser must not take down other test sessions.
            let _ = request.respond(response);
        }
    }

    /// Unblock the request loop. Drop the server after joining it to close the listener.
    pub fn stop(&self) {
        self.http.unblock();
    }
}

/// Bind an unused loopback port, print its URL and serve the embedded app.
pub fn serve(assets: Assets) -> Result<()> {
    let server = Server::bind(assets, 0)?;
    println!("{}", server.url());
    server.run();
    Ok(())
}
