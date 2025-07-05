//! Filter Engine - Core ad blocking logic
//! 
//! TDD Implementation - Starting with minimal code to pass tests

use aho_corasick::AhoCorasick;
use std::sync::Arc;

/// Result of a block decision
#[derive(Debug, Clone, PartialEq)]
pub struct BlockDecision {
    /// Whether the request should be blocked
    pub should_block: bool,
    /// Optional reason for the decision
    pub reason: Option<String>,
}

/// Pattern matching statistics
#[derive(Debug, Clone)]
pub struct PatternStats {
    /// Number of compiled patterns
    pub compiled_patterns: usize,
    /// Whether Aho-Corasick is used
    pub uses_aho_corasick: bool,
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
    /// Aho-Corasick automaton for fast domain matching
    domain_matcher: Option<Arc<AhoCorasick>>,
    /// Domain patterns for Aho-Corasick
    domain_patterns: Vec<String>,
}

impl FilterEngine {
    /// Create a filter engine from a filter list string
    pub fn from_filter_list(filter_list: &str) -> Result<Self, Box<dyn std::error::Error>> {
        use crate::filter_list::FilterListLoader;
        
        let loader = FilterListLoader::new();
        let raw_rules = loader.parse_filter_list(filter_list)?;
        
        let rules: Vec<FilterRule> = raw_rules
            .into_iter()
            .map(Self::parse_rule)
            .collect();
        
        let mut engine = FilterEngine {
            rules,
            domain_matcher: None,
            domain_patterns: Vec::new(),
        };
        
        engine.compile_patterns();
        Ok(engine)
    }
    
    /// Parse a raw rule string into a FilterRule
    fn parse_rule(raw_rule: String) -> FilterRule {
        if raw_rule.starts_with("@@") {
            FilterRule::Exception(raw_rule[2..].to_string())
        } else if raw_rule.starts_with("||") && raw_rule.ends_with("^") {
            FilterRule::SubdomainPattern(raw_rule[2..raw_rule.len()-1].to_string())
        } else if raw_rule.contains('*') || (raw_rule.starts_with("/") && raw_rule.ends_with("/*")) {
            FilterRule::Pattern(raw_rule)
        } else {
            FilterRule::Domain(raw_rule)
        }
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
        
        let mut engine = FilterEngine {
            rules,
            domain_matcher: None,
            domain_patterns: Vec::new(),
        };
        
        engine.compile_patterns();
        engine
    }
    
    /// Create a new filter engine with custom patterns
    pub fn new_with_patterns(patterns: Vec<String>) -> Self {
        let rules = patterns.into_iter()
            .map(Self::parse_rule)
            .collect();
        
        let mut engine = FilterEngine {
            rules,
            domain_matcher: None,
            domain_patterns: Vec::new(),
        };
        
        engine.compile_patterns();
        engine
    }
    
    /// Compile patterns for efficient matching
    fn compile_patterns(&mut self) {
        // Extract domain and subdomain patterns for Aho-Corasick
        self.domain_patterns = self.rules.iter()
            .filter_map(|rule| match rule {
                FilterRule::Domain(domain) => Some(domain.clone()),
                FilterRule::SubdomainPattern(domain) => Some(domain.clone()),
                _ => None,
            })
            .collect();
        
        // Build Aho-Corasick automaton if we have domain patterns
        if !self.domain_patterns.is_empty() {
            let ac = AhoCorasick::new(&self.domain_patterns).unwrap();
            self.domain_matcher = Some(Arc::new(ac));
        }
    }
    
    /// Get pattern statistics
    pub fn get_pattern_stats(&self) -> PatternStats {
        PatternStats {
            compiled_patterns: self.rules.len(),
            uses_aho_corasick: self.domain_matcher.is_some(),
        }
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
        
        // Use Aho-Corasick for fast domain matching if available
        if let Some(ref matcher) = self.domain_matcher {
            for match_result in matcher.find_iter(url) {
                let matched_pattern = &self.domain_patterns[match_result.pattern()];
                
                // Check if this is a subdomain pattern that needs special handling
                let is_subdomain_pattern = self.rules.iter().any(|rule| {
                    matches!(rule, FilterRule::SubdomainPattern(domain) if domain == matched_pattern)
                });
                
                if is_subdomain_pattern {
                    // Verify it's actually a subdomain match
                    if self.matches_subdomain(url, matched_pattern) {
                        return BlockDecision {
                            should_block: true,
                            reason: Some(format!("Matched subdomain: {}", matched_pattern)),
                        };
                    }
                } else {
                    // Regular domain match
                    return BlockDecision {
                        should_block: true,
                        reason: Some(format!("Matched ad domain: {}", matched_pattern)),
                    };
                }
            }
        }
        
        // Then check other blocking rules
        for rule in &self.rules {
            match rule {
                FilterRule::Domain(_) | FilterRule::SubdomainPattern(_) => {
                    // Already handled by Aho-Corasick above
                }
                FilterRule::Pattern(pattern) => {
                    if self.matches_wildcard_pattern(url, pattern) {
                        return BlockDecision {
                            should_block: true,
                            reason: Some(format!("Matched pattern: {}", pattern)),
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
        // Handle subdomain patterns (||domain)
        if pattern.starts_with("||") {
            return self.matches_subdomain_pattern(url, &pattern[2..]);
        }
        
        // Handle wildcard patterns
        if pattern.contains('*') {
            return self.matches_wildcard_pattern(url, pattern);
        }
        
        // Handle domain/path patterns
        if let Some(slash_pos) = pattern.find('/') {
            return self.matches_domain_path_pattern(url, pattern, slash_pos);
        }
        
        // Simple contains check
        url.contains(pattern)
    }
    
    /// Match subdomain patterns like ||domain.com or ||domain.com/path/*
    fn matches_subdomain_pattern(&self, url: &str, pattern_without_prefix: &str) -> bool {
        if pattern_without_prefix.ends_with("^") {
            // Pattern like ||domain.com^
            let domain = &pattern_without_prefix[..pattern_without_prefix.len()-1];
            self.matches_subdomain(url, domain)
        } else if let Some(slash_pos) = pattern_without_prefix.find('/') {
            // Pattern like ||domain.com/path/*
            let domain = &pattern_without_prefix[..slash_pos];
            let path_pattern = &pattern_without_prefix[slash_pos..];
            
            if self.matches_subdomain(url, domain) {
                if let Some(url_domain_end) = url.find(domain) {
                    let url_after_domain = &url[url_domain_end + domain.len()..];
                    
                    if path_pattern.contains('*') {
                        self.matches_wildcard_pattern(url_after_domain, path_pattern)
                    } else {
                        url_after_domain.starts_with(path_pattern)
                    }
                } else {
                    false
                }
            } else {
                false
            }
        } else {
            // Just ||domain without path
            self.matches_subdomain(url, pattern_without_prefix)
        }
    }
    
    /// Match domain/path patterns like domain.com/path/*
    fn matches_domain_path_pattern(&self, url: &str, pattern: &str, slash_pos: usize) -> bool {
        let domain_part = &pattern[..slash_pos];
        let path_part = &pattern[slash_pos..];
        
        if url.contains(domain_part) {
            if let Some(pos) = url.find(domain_part) {
                let url_after_domain = &url[pos + domain_part.len()..];
                path_part == "/*" || url_after_domain.starts_with(path_part.trim_end_matches('*'))
            } else {
                false
            }
        } else {
            false
        }
    }
    
    /// Create a new filter engine from configuration
    pub fn new(_config: &crate::Config) -> Result<Self, Box<dyn std::error::Error>> {
        // TODO: Load rules based on config
        // For now, just return default
        Ok(Self::new_with_defaults())
    }
}