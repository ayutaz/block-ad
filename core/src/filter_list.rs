//! Filter list loading and parsing
//!
//! Supports EasyList format filter rules

/// Filter list loader for parsing EasyList format
pub struct FilterListLoader {
    // Future: Add configuration options
}

/// Parsed filter rule types
#[derive(Debug, Clone)]
pub enum ParsedRule {
    /// URL blocking rule
    UrlBlock(String),
    /// Exception rule (whitelist)
    Exception(String),
    /// CSS element hiding rule
    CssHide {
        selector: String,
        domains: Vec<String>,
    },
}

impl FilterListLoader {
    /// Create a new filter list loader
    pub fn new() -> Self {
        FilterListLoader {}
    }

    /// Load filter list from URL
    pub fn load_from_url(&self, url: &str) -> Result<String, Box<dyn std::error::Error>> {
        // For now, return empty string since we don't have network implementation yet
        // This will be implemented when network module is complete
        eprintln!("Warning: Network loading not implemented yet for URL: {}", url);
        Ok(String::new())
    }

    /// Parse a filter list string into rules
    pub fn parse_filter_list(
        &self,
        content: &str,
    ) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let mut rules = Vec::new();

        for line in content.lines() {
            let trimmed = line.trim();

            // Skip empty lines and comments
            if trimmed.is_empty() || trimmed.starts_with('!') {
                continue;
            }

            // Skip malformed rules
            if trimmed.starts_with('[') || trimmed.starts_with("!!!") {
                continue;
            }

            // Skip CSS rules for now (handled separately)
            if trimmed.starts_with("##") || trimmed.contains("##") {
                continue;
            }

            // Add valid rules
            rules.push(trimmed.to_string());
        }

        Ok(rules)
    }

    /// Get CSS rules for a specific domain
    pub fn get_css_rules(
        &self,
        content: &str,
        domain: &str,
    ) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let mut css_rules = Vec::new();

        for line in content.lines() {
            let trimmed = line.trim();

            // Global CSS rules
            if let Some(selector) = trimmed.strip_prefix("##") {
                css_rules.push(selector.to_string());
            }
            // Domain-specific CSS rules
            else if let Some(separator_pos) = trimmed.find("##") {
                let domains_part = &trimmed[..separator_pos];
                let selector = &trimmed[separator_pos + 2..];

                // Check if rule applies to this domain
                if let Some(excluded_domain) = domains_part.strip_prefix('~') {
                    // Exclusion rule
                    if excluded_domain != domain {
                        css_rules.push(selector.to_string());
                    }
                } else if domains_part == domain {
                    css_rules.push(selector.to_string());
                }
            }
        }

        Ok(css_rules)
    }
}

impl Default for FilterListLoader {
    fn default() -> Self {
        Self::new()
    }
}
