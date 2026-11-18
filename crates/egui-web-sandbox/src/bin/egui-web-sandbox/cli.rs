use std::{
    ffi::{OsStr, OsString},
    net::{IpAddr, Ipv4Addr, SocketAddr},
    path::PathBuf,
};

use super::{PACK_USAGE, Result};

const BUNDLE_EXTENSION: &str = "egsx";

pub const LISTEN_HELP: &str = "\
  -b, --bind ADDRESS  Listen on this IPv4 or IPv6 address [default: 127.0.0.1]
      --port PORT     Listen on this port; 0 selects an unused port [default: 0]
  -h, --help          Show this help";

pub struct Options {
    pub address: SocketAddr,
    pub help: bool,
    pub bundle: Option<PathBuf>,
}

impl Options {
    pub fn parse(mut args: impl Iterator<Item = OsString>) -> Result<Self> {
        let mut ip = IpAddr::V4(Ipv4Addr::LOCALHOST);
        let mut port = None;
        let mut bundle = None;
        let mut help = false;
        while let Some(arg) = args.next() {
            let (flag, value) = match arg.to_str() {
                Some(arg) if arg.starts_with('-') => arg
                    .split_once('=')
                    .map_or((arg, None), |(flag, value)| (flag, Some(value.into()))),
                _ => {
                    if arg.to_str().is_some_and(|value| {
                        !value.is_empty() && value.bytes().all(|byte| byte.is_ascii_digit())
                    }) {
                        set_port(&mut port, &arg)?;
                    } else if bundle.is_none()
                        && PathBuf::from(&arg)
                            .extension()
                            .is_some_and(|extension| extension == BUNDLE_EXTENSION)
                    {
                        bundle = Some(arg.into());
                    } else {
                        return Err(format!("unexpected argument {arg:?}").into());
                    }
                    continue;
                }
            };
            match flag {
                "--help" | "-h" if value.is_none() => help = true,
                "--bind" | "-b" | "--port" => {
                    let value = value
                        .or_else(|| args.next())
                        .ok_or_else(|| format!("{flag} requires a value"))?;
                    if flag == "--port" {
                        set_port(&mut port, &value)?;
                    } else {
                        ip = match value.to_str() {
                            Some("localhost") => Ipv4Addr::LOCALHOST.into(),
                            Some(value) => value.parse().map_err(|_| {
                                format!(
                                    "invalid bind address {value:?}; use an IPv4 or IPv6 address"
                                )
                            })?,
                            None => {
                                return Err("bind address must be an IPv4 or IPv6 address".into());
                            }
                        };
                    }
                }
                _ => return Err(format!("unknown option {arg:?}").into()),
            }
        }
        Ok(Self {
            address: SocketAddr::new(ip, port.unwrap_or(0)),
            help,
            bundle,
        })
    }
}

pub struct PackOptions {
    pub entry: PathBuf,
    pub output: PathBuf,
    pub app_version: Option<String>,
    pub wasm_bindgen_version: Option<String>,
}

impl PackOptions {
    pub fn parse(mut args: impl Iterator<Item = OsString>) -> Result<Self> {
        let entry = PathBuf::from(args.next().ok_or(PACK_USAGE)?);
        let output = PathBuf::from(args.next().ok_or(PACK_USAGE)?);
        if entry.extension() != Some(OsStr::new("js"))
            || output.extension() != Some(OsStr::new(BUNDLE_EXTENSION))
        {
            return Err(PACK_USAGE.into());
        }
        let mut options = Self {
            entry,
            output,
            app_version: None,
            wasm_bindgen_version: None,
        };
        while let Some(flag) = args.next() {
            let version = match flag.to_str() {
                Some("--app-version") => &mut options.app_version,
                Some("--wasm-bindgen-version") => &mut options.wasm_bindgen_version,
                _ => return Err(format!("unknown pack option {flag:?}").into()),
            };
            if version.is_some() {
                return Err(format!("specify {flag:?} only once").into());
            }
            *version = Some(
                args.next()
                    .and_then(|value| value.into_string().ok())
                    .filter(|value| !value.is_empty() && !value.starts_with('-'))
                    .ok_or_else(|| format!("{flag:?} requires a version"))?,
            );
        }
        Ok(options)
    }
}

fn set_port(port: &mut Option<u16>, value: &OsStr) -> Result<()> {
    if port.is_some() {
        return Err("specify the port only once".into());
    }
    *port = Some(
        value
            .to_str()
            .and_then(|value| value.parse().ok())
            .ok_or_else(|| format!("invalid port {value:?}; expected 0–65535"))?,
    );
    Ok(())
}
