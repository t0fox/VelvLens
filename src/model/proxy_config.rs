use std::{collections::BTreeMap, fmt, sync::Arc};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Protocol {
    Vless,
    Vmess,
    Trojan,
    Shadowsocks,
    Socks5,
    Hysteria2,
    Tuic,
    Unknown,
}

impl Protocol {
    pub const ALL: [Self; 8] = [
        Self::Vless,
        Self::Vmess,
        Self::Trojan,
        Self::Shadowsocks,
        Self::Socks5,
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
            Self::Socks5 => "Socks5",
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OriginalRepresentation {
    ShareUri(String),
    JsonProfile {
        document: Arc<str>,
        profile_index: usize,
        outbound_index: usize,
        endpoint_index: usize,
    },
}

impl OriginalRepresentation {
    pub fn json_profile(
        document: Arc<str>,
        profile_index: usize,
        outbound_index: usize,
        endpoint_index: usize,
    ) -> Self {
        Self::JsonProfile {
            document,
            profile_index,
            outbound_index,
            endpoint_index,
        }
    }

    pub fn text(&self) -> &str {
        match self {
            Self::ShareUri(uri) => uri,
            Self::JsonProfile { document, .. } => document,
        }
    }

    pub fn is_json(&self) -> bool {
        matches!(self, Self::JsonProfile { .. })
    }

    pub fn identity_key(&self) -> String {
        match self {
            Self::ShareUri(uri) => format!("uri:{uri}"),
            Self::JsonProfile {
                document,
                profile_index,
                outbound_index,
                endpoint_index,
            } => {
                let digest = Sha256::digest(document.as_bytes());
                format!("json:{digest:x}:{profile_index}:{outbound_index}:{endpoint_index}")
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConversionLimitation {
    MissingRequired { field: String },
    UnsupportedParameter { field: String },
    AmbiguousParameter { field: String },
    InvalidValue { field: String },
}

impl fmt::Display for ConversionLimitation {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingRequired { field } => {
                write!(formatter, "Не указан обязательный параметр: {field}")
            }
            Self::UnsupportedParameter { field } => {
                write!(formatter, "Параметр не поддерживается share URI: {field}")
            }
            Self::AmbiguousParameter { field } => write!(
                formatter,
                "Параметр нельзя однозначно перенести в share URI: {field}"
            ),
            Self::InvalidValue { field } => {
                write!(formatter, "Некорректное значение параметра: {field}")
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShareUriResult {
    Available {
        uri: String,
    },
    Limited {
        uri: String,
        limitations: Vec<ConversionLimitation>,
    },
    Unavailable {
        reasons: Vec<ConversionLimitation>,
    },
}

impl ShareUriResult {
    pub fn uri(&self) -> Option<&str> {
        match self {
            Self::Available { uri } | Self::Limited { uri, .. } => Some(uri),
            Self::Unavailable { .. } => None,
        }
    }

    pub fn limitations(&self) -> &[ConversionLimitation] {
        match self {
            Self::Available { .. } => &[],
            Self::Limited { limitations, .. } => limitations,
            Self::Unavailable { reasons } => reasons,
        }
    }

    pub fn is_available(&self) -> bool {
        matches!(self, Self::Available { .. })
    }

    pub fn is_limited(&self) -> bool {
        matches!(self, Self::Limited { .. })
    }

    pub fn is_unavailable(&self) -> bool {
        matches!(self, Self::Unavailable { .. })
    }
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
    /// Labels found around the endpoint in source formats such as Xray JSON.
    /// They are kept separately from the display name so filters can match
    /// transport/source tags that are not chosen as the visible title.
    #[serde(default)]
    pub labels: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProxyConfig {
    pub id: String,
    pub protocol: Protocol,
    pub name: Option<String>,

    pub host: String,
    pub port: u16,
    pub port_range: Option<String>,

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
    pub original: OriginalRepresentation,
    pub share_uri: ShareUriResult,
    pub metadata: ConfigMetadata,
}

impl ProxyConfig {
    pub fn original_text(&self) -> &str {
        self.original.text()
    }

    pub fn original_is_json(&self) -> bool {
        self.original.is_json()
    }

    pub fn semantic_key(&self) -> String {
        let canonical = format!(
            "{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{:?}",
            self.protocol.as_str(),
            self.host.to_ascii_lowercase(),
            self.port,
            self.port_range.as_deref().unwrap_or_default(),
            self.security.as_str(),
            self.transport.as_str(),
            self.uuid.as_deref().unwrap_or_default(),
            self.username.as_deref().unwrap_or_default(),
            self.password.as_deref().unwrap_or_default(),
            self.sni.as_deref().unwrap_or_default(),
            self.fingerprint.as_deref().unwrap_or_default(),
            self.reality_public_key.as_deref().unwrap_or_default(),
            self.reality_short_id.as_deref().unwrap_or_default(),
            self.flow.as_deref().unwrap_or_default(),
            self.encryption.as_deref().unwrap_or_default(),
            self.path.as_deref().unwrap_or_default(),
            self.host_header.as_deref().unwrap_or_default(),
            self.service_name.as_deref().unwrap_or_default(),
            self.mode.as_deref().unwrap_or_default(),
            self.unknown_params,
        );
        let digest = Sha256::digest(canonical.as_bytes());
        format!("sha256:{digest:x}")
    }
}
