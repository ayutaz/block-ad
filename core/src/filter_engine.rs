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
    /// Exception rule (e.g., "@@||example.com/ads/acceptable")
    Exception(String),
}

/// Main filter engine for ad blocking
pub struct FilterEngine {
    /// Compiled filter rules
    rules: Vec<FilterRule>,
}

impl FilterEngine {
    /// Create a filter engine from a filter list string
    pub fn from_filter_list(filter_list: &str) -> Result<Self, Box<dyn std::error::Error>> {
        use crate::filter_list::FilterListLoader;
        
        let loader = FilterListLoader::new();
        let raw_rules = loader.parse_filter_list(filter_list)?;
        
        let mut rules = Vec::new();
        
        for raw_rule in raw_rules {
            // Parse exception rules (@@)
            if raw_rule.starts_with("@@") {
                let exception_pattern = &raw_rule[2..];
                rules.push(FilterRule::Exception(exception_pattern.to_string()));
                continue;
            }
            
            // Parse subdomain patterns
            else if raw_rule.starts_with("||") && raw_rule.ends_with("^") {
                let domain = raw_rule[2..raw_rule.len()-1].to_string();
                rules.push(FilterRule::SubdomainPattern(domain));
            }
            // Parse path patterns
            else if raw_rule.starts_with("/") && raw_rule.ends_with("/*") {
                rules.push(FilterRule::Pattern(raw_rule.to_string()));
            }
            // Parse wildcard patterns
            else if raw_rule.contains('*') {
                rules.push(FilterRule::Pattern(raw_rule.to_string()));
            }
            // Default to domain rule
            else {
                rules.push(FilterRule::Domain(raw_rule.to_string()));
            }
        }
        
        Ok(FilterEngine { rules })
    }
    
    /// Create a filter engine from a file
    pub fn from_file(path: &std::path::Path) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        Self::from_filter_list(&content)
    }
    
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
        // First check exception rules
        for rule in &self.rules {
            if let FilterRule::Exception(pattern) = rule {
                if self.matches_exception_pattern(url, pattern) {
                    return BlockDecision {
                        should_block: false,
                        reason: Some(format!("Whitelisted by exception: {}", pattern)),
                    };
                }
            }
        }
        
        // Then check blocking rules
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
                FilterRule::Exception(_) => {
                    // Already handled above
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
    
    /// Check if URL matches an exception pattern
    fn matches_exception_pattern(&self, url: &str, pattern: &str) -> bool {
        // Handle subdomain patterns
        if pattern.starts_with("||") {
            if pattern.ends_with("^") {
                let domain = &pattern[2..pattern.len()-1];
                return self.matches_subdomain(url, domain);
            } else {
                // Pattern like ||ads.com/acceptable/*
                // Extract domain and path from pattern
                let pattern_without_prefix = &pattern[2..];
                
                if let Some(slash_pos) = pattern_without_prefix.find('/') {
                    let domain = &pattern_without_prefix[..slash_pos];
                    let path_pattern = &pattern_without_prefix[slash_pos..];
                    
                    // Check if URL matches the domain
                    if self.matches_subdomain(url, domain) {
                        // Extract path from URL after domain
                        if let Some(url_domain_end) = url.find(domain) {
                            let url_after_domain = &url[url_domain_end + domain.len()..];
                            
                            // Check if path matches the pattern
                            if path_pattern.contains('*') {
                                return self.matches_wildcard_pattern(url_after_domain, path_pattern);
                            } else {
                                return url_after_domain.starts_with(path_pattern);
                            }
                        }
                    }
                } else {
                    // Just a domain pattern without path
                    return self.matches_subdomain(url, pattern_without_prefix);
                }
                
                return false;
            }
        }
        
        // Handle wildcard patterns
        if pattern.contains('*') {
            return self.matches_wildcard_pattern(url, pattern);
        }
        
        // Handle path patterns with wildcards like ads.com/acceptable/*
        if let Some(domain_end) = pattern.find('/') {
            let domain_part = &pattern[..domain_end];
            let path_part = &pattern[domain_end..];
            
            // Check if URL contains the domain and path pattern
            if url.contains(domain_part) {
                let url_after_domain = if let Some(pos) = url.find(domain_part) {
                    &url[pos + domain_part.len()..]
                } else {
                    return false;
                };
                
                // Match path pattern
                if path_part == "/*" || url_after_domain.starts_with(path_part.trim_end_matches('*')) {
                    return true;
                }
            }
        }
        
        // Simple contains check
        url.contains(pattern)
    }
    
    /// Create a new filter engine from configuration
    pub fn new(_config: &crate::Config) -> Result<Self, Box<dyn std::error::Error>> {
        // TODO: Load rules based on config
        // For now, just return default
        Ok(Self::new_with_defaults())
    }
}