use crate::{Result, bundle::SITE_INDEX};
use percent_encoding::{AsciiSet, NON_ALPHANUMERIC, percent_decode_str, utf8_percent_encode};
use std::{
    borrow::Cow,
    collections::BTreeMap,
    io::Cursor,
    net::{Ipv4Addr, Ipv6Addr, SocketAddr},
};
use tiny_http::{Header, Method, Response, Server};

const URL_COMPONENT: &AsciiSet = &NON_ALPHANUMERIC
    .remove(b'_')
    .remove(b'.')
    .remove(b'-')
    .remove(b'~');

pub fn serve(
    module: &str,
    generated: BTreeMap<String, Vec<u8>>,
    address: SocketAddr,
) -> Result<()> {
    let files = [
        (
            "index.html",
            include_bytes!("../../../browser/index.html").as_slice(),
        ),
        (
            "desktop.html",
            include_bytes!("../../../browser/desktop.html").as_slice(),
        ),
        (
            "desktop.mjs",
            include_bytes!("../../../browser/desktop.mjs").as_slice(),
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
    let entry_location = Header::from_bytes(
        "Location",
        format!("/{}", utf8_percent_encode(module, URL_COMPONENT)),
    )
    .unwrap();
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
        let (path, query) = request.url().split_once('?').unwrap_or((request.url(), ""));
        let Ok(path) = percent_decode_str(path).decode_utf8() else {
            let _ = request.respond(Response::empty(404));
            continue;
        };
        let name = if path == "/" {
            "index.html"
        } else {
            path.trim_start_matches('/')
        };
        if module == SITE_INDEX && name == "index.html" {
            let location = if query.is_empty() {
                format!("/{SITE_INDEX}")
            } else {
                format!("/{SITE_INDEX}?{query}")
            };
            let response = match Header::from_bytes("Location", location) {
                Ok(location) => Response::empty(302).with_header(location),
                Err(()) => Response::empty(400),
            };
            let _ = request.respond(response);
            continue;
        }
        if module != SITE_INDEX && name == "app.js" && module != name {
            let _ = request.respond(Response::empty(302).with_header(entry_location.clone()));
            continue;
        }
        let Some(bytes) = assets.get(name) else {
            let _ = request.respond(Response::empty(404));
            continue;
        };
        let content_type = match name.rsplit('.').next() {
            Some("html") => "text/html; charset=utf-8",
            Some("js" | "mjs") => "text/javascript; charset=utf-8",
            Some("css") => "text/css; charset=utf-8",
            Some("wasm") => "application/wasm",
            Some("json") => "application/json",
            Some("webp") => "image/webp",
            Some("png") => "image/png",
            Some("svg") => "image/svg+xml",
            Some("jpg" | "jpeg") => "image/jpeg",
            Some("gif") => "image/gif",
            Some("ico") => "image/x-icon",
            Some("ttf") => "font/ttf",
            Some("otf") => "font/otf",
            Some("woff") => "font/woff",
            Some("woff2") => "font/woff2",
            Some("txt") => "text/plain; charset=utf-8",
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
