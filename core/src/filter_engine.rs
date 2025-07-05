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

/// Type of filter rule
#[derive(Debug, Clone)]
enum FilterRule {
    /// Simple domain blocking (e.g., "doubleclick.net")
    Domain(String),
    /// Pattern with wildcards (e.g., "*/ads/*")
    Pattern(String),
    /// Subdomain pattern (e.g., "||domain.com^")
    SubdomainPattern(String),
}

/// Main filter engine for ad blocking
pub struct FilterEngine {
    /// Compiled filter rules
    rules: Vec<FilterRule>,
}

impl FilterEngine {
    /// Create a new filter engine with default ad-blocking rules
    pub fn new_with_defaults() -> Self {
        let rules = vec![
            FilterRule::Domain("doubleclick.net".to_string()),
            FilterRule::Domain("googleadservices.com".to_string()),
            FilterRule::Domain("googlesyndication.com".to_string()),
            FilterRule::Domain("facebook.com/tr".to_string()),
            FilterRule::Domain("amazon-adsystem.com".to_string()),
        ];
        
        FilterEngine { rules }
    }
    
    /// Create a new filter engine with custom patterns
    pub fn new_with_patterns(patterns: Vec<String>) -> Self {
        let rules = patterns.into_iter().map(|pattern| {
            if pattern.starts_with("||") && pattern.ends_with("^") {
                FilterRule::SubdomainPattern(pattern[2..pattern.len()-1].to_string())
            } else if pattern.contains('*') {
                FilterRule::Pattern(pattern)
            } else {
                FilterRule::Domain(pattern)
            }
        }).collect();
        
        FilterEngine { rules }
    }
    
    /// Check if a URL should be blocked
    pub fn should_block(&self, url: &str) -> BlockDecision {
        for rule in &self.rules {
            match rule {
                FilterRule::Domain(domain) => {
                    if url.contains(domain) {
                        return BlockDecision {
                            should_block: true,
                            reason: Some(format!("Matched ad domain: {}", domain)),
                        };
                    }
                }
                FilterRule::Pattern(pattern) => {
                    if self.matches_wildcard_pattern(url, pattern) {
                        return BlockDecision {
                            should_block: true,
                            reason: Some(format!("Matched pattern: {}", pattern)),
                        };
                    }
                }
                FilterRule::SubdomainPattern(domain) => {
                    if self.matches_subdomain(url, domain) {
                        return BlockDecision {
                            should_block: true,
                            reason: Some(format!("Matched subdomain: {}", domain)),
                        };
                    }
                }
            }
        }
        
        BlockDecision {
            should_block: false,
            reason: None,
        }
    }
    
    /// Check if URL matches a subdomain pattern
    fn matches_subdomain(&self, url: &str, domain: &str) -> bool {
        if let Some(start) = url.find("://") {
            let url_after_protocol = &url[start + 3..];
            let url_host = url_after_protocol.split('/').next().unwrap_or("");
            
            // Exact match or subdomain match
            url_host == domain || url_host.ends_with(&format!(".{}", domain))
        } else {
            false
        }
    }
    
    /// Check if URL matches a wildcard pattern
    fn matches_wildcard_pattern(&self, url: &str, pattern: &str) -> bool {
        let pattern_parts: Vec<&str> = pattern.split('*').collect();
        
        if pattern_parts.is_empty() {
            return true;
        }
        
        let mut current_pos = 0;
        
        for (i, part) in pattern_parts.iter().enumerate() {
            if part.is_empty() {
                // Skip empty parts (consecutive wildcards)
                continue;
            }
            
            if i == 0 && !pattern.starts_with('*') {
                // Pattern doesn't start with wildcard, must match from beginning
                if !url.starts_with(part) {
                    return false;
                }
                current_pos = part.len();
            } else if i == pattern_parts.len() - 1 && !pattern.ends_with('*') {
                // Pattern doesn't end with wildcard, must match at end
                if !url[current_pos..].ends_with(part) {
                    return false;
                }
            } else {
                // Find the part somewhere after current position
                if let Some(pos) = url[current_pos..].find(part) {
                    current_pos += pos + part.len();
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