const URI_MASK: &str = "••••••••";

const SENSITIVE_KEYS: &[&str] = &[
    "uuid",
    "password",
    "pass",
    "token",
    "pbk",
    "sid",
    "private-key",
    "private_key",
    "secret",
    "auth",
];

pub fn redact_secret(value: &str) -> String {
    let chars: Vec<char> = value.chars().collect();
    if chars.len() <= 8 {
        return "••••••".to_owned();
    }

    let prefix: String = chars.iter().take(2).collect();
    let suffix: String = chars
        .iter()
        .rev()
        .take(2)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();
    format!("{prefix}••••••{suffix}")
}

pub fn redact_uri(uri: &str) -> String {
    let Some(scheme_end) = uri.find("://") else {
        return uri.to_owned();
    };

    let authority_start = scheme_end + 3;
    let authority_end = uri[authority_start..]
        .find(['/', '?', '#'])
        .map(|offset| authority_start + offset)
        .unwrap_or(uri.len());

    let mut output = String::with_capacity(uri.len());
    output.push_str(&uri[..authority_start]);
    let authority = &uri[authority_start..authority_end];
    if let Some(at) = authority.rfind('@') {
        output.push_str(URI_MASK);
        output.push('@');
        output.push_str(&authority[at + 1..]);
    } else {
        output.push_str(authority);
    }

    let remainder = redact_http_path(&uri[..scheme_end], &uri[authority_end..]);
    if let Some(query_start) = remainder.find('?') {
        output.push_str(&remainder[..=query_start]);
        let query_end = remainder[query_start + 1..]
            .find('#')
            .map(|offset| query_start + 1 + offset)
            .unwrap_or(remainder.len());
        output.push_str(&redact_query(&remainder[query_start + 1..query_end]));
        output.push_str(&remainder[query_end..]);
    } else {
        output.push_str(&remainder);
    }

    output
}

fn redact_http_path(scheme: &str, remainder: &str) -> String {
    if !scheme.eq_ignore_ascii_case("http") && !scheme.eq_ignore_ascii_case("https") {
        return remainder.to_owned();
    }
    let path_end = remainder.find(['?', '#']).unwrap_or(remainder.len());
    let path = &remainder[..path_end];
    if path.is_empty() || path == "/" {
        return remainder.to_owned();
    }
    format!("/••••••{}", &remainder[path_end..])
}

fn redact_query(query: &str) -> String {
    query
        .split('&')
        .map(|pair| {
            let Some((key, value)) = pair.split_once('=') else {
                return pair.to_owned();
            };
            if SENSITIVE_KEYS
                .iter()
                .any(|candidate| key.eq_ignore_ascii_case(candidate))
            {
                format!("{key}={URI_MASK}")
            } else {
                format!("{key}={value}")
            }
        })
        .collect::<Vec<_>>()
        .join("&")
}
