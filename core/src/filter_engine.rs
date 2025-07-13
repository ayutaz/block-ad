//! Filter Engine - Core ad blocking logic
//!
//! TDD Implementation - Starting with minimal code to pass tests

use aho_corasick::AhoCorasick;
use std::sync::Arc;
use crate::metrics::{PerformanceMetrics, PerfTimer};

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

/// Pattern info for tracking rule types
#[derive(Debug, Clone)]
struct PatternInfo {
    pattern: String,
    rule_type: PatternType,
}

#[derive(Debug, Clone, PartialEq)]
enum PatternType {
    Domain,
    Subdomain,
}

/// Main filter engine for ad blocking
pub struct FilterEngine {
    /// Compiled filter rules
    rules: Vec<FilterRule>,
    /// Aho-Corasick automaton for fast domain matching
    domain_matcher: Option<Arc<AhoCorasick>>,
    /// Pattern info for matched patterns
    pattern_info: Vec<PatternInfo>,
    /// Performance metrics
    metrics: PerformanceMetrics,
}

impl FilterEngine {
    /// Create a filter engine from a filter list string
    pub fn from_filter_list(filter_list: &str) -> Result<Self, Box<dyn std::error::Error>> {
        use crate::filter_list::FilterListLoader;

        let loader = FilterListLoader::new();
        let raw_rules = loader.parse_filter_list(filter_list)?;

        let rules: Vec<FilterRule> = raw_rules.into_iter().map(Self::parse_rule).collect();

        let mut engine = FilterEngine {
            rules,
            domain_matcher: None,
            pattern_info: Vec::new(),
            metrics: PerformanceMetrics::new(),
        };

        engine.compile_patterns();
        Ok(engine)
    }

    /// Parse a raw rule string into a FilterRule
    fn parse_rule(raw_rule: String) -> FilterRule {
        if let Some(stripped) = raw_rule.strip_prefix("@@") {
            FilterRule::Exception(stripped.to_string())
        } else if let Some(stripped) = raw_rule.strip_prefix("||") {
            if let Some(domain) = stripped.strip_suffix('^') {
                FilterRule::SubdomainPattern(domain.to_string())
            } else {
                FilterRule::Pattern(raw_rule)
            }
        } else if raw_rule.contains('*') || (raw_rule.starts_with("/") && raw_rule.ends_with("/*"))
        {
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
            pattern_info: Vec::new(),
            metrics: PerformanceMetrics::new(),
        };

        engine.compile_patterns();
        engine
    }

    /// Create a new filter engine with custom patterns
    pub fn new_with_patterns(patterns: Vec<String>) -> Self {
        let rules = patterns.into_iter().map(Self::parse_rule).collect();

        let mut engine = FilterEngine {
            rules,
            domain_matcher: None,
            pattern_info: Vec::new(),
            metrics: PerformanceMetrics::new(),
        };

        engine.compile_patterns();
        engine
    }

    /// Compile patterns for efficient matching
    fn compile_patterns(&mut self) {
        // Extract patterns and their info for Aho-Corasick
        let mut patterns = Vec::new();
        self.pattern_info.clear();

        for rule in &self.rules {
            match rule {
                FilterRule::Domain(domain) => {
                    patterns.push(domain.clone());
                    self.pattern_info.push(PatternInfo {
                        pattern: domain.clone(),
                        rule_type: PatternType::Domain,
                    });
                }
                FilterRule::SubdomainPattern(domain) => {
                    patterns.push(domain.clone());
                    self.pattern_info.push(PatternInfo {
                        pattern: domain.clone(),
                        rule_type: PatternType::Subdomain,
                    });
                }
                _ => {}
            }
        }

        // Build Aho-Corasick automaton if we have patterns
        if !patterns.is_empty() {
            let ac = AhoCorasick::new(&patterns).unwrap();
            self.domain_matcher = Some(Arc::new(ac));
        }
        
        // Update metrics
        self.metrics.set_filter_count(self.rules.len());
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
        let timer = PerfTimer::start();
        // First check exception rules
        for rule in &self.rules {
            if let FilterRule::Exception(pattern) = rule {
                if self.matches_exception_pattern(url, pattern) {
                    return BlockDecision {
                        should_block: false,
                        reason: Some(format!("Whitelisted by exception: {pattern}")),
                    };
                }
            }
        }

        // Use Aho-Corasick for fast domain matching
        if let Some(decision) = self.check_aho_corasick_matches(url) {
            self.metrics.record_request(decision.should_block, timer.elapsed());
            return decision;
        }

        // Then check other blocking rules
        for rule in &self.rules {
            match rule {
                FilterRule::Domain(_) | FilterRule::SubdomainPattern(_) => {
                    // Already handled by Aho-Corasick above
                }
                FilterRule::Pattern(pattern) => {
                    if self.matches_wildcard_pattern(url, pattern) {
                        let decision = BlockDecision {
                            should_block: true,
                            reason: Some(format!("Matched pattern: {pattern}")),
                        };
                        self.metrics.record_request(decision.should_block, timer.elapsed());
                        return decision;
                    }
                }
                FilterRule::Exception(_) => {
                    // Already handled above
                }
            }
        }

        let decision = BlockDecision {
            should_block: false,
            reason: None,
        };
        self.metrics.record_request(decision.should_block, timer.elapsed());
        decision
    }

    /// Check Aho-Corasick matches
    fn check_aho_corasick_matches(&self, url: &str) -> Option<BlockDecision> {
        let matcher = self.domain_matcher.as_ref()?;

        for match_result in matcher.find_iter(url) {
            let pattern_info = &self.pattern_info[match_result.pattern()];

            match pattern_info.rule_type {
                PatternType::Subdomain => {
                    // Verify it's actually a subdomain match
                    if self.matches_subdomain(url, &pattern_info.pattern) {
                        return Some(BlockDecision {
                            should_block: true,
                            reason: Some(format!("Matched subdomain: {}", pattern_info.pattern)),
                        });
                    }
                }
                PatternType::Domain => {
                    return Some(BlockDecision {
                        should_block: true,
                        reason: Some(format!("Matched ad domain: {}", pattern_info.pattern)),
                    });
                }
            }
        }

        None
    }

    /// Check if URL matches a subdomain pattern
    fn matches_subdomain(&self, url: &str, domain: &str) -> bool {
        if let Some(start) = url.find("://") {
            let url_after_protocol = &url[start + 3..];
            let url_host = url_after_protocol.split('/').next().unwrap_or("");

            // Exact match or subdomain match
            url_host == domain || url_host.ends_with(&format!(".{domain}"))
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
        if let Some(pattern_without_prefix) = pattern.strip_prefix("||") {
            return self.matches_subdomain_pattern(url, pattern_without_prefix);
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
        if let Some(domain) = pattern_without_prefix.strip_suffix('^') {
            // Pattern like ||domain.com^
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

    /// Add a single rule to the engine
    pub fn add_rule(&mut self, rule: &str) {
        let parsed_rule = Self::parse_rule(rule.to_string());
        self.rules.push(parsed_rule);
    }

    /// Rebuild the domain matcher (alias for compile_patterns)
    pub fn build_domain_matcher(&mut self) {
        self.compile_patterns();
    }

    /// Load rules from EasyList format content
    pub fn load_easylist_rules(&mut self, content: &str) -> Result<(), Box<dyn std::error::Error>> {
        let loader = crate::FilterListLoader::new();
        let rules = loader.parse_filter_list(content)?;

        for rule_str in rules {
            self.add_rule(&rule_str);
        }

        // Rebuild the Aho-Corasick matcher after adding new rules
        self.build_domain_matcher();

        Ok(())
    }

    /// Create a new filter engine from configuration
    pub fn new(config: &crate::Config) -> Result<Self, Box<dyn std::error::Error>> {
        let mut engine = Self::new_with_defaults();

        // Load filter lists from config
        if !config.filter_lists.is_empty() {
            let loader = crate::FilterListLoader::new();
            for url in &config.filter_lists {
                if let Ok(content) = loader.load_from_url(url) {
                    engine.load_easylist_rules(&content)?;
                }
            }
        }

        // Load custom rules if specified
        if let Some(custom_path) = &config.custom_rules_path {
            if let Ok(content) = std::fs::read_to_string(custom_path) {
                engine.load_easylist_rules(&content)?;
            }
        }

        Ok(engine)
    }
    
    /// Get performance metrics
    pub fn get_metrics(&self) -> &PerformanceMetrics {
        &self.metrics
    }
    
    /// Reset performance metrics
    pub fn reset_metrics(&mut self) {
        self.metrics.reset();
    }
}
