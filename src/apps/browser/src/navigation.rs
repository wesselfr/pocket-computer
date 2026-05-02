use heapless::String;

const MAX_LOCAL_PATH_LEN: usize = 64;

pub enum Resource<'a> {
    Local(&'a str),
    Remote {
        host: &'a str,
        port: u16,
        path: &'a str,
    },
}

pub fn resolve_href(href: &str) -> Resource<'_> {
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

        Resource::Remote { host, port, path }
    } else {
        Resource::Local(href)
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
