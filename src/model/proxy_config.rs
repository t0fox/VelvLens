use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Protocol {
    Vless,
    Vmess,
    Trojan,
    Shadowsocks,
    Hysteria2,
    Tuic,
    Unknown,
}

impl Protocol {
    pub const ALL: [Self; 7] = [
        Self::Vless,
        Self::Vmess,
        Self::Trojan,
        Self::Shadowsocks,
        Self::Hysteria2,
        Self::Tuic,
        Self::Unknown,
    ];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Vless => "VLESS",
            Self::Vmess => "VMess",
            Self::Trojan => "Trojan",
            Self::Shadowsocks => "Shadowsocks",
            Self::Hysteria2 => "Hysteria2",
            Self::Tuic => "TUIC",
            Self::Unknown => "Unknown",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Security {
    Reality,
    Tls,
    None,
    Unknown,
}

impl Security {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Reality => "Reality",
            Self::Tls => "TLS",
            Self::None => "None",
            Self::Unknown => "Unknown",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Transport {
    Tcp,
    XHttp,
    WebSocket,
    Grpc,
    Http2,
    Quic,
    Unknown,
}

impl Transport {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Tcp => "TCP",
            Self::XHttp => "XHTTP",
            Self::WebSocket => "WebSocket",
            Self::Grpc => "gRPC",
            Self::Http2 => "HTTP/2",
            Self::Quic => "QUIC",
            Self::Unknown => "Unknown",
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConfigMetadata {
    pub source_url: Option<String>,
    pub depth: u8,
    pub exact_duplicate_count: usize,
    pub semantic_duplicate_group: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProxyConfig {
    pub id: String,
    pub protocol: Protocol,
    pub name: Option<String>,

    pub host: String,
    pub port: u16,

    pub uuid: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,

    pub security: Security,
    pub transport: Transport,

    pub sni: Option<String>,
    pub fingerprint: Option<String>,

    pub reality_public_key: Option<String>,
    pub reality_short_id: Option<String>,

    pub flow: Option<String>,
    pub encryption: Option<String>,
    pub path: Option<String>,
    pub host_header: Option<String>,
    pub service_name: Option<String>,
    pub mode: Option<String>,

    pub unknown_params: BTreeMap<String, Vec<String>>,
    pub raw_uri: String,
    pub metadata: ConfigMetadata,
}

impl ProxyConfig {
    pub fn semantic_key(&self) -> String {
        let canonical = format!(
            "{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{:?}",
            self.protocol.as_str(),
            self.host.to_ascii_lowercase(),
            self.port,
            self.security.as_str(),
            self.transport.as_str(),
            self.uuid.as_deref().unwrap_or_default(),
            self.username.as_deref().unwrap_or_default(),
            self.password.as_deref().unwrap_or_default(),
            self.sni.as_deref().unwrap_or_default(),
            self.fingerprint.as_deref().unwrap_or_default(),
            self.reality_public_key.as_deref().unwrap_or_default(),
            self.reality_short_id.as_deref().unwrap_or_default(),
            self.path.as_deref().unwrap_or_default(),
            self.service_name.as_deref().unwrap_or_default(),
            self.unknown_params,
        );
        let digest = Sha256::digest(canonical.as_bytes());
        format!("sha256:{digest:x}")
    }
}
