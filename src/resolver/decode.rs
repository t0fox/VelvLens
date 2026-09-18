use base64::engine::general_purpose::{STANDARD, STANDARD_NO_PAD, URL_SAFE, URL_SAFE_NO_PAD};
use base64::Engine;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecodeCodec {
    Standard,
    StandardNoPad,
    UrlSafe,
    UrlSafeNoPad,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodedCandidate {
    pub codec: DecodeCodec,
    pub text: String,
}

pub fn decode_candidates(input: &str) -> Vec<DecodedCandidate> {
    let normalized = input
        .chars()
        .filter(|character| *character != '\u{feff}' && !character.is_ascii_whitespace())
        .collect::<String>();
    if normalized.is_empty() || normalized.contains("://") {
        return Vec::new();
    }

    let attempts = [
        (DecodeCodec::Standard, STANDARD.decode(&normalized)),
        (
            DecodeCodec::StandardNoPad,
            STANDARD_NO_PAD.decode(&normalized),
        ),
        (DecodeCodec::UrlSafe, URL_SAFE.decode(&normalized)),
        (
            DecodeCodec::UrlSafeNoPad,
            URL_SAFE_NO_PAD.decode(&normalized),
        ),
    ];

    attempts
        .into_iter()
        .filter_map(|(codec, result)| {
            let bytes = result.ok()?;
            let text = String::from_utf8(bytes).ok()?;
            is_useful_decoded(&text).then_some(DecodedCandidate { codec, text })
        })
        .fold(Vec::new(), |mut candidates, candidate| {
            if !candidates
                .iter()
                .any(|seen: &DecodedCandidate| seen.text == candidate.text)
            {
                candidates.push(candidate);
            }
            candidates
        })
}

fn is_useful_decoded(text: &str) -> bool {
    let trimmed = text.trim();
    let lower = trimmed.to_ascii_lowercase();
    lower.contains("vless://")
        || lower.contains("vmess://")
        || lower.contains("trojan://")
        || lower.contains("ss://")
        || lower.contains("hysteria://")
        || lower.contains("hysteria2://")
        || lower.contains("hy2://")
        || lower.contains("tuic://")
        || lower.contains("http://")
        || lower.contains("https://")
        || trimmed.starts_with('{')
        || trimmed.starts_with('[')
        || trimmed.starts_with('<')
        || contains_uri_scheme(trimmed)
}

fn contains_uri_scheme(text: &str) -> bool {
    text.lines().any(|line| {
        line.split_once("://").is_some_and(|(scheme, remainder)| {
            !remainder.is_empty()
                && !scheme.is_empty()
                && scheme.chars().enumerate().all(|(index, character)| {
                    character.is_ascii_alphanumeric()
                        || matches!(character, '+' | '-' | '.') && index > 0
                })
        })
    })
}
