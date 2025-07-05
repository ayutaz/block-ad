//! Filter Engine - Core ad blocking logic
//! 
//! TDD Implementation - Starting with minimal code to pass tests

/// Result of a block decision
#[derive(Debug, Clone, PartialEq)]
pub struct BlockDecision {
    /// Whether the request should be blocked
    pub should_block: bool,
    /// Optional reason for the decision
    pub reason: Option<String>,
}

/// Main filter engine for ad blocking
pub struct FilterEngine {
    // For now, we'll use a simple list of blocked domains
    blocked_domains: Vec<String>,
    // Patterns that support wildcards
    patterns: Vec<String>,
}

impl FilterEngine {
    /// Create a new filter engine with default ad-blocking rules
    pub fn new_with_defaults() -> Self {
        FilterEngine {
            blocked_domains: vec![
                "doubleclick.net".to_string(),
                "googleadservices.com".to_string(),
                "googlesyndication.com".to_string(),
                "facebook.com/tr".to_string(),
                "amazon-adsystem.com".to_string(),
            ],
            patterns: vec![],
        }
    }
    
    /// Create a new filter engine with custom patterns
    pub fn new_with_patterns(patterns: Vec<String>) -> Self {
        FilterEngine {
            blocked_domains: vec![],
            patterns,
        }
    }
    
    /// Check if a URL should be blocked
    pub fn should_block(&self, url: &str) -> BlockDecision {
        // Simple implementation: check if URL contains any blocked domain
        for domain in &self.blocked_domains {
            if url.contains(domain) {
                return BlockDecision {
                    should_block: true,
                    reason: Some(format!("Matched ad domain: {}", domain)),
                };
            }
        }
        
        // Check patterns
        for pattern in &self.patterns {
            if self.matches_pattern(url, pattern) {
                return BlockDecision {
                    should_block: true,
                    reason: Some(format!("Matched pattern: {}", pattern)),
                };
            }
        }
        
        BlockDecision {
            should_block: false,
            reason: None,
        }
    }
    
    /// Check if a URL matches a pattern with wildcards
    fn matches_pattern(&self, url: &str, pattern: &str) -> bool {
        // Handle subdomain patterns like ||domain.com^
        if pattern.starts_with("||") && pattern.ends_with("^") {
            let domain = &pattern[2..pattern.len()-1];
            // Check if URL contains the domain
            if let Some(start) = url.find("://") {
                let url_after_protocol = &url[start+3..];
                return url_after_protocol.starts_with(domain) || 
                       url_after_protocol.contains(&format!(".{}", domain));
            }
        }
        
        // Convert wildcard pattern to regex-like matching
        let mut pattern_parts = vec![];
        let mut current = String::new();
        
        for ch in pattern.chars() {
            if ch == '*' {
                if !current.is_empty() {
                    pattern_parts.push(current.clone());
                    current.clear();
                }
                pattern_parts.push("*".to_string());
            } else {
                current.push(ch);
            }
        }
        if !current.is_empty() {
            pattern_parts.push(current);
        }
        
        // Simple wildcard matching
        let mut url_pos = 0;
        for (i, part) in pattern_parts.iter().enumerate() {
            if part == "*" {
                // Wildcard - skip to next part if exists
                if i + 1 < pattern_parts.len() {
                    if let Some(pos) = url[url_pos..].find(&pattern_parts[i + 1]) {
                        url_pos += pos;
                    } else {
                        return false;
                    }
                }
            } else {
                // Literal match
                if url[url_pos..].starts_with(part) {
                    url_pos += part.len();
                } else {
                    return false;
                }
            }
        }
        
        true
    }
    
    /// Create a new filter engine from configuration
    pub fn new(_config: &crate::Config) -> Result<Self, Box<dyn std::error::Error>> {
        // TODO: Load rules based on config
        // For now, just return default
        Ok(Self::new_with_defaults())
    }
}