use serde::{Serialize, Deserialize};
use serde_json::Value;
use sha2::{Sha256, Digest};
use std::os::raw::c_char;
use std::ffi::CString;

pub const DASL_TAG: u64 = 55889;
pub const PRIMES: [(u64, &str); 20] = [
    (2,"position"),(3,"credits"),(5,"crypto"),(7,"network"),(11,"count"),
    (13,"peers"),(17,"turn"),(19,"health"),(23,"cargo"),(29,"monitor"),
    (31,"build"),(37,"deploy"),(41,"test"),(43,"render"),(47,"agent"),
    (53,"stego"),(59,"tunnel"),(61,"record"),(67,"shard"),(71,"meta"),
];

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum GuiComponent {
    Heading { level: u8, text: String },
    Paragraph { text: String },
    Code { language: String, source: String },
    Table { headers: Vec<String>, rows: Vec<Vec<String>> },
    Button { label: String, command: String },
    KeyValue { pairs: Vec<(String, String)> },
    Group { role: String, children: Vec<GuiComponent> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DA51Shard {
    pub plugin: String, pub command: String, pub cid: String,
    pub dasl: String, pub orbifold: [u64; 3], pub bott: u8, pub data: Value,
}

impl DA51Shard {
    pub fn from_result(plugin: &str, command: &str, data: &Value) -> Self {
        let json = serde_json::to_vec(data).unwrap_or_default();
        let hash = Sha256::digest(&json);
        let cid = format!("bafk{}", hex::encode(&hash[..16]));
        let dasl = format!("0xda51{}", hex::encode(&hash[..8]));
        let n = u64::from_le_bytes(hash[..8].try_into().unwrap_or([0;8]));
        Self { plugin: plugin.into(), command: command.into(), cid, dasl,
               orbifold: [n%71, n%59, n%47], bott: (hash[2]%8) as u8, data: data.clone() }
    }
    pub fn to_cbor(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        ciborium::into_writer(self, &mut buf).unwrap_or_default();
        buf
    }
}

// C FFI helpers
pub fn to_c(s: &str) -> *mut c_char { CString::new(s).unwrap().into_raw() }

#[no_mangle] pub extern "C" fn zos_free_string(s: *mut c_char) {
    if !s.is_null() { unsafe { drop(CString::from_raw(s)); } }
}

// ── State sharing: read/write via DA51 CBOR ─────────────────────

/// Plugin state envelope — serialized as DA51 CBOR for P2P sharing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginState {
    pub plugin: String,
    pub version: String,
    pub fractran_state: u64,
    pub factors: Vec<(u64, u32)>,
    pub data: Value,
    pub shard: DA51Shard,
    pub timestamp: i64,
}

impl PluginState {
    pub fn to_cbor(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        ciborium::into_writer(self, &mut buf).unwrap_or_default();
        buf
    }
    pub fn from_cbor(bytes: &[u8]) -> Option<Self> {
        ciborium::from_reader(bytes).ok()
    }
    pub fn cid(&self) -> String {
        let cbor = self.to_cbor();
        let hash = Sha256::digest(&cbor);
        format!("bafk{}", hex::encode(&hash[..16]))
    }
}

/// FRACTRAN step: apply first matching ratio
pub fn fractran_run(mut state: u64, ratios: &[(Vec<(u64,u32)>, Vec<(u64,u32)>)]) -> u64 {
    for (num, den) in ratios {
        let mut s = state;
        let mut ok = true;
        for &(p, e) in den { for _ in 0..e { if s % p != 0 { ok = false; break; } s /= p; } if !ok { break; } }
        if ok { for &(p, e) in num { for _ in 0..e { s *= p; } } return s; }
    }
    state
}

// ── C FFI for state read/write ──────────────────────────────────

#[no_mangle] pub extern "C" fn zos_plugin_state_read() -> *mut c_char { to_c("{}") }
#[no_mangle] pub extern "C" fn zos_plugin_state_write(_cbor: *const u8, _len: usize) -> i32 { 0 }
#[no_mangle] pub extern "C" fn zos_plugin_state_cbor(out: *mut u8, max: usize) -> usize { 0 }
