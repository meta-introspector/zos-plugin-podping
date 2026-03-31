//! Private gossip channels for SOLFUNMEME
//!
//! Topics are SHA-256(secret + channel_name). Only nodes with the secret can join.
//! The secret is shared out-of-band (WireGuard mesh, pastebin, QR code).

#![cfg(feature = "gossip")]

use sha2::{Sha256, Digest};

/// Derive a private topic ID from a shared secret + channel name.
/// Only nodes that know the secret can compute the same topic ID.
pub fn private_topic(secret: &str, channel: &str) -> [u8; 32] {
    let mut h = Sha256::new();
    h.update(b"solfunmeme-private-v1:");
    h.update(secret.as_bytes());
    h.update(b":");
    h.update(channel.as_bytes());
    let hash = h.finalize();
    let mut id = [0u8; 32];
    id.copy_from_slice(&hash);
    id
}

/// Pre-defined private channels (secret required to join)
pub const CHANNELS: &[&str] = &[
    "git-updates",   // forge push events
    "witnesses",     // zkperf witnesses
    "ooda",          // OODA loop state
    "jocko",         // fuzz results
    "mesh-sync",     // WireGuard mesh log replication
    "dao-votes",     // governance votes
];

/// Generate all topic IDs for a given secret
pub fn all_topics(secret: &str) -> Vec<(&'static str, [u8; 32])> {
    CHANNELS.iter().map(|ch| (*ch, private_topic(secret, ch))).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_secret_same_topic() {
        let a = private_topic("monster:gyroscope", "git-updates");
        let b = private_topic("monster:gyroscope", "git-updates");
        assert_eq!(a, b);
    }

    #[test]
    fn different_secret_different_topic() {
        let a = private_topic("monster:gyroscope", "git-updates");
        let b = private_topic("wrong-secret", "git-updates");
        assert_ne!(a, b);
    }

    #[test]
    fn different_channel_different_topic() {
        let a = private_topic("monster:gyroscope", "git-updates");
        let b = private_topic("monster:gyroscope", "witnesses");
        assert_ne!(a, b);
    }

    #[test]
    fn all_channels_unique() {
        let topics = all_topics("monster:gyroscope");
        assert_eq!(topics.len(), CHANNELS.len());
        let ids: std::collections::HashSet<[u8; 32]> = topics.iter().map(|(_, id)| *id).collect();
        assert_eq!(ids.len(), CHANNELS.len());
    }
}
