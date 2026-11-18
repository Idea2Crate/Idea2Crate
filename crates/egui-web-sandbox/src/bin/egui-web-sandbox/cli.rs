use std::{
    env,
    ffi::OsStr,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    path::PathBuf,
};

use super::Result;

pub const LISTEN_HELP: &str = "\
  -b, --bind ADDRESS  Listen on this IPv4 or IPv6 address [default: 127.0.0.1]
      --port PORT     Listen on this port; 0 selects an unused port [default: 0]
  -h, --help          Show this help";

pub struct Options {
    pub address: SocketAddr,
    pub help: bool,
    pub wasm: Option<PathBuf>,
}

impl Options {
    pub fn parse() -> Result<Self> {
        let mut args = env::args_os().skip(1);
        let mut ip = IpAddr::V4(Ipv4Addr::LOCALHOST);
        let mut port = None;
        let mut wasm = None;
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
                    } else if wasm.is_none()
                        && PathBuf::from(&arg)
                            .extension()
                            .is_some_and(|extension| extension == "wasm")
                    {
                        wasm = Some(arg.into());
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
            wasm,
        })
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
