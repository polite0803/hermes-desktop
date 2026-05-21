use url::Url;

/// Allowed protocols for external URL opening
const ALLOWED_EXTERNAL_SCHEMES: &[&str] = &["https", "http", "mailto"];

/// Check if a URL can be safely opened via shell.openExternal
pub fn is_allowed_external_url(url_str: &str) -> bool {
    match Url::parse(url_str) {
        Ok(parsed) => ALLOWED_EXTERNAL_SCHEMES.contains(&parsed.scheme()),
        Err(_) => false,
    }
}

/// Check if a URL is allowed for webview navigation (localhost only)
pub fn is_allowed_webview_url(url_str: &str) -> bool {
    match Url::parse(url_str) {
        Ok(parsed) => {
            let host = match parsed.host() {
                Some(h) => h,
                None => return false,
            };
            let is_loopback = match &host {
                url::Host::Domain(d) => *d == "localhost",
                url::Host::Ipv4(ip) => ip.is_loopback(),
                url::Host::Ipv6(ip) => ip.is_loopback(),
            };
            if !is_loopback {
                return false;
            }
            // Port must be in 1024-65535 range
            match parsed.port() {
                Some(port) => port >= 1024,
                None => false,
            }
        }
        Err(_) => false,
    }
}

/// Check if a URL is allowed for in-app navigation
pub fn is_allowed_app_navigation(url_str: &str) -> bool {
    match Url::parse(url_str) {
        Ok(parsed) => {
            let scheme = parsed.scheme();
            if scheme == "file" {
                return true;
            }
            if scheme == "http" || scheme == "https" {
                // Allow localhost dev server
                match parsed.host() {
                    Some(host) => match &host {
                        url::Host::Domain(d) => *d == "localhost",
                        url::Host::Ipv4(ip) => ip.is_loopback(),
                        url::Host::Ipv6(ip) => ip.is_loopback(),
                    },
                    None => false,
                }
            } else {
                false
            }
        }
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_allowed_external_urls() {
        assert!(is_allowed_external_url("https://example.com"));
        assert!(is_allowed_external_url("http://example.com"));
        assert!(is_allowed_external_url("mailto:test@example.com"));
        assert!(!is_allowed_external_url("file:///etc/passwd"));
        assert!(!is_allowed_external_url("javascript:alert(1)"));
        assert!(!is_allowed_external_url("ftp://example.com"));
    }

    #[test]
    fn test_allowed_webview_url() {
        assert!(is_allowed_webview_url("http://localhost:8642"));
        assert!(is_allowed_webview_url("http://127.0.0.1:8080"));
        assert!(!is_allowed_webview_url("http://evil.com:8080"));
        assert!(!is_allowed_webview_url("http://localhost:80")); // port too low
    }

    #[test]
    fn test_allowed_app_navigation() {
        assert!(is_allowed_app_navigation("file:///some/path"));
        assert!(is_allowed_app_navigation("http://localhost:1420"));
        assert!(!is_allowed_app_navigation("https://evil.com"));
    }
}
