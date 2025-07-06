//! Utility functions for the ad blocker

/// Extract domain from a URL
///
/// # Examples
/// ```
/// use adblock_core::utils::extract_domain;
///
/// assert_eq!(extract_domain("https://example.com/path"), "example.com");
/// assert_eq!(extract_domain("http://sub.example.com:8080/"), "sub.example.com:8080");
/// ```
pub fn extract_domain(url: &str) -> String {
    // Find protocol separator
    if let Some(protocol_end) = url.find("://") {
        let after_protocol = &url[protocol_end + 3..];

        // Find the end of domain (path separator or end of string)
        if let Some(path_start) = after_protocol.find('/') {
            after_protocol[..path_start].to_string()
        } else {
            after_protocol.to_string()
        }
    } else {
        // No protocol, assume the whole string is domain
        if let Some(path_start) = url.find('/') {
            url[..path_start].to_string()
        } else {
            url.to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_domain() {
        assert_eq!(extract_domain("https://example.com/path"), "example.com");
        assert_eq!(
            extract_domain("http://sub.example.com:8080/"),
            "sub.example.com:8080"
        );
        assert_eq!(extract_domain("https://example.com"), "example.com");
        assert_eq!(extract_domain("example.com/path"), "example.com");
        assert_eq!(extract_domain("example.com"), "example.com");
    }
}
