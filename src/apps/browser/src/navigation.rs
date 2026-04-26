use heapless::String;

const MAX_LOCAL_PATH_LEN: usize = 64;

pub enum Resource<'a> {
    Local(&'a str),
    Remote { host: &'a str, path: &'a str },
}

pub fn resolve_href(href: &str) -> Resource<'_> {
    if let Some(rest) = href.strip_prefix("http://") {
        if let Some((host, path)) = rest.split_once('/') {
            Resource::Remote { host, path }
        } else {
            Resource::Remote {
                host: rest,
                path: "/",
            }
        }
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
