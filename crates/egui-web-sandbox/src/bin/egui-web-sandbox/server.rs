use crate::Result;
use std::{
    borrow::Cow,
    collections::BTreeMap,
    io::Cursor,
    net::{Ipv4Addr, Ipv6Addr, SocketAddr},
};
use tiny_http::{Header, Method, Response, Server};

pub fn serve(generated: Vec<(String, Vec<u8>)>, address: SocketAddr) -> Result<()> {
    let files = [
        (
            "index.html",
            include_bytes!("../../../browser/index.html").as_slice(),
        ),
        (
            "launch.mjs",
            include_bytes!("../../../browser/launch.mjs").as_slice(),
        ),
        (
            "client.mjs",
            include_bytes!("../../../browser/client.mjs").as_slice(),
        ),
        ("LICENSE.txt", include_bytes!("../../../LICENSE").as_slice()),
    ];
    let assets: BTreeMap<_, _> = files
        .into_iter()
        .map(|(name, bytes)| (Cow::Borrowed(name), Cow::Borrowed(bytes)))
        .chain(
            generated
                .into_iter()
                .map(|(name, bytes)| (Cow::Owned(name), Cow::Owned(bytes))),
        )
        .collect();
    let server = Server::http(address)?;
    let mut address = server
        .server_addr()
        .to_ip()
        .expect("bound to an IP address");
    if address.ip().is_unspecified() {
        address.set_ip(match address {
            SocketAddr::V4(_) => Ipv4Addr::LOCALHOST.into(),
            SocketAddr::V6(_) => Ipv6Addr::LOCALHOST.into(),
        });
    }
    println!("http://{address}/");

    for request in server.incoming_requests() {
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
        let Some(bytes) = assets.get(name) else {
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
    Ok(())
}
