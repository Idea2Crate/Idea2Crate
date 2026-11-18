//! Serve an embedded Web build. The executable needs neither Cargo nor source files.
use std::{
    borrow::Cow,
    collections::BTreeMap,
    error::Error,
    io::Cursor,
    net::{Ipv4Addr, Ipv6Addr, SocketAddr},
};
use tiny_http::{Header, Method, Response};

#[doc(hidden)]
pub mod cli;

pub type Result<T> = std::result::Result<T, Box<dyn Error + Send + Sync>>;
pub type Assets = &'static [(&'static str, &'static [u8])];

/// A Web resource server. Binding port 0 chooses an unused port for each instance.
pub struct Server {
    http: tiny_http::Server,
    assets: BTreeMap<Cow<'static, str>, Cow<'static, [u8]>>,
}

impl Server {
    /// Bind with resources embedded by `include_app!()`.
    pub fn bind(assets: Assets, port: u16) -> Result<Self> {
        Self::bind_assets(assets.iter().copied(), port)
    }

    /// Bind with a snapshot of the generated resources. Only supplied resources
    /// and the standard page are served.
    pub fn bind_assets(
        assets: impl IntoIterator<Item = (impl Into<Cow<'static, str>>, impl Into<Cow<'static, [u8]>>)>,
        port: u16,
    ) -> Result<Self> {
        Self::bind_assets_at(assets, SocketAddr::from((Ipv4Addr::LOCALHOST, port)))
    }

    /// Bind generated resources to an explicit address and port.
    pub fn bind_assets_at(
        assets: impl IntoIterator<Item = (impl Into<Cow<'static, str>>, impl Into<Cow<'static, [u8]>>)>,
        address: SocketAddr,
    ) -> Result<Self> {
        let files = [
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
        ];
        Ok(Self {
            http: tiny_http::Server::http(address)?,
            assets: files
                .into_iter()
                .map(|(name, bytes)| (Cow::Borrowed(name), Cow::Borrowed(bytes)))
                .chain(
                    assets
                        .into_iter()
                        .map(|(name, bytes)| (name.into(), bytes.into())),
                )
                .collect(),
        })
    }

    pub fn url(&self) -> String {
        let mut address = self
            .http
            .server_addr()
            .to_ip()
            .expect("bound to an IP address");
        if address.ip().is_unspecified() {
            address.set_ip(match address {
                SocketAddr::V4(_) => Ipv4Addr::LOCALHOST.into(),
                SocketAddr::V6(_) => Ipv6Addr::LOCALHOST.into(),
            });
        }
        format!("http://{address}/")
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
                Cursor::new(bytes.as_ref()),
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

/// Serve the embedded app with `[PORT]`, `--port PORT` and `--bind ADDRESS`
/// from the executable's command line. Use this as the entry point of a Web demo.
pub fn serve_from_args(assets: Assets) -> Result<()> {
    let options = cli::Options::parse(false)?;
    if options.help {
        println!("Usage: <program> [PORT] [OPTIONS]\n\n{}", cli::LISTEN_HELP);
        return Ok(());
    }
    let server = Server::bind_assets_at(assets.iter().copied(), options.address)?;
    println!("{}", server.url());
    server.run();
    Ok(())
}
