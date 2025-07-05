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
        
        BlockDecision {
            should_block: false,
            reason: None,
        }
    }
    
    /// Create a new filter engine from configuration
    pub fn new(_config: &crate::Config) -> Result<Self, Box<dyn std::error::Error>> {
        // TODO: Load rules based on config
        // For now, just return default
        Ok(Self::new_with_defaults())
    }
}