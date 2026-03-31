mod plugin_trait;
#[cfg(feature = "gossip")]
pub mod gossip;
#[cfg(feature = "gossip")]
pub mod private_channel;
use plugin_trait::*;
use serde_json::json;
use std::os::raw::c_char;

/// Podping ZOS Plugin — git update notifications via gossip
///
/// Maps podping protocol to SOLFUNMEME:
///   Podcast feed URL ping  →  git push / witness ping
///   gossip topic           →  solfunmeme/git-updates/v1
///   trusted publishers     →  wallet-signed witnesses
///   SSE listener           →  dioxus real-time feed
///   archive DB             →  mesh logs + RabbitMQ

#[no_mangle] pub extern "C" fn zos_plugin_name() -> *mut c_char { to_c("podping") }
#[no_mangle] pub extern "C" fn zos_plugin_version() -> *mut c_char { to_c("0.1.0") }
#[no_mangle] pub extern "C" fn zos_plugin_commands() -> *mut c_char {
    to_c("ping,subscribe,publishers,topics,history,forge-hook")
}

#[no_mangle] pub extern "C" fn zos_plugin_execute(cmd: *const c_char, arg: *const c_char) -> *mut c_char {
    let cmd = unsafe { std::ffi::CStr::from_ptr(cmd) }.to_str().unwrap_or("");
    let arg = unsafe { std::ffi::CStr::from_ptr(arg) }.to_str().unwrap_or("");
    let ts = chrono::Utc::now().timestamp();

    let result = match cmd {
        "ping" => json!({
            "action": "broadcast",
            "url": arg,
            "topic": "solfunmeme/git-updates/v1",
            "reason": "update",
            "ts": ts
        }),
        "subscribe" => json!({
            "action": "subscribe",
            "topic": if arg.is_empty() { "solfunmeme/git-updates/v1" } else { arg },
            "protocol": "iroh-gossip",
            "transport": "QUIC"
        }),
        "publishers" => json!({
            "trusted": [
                {"name": "local-forgejo", "url": "http://localhost:3000", "key": "ed25519"},
                {"name": "github", "url": "https://github.com/meta-introspector", "key": "ed25519"},
            ]
        }),
        "topics" => json!({
            "topics": [
                "solfunmeme/git-updates/v1",
                "solfunmeme/witnesses/v1",
                "solfunmeme/ooda/v1",
                "solfunmeme/jocko/v1"
            ]
        }),
        "history" => json!({
            "action": "query",
            "topic": "solfunmeme/git-updates/v1",
            "limit": 10,
            "source": "mesh-logs + rabbitmq"
        }),
        "forge-hook" => json!({
            "action": "webhook",
            "forge": arg,
            "event": "push",
            "broadcast": "solfunmeme/git-updates/v1",
            "rmq_key": "paste.tag.podping"
        }),
        _ => json!({"error": cmd}),
    };

    let shard = DA51Shard::from_result("podping", cmd, &result);
    to_c(&serde_json::to_string(&json!({"result": result, "shard": shard})).unwrap())
}

#[no_mangle] pub extern "C" fn zos_plugin_render() -> *mut c_char {
    let gui = vec![
        GuiComponent::Heading { level: 2, text: "📡 Podping — Git Update Notifications".into() },
        GuiComponent::Table {
            headers: vec!["Topic".into(), "Protocol".into()],
            rows: vec![
                vec!["solfunmeme/git-updates/v1".into(), "iroh-gossip".into()],
                vec!["solfunmeme/witnesses/v1".into(), "iroh-gossip".into()],
                vec!["solfunmeme/ooda/v1".into(), "iroh-gossip".into()],
            ],
        },
        GuiComponent::KeyValue { pairs: vec![
            ("Transport".into(), "QUIC (iroh)".into()),
            ("Signing".into(), "ed25519-dalek".into()),
            ("Archive".into(), "mesh-logs + RabbitMQ".into()),
        ]},
        GuiComponent::Group { role: "toolbar".into(), children: vec![
            GuiComponent::Button { label: "Ping".into(), command: "ping".into() },
            GuiComponent::Button { label: "Subscribe".into(), command: "subscribe".into() },
            GuiComponent::Button { label: "History".into(), command: "history".into() },
            GuiComponent::Button { label: "Forge Hook".into(), command: "forge-hook".into() },
        ]},
    ];
    to_c(&serde_json::to_string(&gui).unwrap())
}

#[no_mangle] pub extern "C" fn zos_plugin_init() -> i32 { 0 }

#[cfg(test)]
mod jocko_fuzz {
    use super::*;
    use std::ffi::{CStr, CString};
    fn s(p: *mut c_char) -> String { unsafe { let s = CStr::from_ptr(p).to_string_lossy().into(); zos_free_string(p); s } }
    fn ex(cmd: &str, arg: &str) -> String {
        let c = CString::new(cmd).unwrap(); let a = CString::new(arg).unwrap();
        s(unsafe { zos_plugin_execute(c.as_ptr(), a.as_ptr()) })
    }
    #[test] fn init() { unsafe { assert_eq!(zos_plugin_init(), 0); } }
    #[test] fn identity() { assert!(!s(unsafe{zos_plugin_name()}).is_empty()); }
    #[test] fn render() { assert!(s(unsafe{zos_plugin_render()}).starts_with("[")); }
    #[test] fn fuzz_all() {
        for cmd in s(unsafe{zos_plugin_commands()}).split(',') {
            for i in &["", "https://github.com/meta-introspector/solfunmeme-dioxus", "🧮"] {
                let v: serde_json::Value = serde_json::from_str(&ex(cmd, i)).unwrap();
                assert!(v.get("shard").is_some());
            }
        }
    }
    #[test] fn da51() {
        let r = ex("ping", "https://github.com/meta-introspector/solfunmeme-dioxus");
        let v: serde_json::Value = serde_json::from_str(&r).unwrap();
        assert!(v["shard"]["cid"].as_str().unwrap().starts_with("bafk"));
        assert!(v["result"]["topic"].as_str().unwrap().contains("git-updates"));
    }
}
