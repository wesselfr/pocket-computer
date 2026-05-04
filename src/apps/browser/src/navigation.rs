use core::str::FromStr;

use heapless::String;

const MAX_LOCAL_PATH_LEN: usize = 64;
const MAX_HOST_LEN: usize = 512;

#[derive(Clone)]
pub enum Resource {
    Local(String<MAX_LOCAL_PATH_LEN>),
    Remote {
        host: String<MAX_HOST_LEN>,
        port: u16,
        path: String<MAX_LOCAL_PATH_LEN>,
    },
}

pub fn resolve_href_from(href: &str, current: &Resource) -> Resource {
    match resolve_href(href) {
        Resource::Remote { host, port, path } => Resource::Remote { host, port, path },
        Resource::Local(path) => match current {
            Resource::Remote { host, port, .. } => Resource::Remote {
                host: host.clone(),
                port: *port,
                path,
            },
            Resource::Local(_) => Resource::Local(path),
        },
    }
}

pub fn resolve_href(href: &str) -> Resource {
    if let Some(rest) = href.strip_prefix("http://") {
        let (host_port, path) = if let Some(idx) = rest.find('/') {
            (&rest[..idx], &rest[idx..])
        } else {
            (rest, "/")
        };

        let (host, port) = if let Some((host, port_str)) = host_port.split_once(':') {
            let port = port_str.parse::<u16>().unwrap_or(80);
            (host, port)
        } else {
            (host_port, 80)
        };

        Resource::Remote {
            host: String::from_str(host).unwrap_or_default(),
            port,
            path: String::from_str(path).unwrap_or_default(),
        }
    } else {
        Resource::Local(String::from_str(href).unwrap_or_default())
    }
}

pub fn resolve_local_path(path: &str) -> String<MAX_LOCAL_PATH_LEN> {
    let mut out = String::<MAX_LOCAL_PATH_LEN>::new();

    let path = path.trim();
    let path = path.strip_prefix('/').unwrap_or(path);

    if path.is_empty() {
        out.push_str("index.html").ok();
        return out;
    }

    if path.contains('.') {
        out.push_str(path).ok();
    } else {
        out.push_str(path).ok();
        out.push_str(".html").ok();
    }

    out
}
