export function isAllowedLocalUrl(candidate, origin) {
  try {
    const url = new URL(candidate);
    return url.protocol === 'http:' && url.hostname === '127.0.0.1' && url.origin === origin;
  } catch {
    return false;
  }
}

export function isExternalHttpUrl(candidate) {
  try {
    const url = new URL(candidate);
    return url.protocol === 'http:' || url.protocol === 'https:';
  } catch {
    return false;
  }
}
