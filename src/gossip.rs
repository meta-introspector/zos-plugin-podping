//! Iroh gossip integration for podping — git update notifications over P2P
//!
//! Enabled with `--features gossip`
//!
//! Topics:
//!   solfunmeme/git-updates/v1  — forge push events
//!   solfunmeme/witnesses/v1    — zkperf witnesses
//!   solfunmeme/ooda/v1         — OODA loop state
//!   solfunmeme/jocko/v1        — fuzz results

#![cfg(feature = "gossip")]

use iroh::SecretKey;
use iroh_gossip::net::Gossip;
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use std::sync::Arc;
use tokio::sync::broadcast;

pub const TOPIC_GIT: &str = "solfunmeme/git-updates/v1";
pub const TOPIC_WITNESS: &str = "solfunmeme/witnesses/v1";
pub const TOPIC_OODA: &str = "solfunmeme/ooda/v1";
pub const TOPIC_JOCKO: &str = "solfunmeme/jocko/v1";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PodpingMessage {
    pub topic: String,
    pub url: String,
    pub reason: String,
    pub timestamp: u64,
    pub signature: String,
    pub node_id: String,
}

impl PodpingMessage {
    pub fn new(topic: &str, url: &str, reason: &str) -> Self {
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let mut h = Sha256::new();
        h.update(format!("{}:{}:{}", topic, url, ts));
        Self {
            topic: topic.into(),
            url: url.into(),
            reason: reason.into(),
            timestamp: ts,
            signature: hex::encode(h.finalize()),
            node_id: String::new(),
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).unwrap_or_default()
    }

    pub fn from_bytes(data: &[u8]) -> Option<Self> {
        serde_json::from_slice(data).ok()
    }
}

/// Topic ID from string (SHA-256 hash)
pub fn topic_id(name: &str) -> [u8; 32] {
    let hash = Sha256::digest(name.as_bytes());
    let mut id = [0u8; 32];
    id.copy_from_slice(&hash);
    id
}
