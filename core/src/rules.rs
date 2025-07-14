//! Filter rules management
//!
//! This module handles loading, parsing, and managing filter rules

use regex::Regex;
use std::collections::HashMap;

/// Rule type enumeration
#[derive(Debug, Clone, PartialEq)]
pub enum RuleType {
    /// Block URL pattern
    Block,
    /// Allow URL pattern (exception)
    Allow,
    /// Element hiding rule
    ElementHide,
    /// Content script injection
    ScriptInject,
}

/// A parsed filter rule
#[derive(Debug, Clone)]
pub struct FilterRule {
    pub rule_type: RuleType,
    pub pattern: String,
    pub domains: Option<Vec<String>>,
    pub options: RuleOptions,
}

/// Rule options and modifiers
#[derive(Debug, Clone, Default)]
pub struct RuleOptions {
    pub third_party: Option<bool>,
    pub first_party: Option<bool>,
    pub script: Option<bool>,
    pub image: Option<bool>,
    pub stylesheet: Option<bool>,
    pub object: Option<bool>,
    pub xmlhttprequest: Option<bool>,
    pub subdocument: Option<bool>,
    pub document: Option<bool>,
    pub websocket: Option<bool>,
    pub webrtc: Option<bool>,
    pub ping: Option<bool>,
    pub media: Option<bool>,
    pub font: Option<bool>,
    pub popup: Option<bool>,
    pub domain: Option<Vec<String>>,
    pub sitekey: Option<String>,
}

/// Rule parser for EasyList format
pub struct RuleParser {
    compiled_patterns: HashMap<String, Regex>,
}

impl RuleParser {
    /// Create a new rule parser
    pub fn new() -> Self {
        RuleParser {
            compiled_patterns: HashMap::new(),
        }
    }

    /// Parse a single rule line
    pub fn parse_rule(&mut self, line: &str) -> Option<FilterRule> {
        let line = line.trim();

        // Skip empty lines and comments
        if line.is_empty() || line.starts_with('!') {
            return None;
        }

        // Skip metadata
        if line.starts_with('[') && line.ends_with(']') {
            return None;
        }

        // Element hiding rules
        if line.contains("##") || line.contains("#@#") {
            return self.parse_element_hiding_rule(line);
        }

        // URL filtering rules
        self.parse_url_rule(line)
    }

    /// Parse element hiding rule
    fn parse_element_hiding_rule(&self, line: &str) -> Option<FilterRule> {
        let is_exception = line.contains("#@#");
        let separator = if is_exception { "#@#" } else { "##" };

        let parts: Vec<&str> = line.splitn(2, separator).collect();
        if parts.len() != 2 {
            return None;
        }

        let domains = if parts[0].is_empty() {
            None
        } else {
            Some(parts[0].split(',').map(|s| s.trim().to_string()).collect())
        };

        let selector = parts[1].trim();

        Some(FilterRule {
            rule_type: if is_exception {
                RuleType::Allow
            } else {
                RuleType::ElementHide
            },
            pattern: selector.to_string(),
            domains,
            options: RuleOptions::default(),
        })
    }

    /// Parse URL filtering rule
    fn parse_url_rule(&mut self, line: &str) -> Option<FilterRule> {
        let (pattern, options_str) = if let Some(pos) = line.rfind('$') {
            let (p, o) = line.split_at(pos);
            (p, Some(&o[1..]))
        } else {
            (line, None)
        };

        // Check if it's an exception rule
        let (rule_type, pattern) = if let Some(stripped) = pattern.strip_prefix("@@") {
            (RuleType::Allow, stripped)
        } else {
            (RuleType::Block, pattern)
        };

        // Parse options
        let options = if let Some(opts) = options_str {
            self.parse_options(opts)
        } else {
            RuleOptions::default()
        };

        // Convert pattern to regex
        let _regex_pattern = self.pattern_to_regex(pattern);

        Some(FilterRule {
            rule_type,
            pattern: pattern.to_string(),
            domains: options.domain.clone(),
            options,
        })
    }

    /// Parse rule options
    fn parse_options(&self, options_str: &str) -> RuleOptions {
        let mut options = RuleOptions::default();

        for option in options_str.split(',') {
            let option = option.trim();

            match option {
                "third-party" => options.third_party = Some(true),
                "~third-party" => options.third_party = Some(false),
                "first-party" => options.first_party = Some(true),
                "~first-party" => options.first_party = Some(false),
                "script" => options.script = Some(true),
                "~script" => options.script = Some(false),
                "image" => options.image = Some(true),
                "~image" => options.image = Some(false),
                "stylesheet" => options.stylesheet = Some(true),
                "~stylesheet" => options.stylesheet = Some(false),
                "object" => options.object = Some(true),
                "~object" => options.object = Some(false),
                "xmlhttprequest" => options.xmlhttprequest = Some(true),
                "~xmlhttprequest" => options.xmlhttprequest = Some(false),
                "subdocument" => options.subdocument = Some(true),
                "~subdocument" => options.subdocument = Some(false),
                "document" => options.document = Some(true),
                "~document" => options.document = Some(false),
                "websocket" => options.websocket = Some(true),
                "~websocket" => options.websocket = Some(false),
                "webrtc" => options.webrtc = Some(true),
                "~webrtc" => options.webrtc = Some(false),
                "ping" => options.ping = Some(true),
                "~ping" => options.ping = Some(false),
                "media" => options.media = Some(true),
                "~media" => options.media = Some(false),
                "font" => options.font = Some(true),
                "~font" => options.font = Some(false),
                "popup" => options.popup = Some(true),
                "~popup" => options.popup = Some(false),
                _ => {
                    if let Some(domains) = option.strip_prefix("domain=") {
                        options.domain =
                            Some(domains.split('|').map(|d| d.trim().to_string()).collect());
                    } else if let Some(key) = option.strip_prefix("sitekey=") {
                        options.sitekey = Some(key.to_string());
                    }
                }
            }
        }

        options
    }

    /// Convert filter pattern to regex
    fn pattern_to_regex(&mut self, pattern: &str) -> String {
        // Check cache
        if let Some(regex) = self.compiled_patterns.get(pattern) {
            return regex.as_str().to_string();
        }

        let mut regex = String::new();
        let mut chars = pattern.chars().peekable();

        // Handle beginning anchor
        if pattern.starts_with("||") {
            regex.push_str("^(https?://)?([^/]*\\.)?");
            chars.next();
            chars.next();
        } else if pattern.starts_with('|') {
            regex.push('^');
            chars.next();
        }

        while let Some(ch) = chars.next() {
            match ch {
                '*' => regex.push_str(".*"),
                '^' => regex.push_str("([?/&#]|$)"),
                '?' => regex.push('.'),
                '.' | '+' | '(' | ')' | '[' | ']' | '{' | '}' | '$' => {
                    regex.push('\\');
                    regex.push(ch);
                }
                '|' if chars.peek().is_none() => regex.push('$'),
                _ => regex.push(ch),
            }
        }

        // Compile and cache
        if let Ok(compiled) = Regex::new(&regex) {
            self.compiled_patterns.insert(pattern.to_string(), compiled);
        }

        regex
    }
}

impl Default for RuleParser {
    fn default() -> Self {
        Self::new()
    }
}

/// Rule matcher for efficient rule matching
pub struct RuleMatcher {
    block_rules: Vec<FilterRule>,
    allow_rules: Vec<FilterRule>,
    element_hide_rules: Vec<FilterRule>,
    domain_index: HashMap<String, Vec<usize>>,
}

impl RuleMatcher {
    /// Create a new rule matcher
    pub fn new() -> Self {
        RuleMatcher {
            block_rules: Vec::new(),
            allow_rules: Vec::new(),
            element_hide_rules: Vec::new(),
            domain_index: HashMap::new(),
        }
    }

    /// Add a parsed rule
    pub fn add_rule(&mut self, rule: FilterRule) {
        match rule.rule_type {
            RuleType::Block => {
                let index = self.block_rules.len();
                self.block_rules.push(rule);
                self.update_domain_index(index, true);
            }
            RuleType::Allow => {
                let index = self.allow_rules.len();
                self.allow_rules.push(rule);
                self.update_domain_index(index, false);
            }
            RuleType::ElementHide => {
                self.element_hide_rules.push(rule);
            }
            _ => {}
        }
    }

    /// Update domain index for faster lookups
    fn update_domain_index(&mut self, rule_index: usize, is_block: bool) {
        let rules = if is_block {
            &self.block_rules
        } else {
            &self.allow_rules
        };

        if let Some(rule) = rules.get(rule_index) {
            if let Some(ref domains) = rule.options.domain {
                for domain in domains {
                    let key = format!("{}:{}", if is_block { "block" } else { "allow" }, domain);
                    self.domain_index.entry(key).or_default().push(rule_index);
                }
            }
        }
    }

    /// Check if a URL should be blocked
    pub fn should_block(&self, url: &str, options: &MatchOptions) -> bool {
        // First check allow rules (exceptions)
        for rule in &self.allow_rules {
            if self.matches_rule(rule, url, options) {
                return false;
            }
        }

        // Then check block rules
        for rule in &self.block_rules {
            if self.matches_rule(rule, url, options) {
                return true;
            }
        }

        false
    }

    /// Check if a rule matches
    fn matches_rule(&self, rule: &FilterRule, url: &str, options: &MatchOptions) -> bool {
        // Check domain restrictions
        if let Some(ref domains) = rule.options.domain {
            if let Some(ref page_domain) = options.domain {
                let matches_domain = domains.iter().any(|d| {
                    if let Some(stripped) = d.strip_prefix('~') {
                        !page_domain.ends_with(stripped)
                    } else {
                        page_domain.ends_with(d)
                    }
                });

                if !matches_domain {
                    return false;
                }
            }
        }

        // Check content type restrictions
        if !self.matches_content_type(rule, options) {
            return false;
        }

        // Match pattern
        self.matches_pattern(&rule.pattern, url)
    }

    /// Check content type restrictions
    fn matches_content_type(&self, rule: &FilterRule, options: &MatchOptions) -> bool {
        let opts = &rule.options;

        match options.content_type {
            ContentType::Script => opts.script.unwrap_or(true),
            ContentType::Image => opts.image.unwrap_or(true),
            ContentType::Stylesheet => opts.stylesheet.unwrap_or(true),
            ContentType::Object => opts.object.unwrap_or(true),
            ContentType::XmlHttpRequest => opts.xmlhttprequest.unwrap_or(true),
            ContentType::Subdocument => opts.subdocument.unwrap_or(true),
            ContentType::Document => opts.document.unwrap_or(true),
            ContentType::Websocket => opts.websocket.unwrap_or(true),
            ContentType::Media => opts.media.unwrap_or(true),
            ContentType::Font => opts.font.unwrap_or(true),
            ContentType::Other => true,
        }
    }

    /// Match URL against pattern
    fn matches_pattern(&self, pattern: &str, url: &str) -> bool {
        // Simple pattern matching for now
        // In production, use compiled regexes

        if let Some(stripped) = pattern.strip_prefix("||") {
            if let Some(separator_pos) = stripped.find('^') {
                let domain = &stripped[..separator_pos];
                return url.contains(domain);
            }
        }

        url.contains(pattern.trim_matches('*'))
    }

    /// Get element hiding rules for a domain
    pub fn get_element_hiding_rules(&self, domain: &str) -> Vec<&FilterRule> {
        self.element_hide_rules
            .iter()
            .filter(|rule| {
                if let Some(ref domains) = rule.domains {
                    domains.iter().any(|d| domain.ends_with(d))
                } else {
                    true // Global rule
                }
            })
            .collect()
    }
}

impl Default for RuleMatcher {
    fn default() -> Self {
        Self::new()
    }
}

/// Match options for rule matching
#[derive(Debug, Clone)]
pub struct MatchOptions {
    pub domain: Option<String>,
    pub content_type: ContentType,
    pub is_third_party: bool,
}

/// Content types for filtering
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ContentType {
    Script,
    Image,
    Stylesheet,
    Object,
    XmlHttpRequest,
    Subdocument,
    Document,
    Websocket,
    Media,
    Font,
    Other,
}
