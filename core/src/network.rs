//! Network interception and packet handling
//!
//! This module handles network-level filtering and DNS resolution

use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

/// DNS query types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DnsQueryType {
    A,     // IPv4 address
    AAAA,  // IPv6 address
    CNAME, // Canonical name
    MX,    // Mail exchange
    TXT,   // Text record
}

/// DNS query structure
#[derive(Debug, Clone)]
pub struct DnsQuery {
    pub domain: String,
    pub query_type: DnsQueryType,
    pub transaction_id: u16,
}

/// DNS response
#[derive(Debug, Clone)]
pub struct DnsResponse {
    pub transaction_id: u16,
    pub answers: Vec<DnsAnswer>,
    pub blocked: bool,
}

/// DNS answer record
#[derive(Debug, Clone)]
pub enum DnsAnswer {
    A(Ipv4Addr),
    AAAA(Ipv6Addr),
    CNAME(String),
    TXT(String),
}

/// Network filter for DNS-level blocking
pub struct NetworkFilter {
    blocked_domains: HashMap<String, bool>,
    redirect_ip: IpAddr,
}

impl NetworkFilter {
    /// Create a new network filter
    pub fn new() -> Self {
        NetworkFilter {
            blocked_domains: HashMap::new(),
            redirect_ip: IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
        }
    }

    /// Set the IP address to redirect blocked domains to
    pub fn set_redirect_ip(&mut self, ip: IpAddr) {
        self.redirect_ip = ip;
    }

    /// Add a domain to the blocklist
    pub fn add_blocked_domain(&mut self, domain: &str) {
        // Normalize domain (remove leading/trailing dots)
        let normalized = domain.trim_matches('.');
        self.blocked_domains.insert(normalized.to_lowercase(), true);

        // Also block www subdomain if not already present
        if !normalized.starts_with("www.") {
            let www_domain = format!("www.{}", normalized);
            self.blocked_domains.insert(www_domain.to_lowercase(), true);
        }
    }

    /// Check if a domain is blocked
    pub fn is_blocked(&self, domain: &str) -> bool {
        let normalized = domain.trim_matches('.').to_lowercase();

        // Check exact match
        if self.blocked_domains.contains_key(&normalized) {
            return true;
        }

        // Check parent domains
        let parts: Vec<&str> = normalized.split('.').collect();
        for i in 0..parts.len() {
            let parent = parts[i..].join(".");
            if self.blocked_domains.contains_key(&parent) {
                return true;
            }
        }

        false
    }

    /// Process a DNS query
    pub fn process_dns_query(&self, query: &DnsQuery) -> DnsResponse {
        let blocked = self.is_blocked(&query.domain);

        let answers = if blocked {
            match query.query_type {
                DnsQueryType::A => {
                    if let IpAddr::V4(ipv4) = self.redirect_ip {
                        vec![DnsAnswer::A(ipv4)]
                    } else {
                        vec![]
                    }
                }
                DnsQueryType::AAAA => {
                    if let IpAddr::V6(ipv6) = self.redirect_ip {
                        vec![DnsAnswer::AAAA(ipv6)]
                    } else {
                        vec![]
                    }
                }
                _ => vec![],
            }
        } else {
            vec![]
        };

        DnsResponse {
            transaction_id: query.transaction_id,
            answers,
            blocked,
        }
    }

    /// Load blocked domains from filter rules
    pub fn load_from_rules(&mut self, rules: &[String]) {
        for rule in rules {
            // Skip comments and empty lines
            if rule.trim().is_empty() || rule.starts_with('!') {
                continue;
            }

            // Extract domain from rule
            if let Some(domain) = extract_domain_from_rule(rule) {
                self.add_blocked_domain(&domain);
            }
        }
    }
}

/// Extract domain from a filter rule
fn extract_domain_from_rule(rule: &str) -> Option<String> {
    let rule = rule.trim();

    // Handle domain rules like ||example.com^
    if let Some(stripped) = rule.strip_prefix("||") {
        if let Some(domain_end) = stripped.find('^') {
            return Some(stripped[..domain_end].to_string());
        }
    }

    // Handle simple domain rules
    if !rule.contains('/')
        && !rule.contains('*')
        && !rule.contains('?')
        && rule.contains('.')
        && !rule.starts_with('.')
    {
        return Some(rule.to_string());
    }

    None
}

impl Default for NetworkFilter {
    fn default() -> Self {
        Self::new()
    }
}

/// Packet information for network-level filtering
#[derive(Debug, Clone)]
pub struct PacketInfo {
    pub src_ip: IpAddr,
    pub dst_ip: IpAddr,
    pub src_port: u16,
    pub dst_port: u16,
    pub protocol: Protocol,
    pub hostname: Option<String>,
}

/// Network protocols
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Protocol {
    TCP,
    UDP,
    ICMP,
    Other(u8),
}

impl PacketInfo {
    /// Create a new packet info
    pub fn new(
        src_ip: IpAddr,
        dst_ip: IpAddr,
        src_port: u16,
        dst_port: u16,
        protocol: Protocol,
    ) -> Self {
        PacketInfo {
            src_ip,
            dst_ip,
            src_port,
            dst_port,
            protocol,
            hostname: None,
        }
    }

    /// Set the hostname (from SNI or DNS)
    pub fn set_hostname(&mut self, hostname: String) {
        self.hostname = Some(hostname);
    }

    /// Check if this is HTTPS traffic
    pub fn is_https(&self) -> bool {
        self.dst_port == 443 && self.protocol == Protocol::TCP
    }

    /// Check if this is DNS traffic
    pub fn is_dns(&self) -> bool {
        self.dst_port == 53 && (self.protocol == Protocol::UDP || self.protocol == Protocol::TCP)
    }
}
