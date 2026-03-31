//! DA51 Channel Registry — every app, agent, service gets a unique channel
//!
//! Each channel has:
//!   - Name (human readable)
//!   - DA51 address (0xDA51 + hash)
//!   - Orbifold coordinates (mod 71, mod 59, mod 47)
//!   - Private topic ID (secret-derived for iroh-gossip)
//!   - Bott periodicity index (0-7)
//!   - Hecke prime (from PRIMES_71)

#![cfg(feature = "gossip")]

use sha2::{Sha256, Digest};
use serde::{Serialize, Deserialize};

pub const DASL_TAG: u64 = 55889; // 0xDA51

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DA51Channel {
    pub name: String,
    pub kind: ChannelKind,
    pub dasl: String,
    pub orbifold: (u8, u8, u8),
    pub bott: u8,
    pub hecke_prime: u64,
    pub topic_id: [u8; 32],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChannelKind {
    App,
    Agent,
    Service,
    Plugin,
    Timer,
}

/// 71 primes for Hecke eigenvalue assignment
const PRIMES_71: [u64; 71] = [
    2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47, 53, 59, 61, 67, 71,
    73, 79, 83, 89, 97, 101, 103, 107, 109, 113, 127, 131, 137, 139, 149, 151,
    157, 163, 167, 173, 179, 181, 191, 193, 197, 199, 211, 223, 227, 229, 233,
    239, 241, 251, 257, 263, 269, 271, 277, 281, 283, 293, 307, 311, 313, 317,
    331, 337, 347, 349, 353,
];

impl DA51Channel {
    pub fn new(name: &str, kind: ChannelKind, secret: &str) -> Self {
        let hash = Sha256::digest(name.as_bytes());
        let v = u64::from_le_bytes(hash[0..8].try_into().unwrap());
        let orbifold = ((v % 71) as u8, (v % 59) as u8, (v % 47) as u8);
        let bott = (hash[2] % 8) as u8;
        let idx = (v % 71) as usize;
        let hecke_prime = PRIMES_71[idx];
        let dasl = format!("0xda51{}", hex::encode(&hash[..8]));

        // Private topic from secret
        let mut th = Sha256::new();
        th.update(b"solfunmeme-private-v1:");
        th.update(secret.as_bytes());
        th.update(b":");
        th.update(name.as_bytes());
        let topic_hash = th.finalize();
        let mut topic_id = [0u8; 32];
        topic_id.copy_from_slice(&topic_hash);

        Self { name: name.into(), kind, dasl, orbifold, bott, hecke_prime, topic_id }
    }
}

/// Generate channels for all known entities
pub fn registry(secret: &str) -> Vec<DA51Channel> {
    let mut channels = vec![];

    // Apps
    for name in &["solfunmeme-dioxus", "solfunmeme-service", "kant-pastebin", "erdfa-publish"] {
        channels.push(DA51Channel::new(name, ChannelKind::App, secret));
    }

    // Agents
    for name in &[
        "kagenti-daemon", "kagenti-portal", "jocko-training", "moltis",
        "fractran-generator", "git-events-consumer",
    ] {
        channels.push(DA51Channel::new(name, ChannelKind::Agent, secret));
    }

    // Services
    for name in &[
        "prometheus", "jaeger", "rabbitmq", "forgejo", "zos-noc-manager",
        "zos-minimal-server", "leech-lattice-24", "otp-bbs-bridge",
        "solfunmeme-mesh-sync", "rust-compiler-zkp", "zkperf",
    ] {
        channels.push(DA51Channel::new(name, ChannelKind::Service, secret));
    }

    // Plugins (41)
    for name in &[
        "monster", "mcp", "rust-parser", "orbits", "charts", "bert", "ooda-noc", "podping",
        "monster-manifold", "snm-atlas", "system-discovery", "harbor", "omnisearch",
        "a11y-gui", "github-manager", "noc-manager", "plugin-registry",
        "bm25", "eliza-rs", "emoji-lang", "extractous", "git", "gline-rs", "json-ld",
        "keyword-extraction", "layered-nlp", "llms-from-scratch", "model2vec", "nlprule",
        "quickwit", "rhai", "rust-sbert", "rust-sentence-transformers", "s3",
        "solana-airdrop", "steel", "tongrams", "vaporetto", "vibrato", "vtext", "zip-export",
    ] {
        channels.push(DA51Channel::new(&format!("zos-plugin-{}", name), ChannelKind::Plugin, secret));
    }

    // Timers
    for name in &[
        "kagenti-solfunmeme-test", "kagenti-solfunmeme-zkperf",
        "kagenti-solfunmeme-roundrobin", "jocko-training",
        "solfunmeme-mesh-sync", "solfunmeme-crawl", "solfunmeme-prove",
    ] {
        channels.push(DA51Channel::new(&format!("{}.timer", name), ChannelKind::Timer, secret));
    }

    channels
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_has_all_entities() {
        let r = registry("monster:gyroscope");
        assert!(r.len() > 60, "got {}", r.len());
    }

    #[test]
    fn all_dasl_unique() {
        let r = registry("monster:gyroscope");
        let dasls: std::collections::HashSet<String> = r.iter().map(|c| c.dasl.clone()).collect();
        assert_eq!(dasls.len(), r.len());
    }

    #[test]
    fn all_topics_unique() {
        let r = registry("monster:gyroscope");
        let topics: std::collections::HashSet<[u8; 32]> = r.iter().map(|c| c.topic_id).collect();
        assert_eq!(topics.len(), r.len());
    }

    #[test]
    fn dasl_format() {
        let ch = DA51Channel::new("test", ChannelKind::App, "secret");
        assert!(ch.dasl.starts_with("0xda51"));
        assert!(ch.orbifold.0 < 71);
        assert!(ch.orbifold.1 < 59);
        assert!(ch.orbifold.2 < 47);
        assert!(ch.bott < 8);
    }
}
