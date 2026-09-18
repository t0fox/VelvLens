#[cfg(windows)]
mod windows_dpapi;

#[cfg(windows)]
pub use windows_dpapi::{protect, unprotect};

#[cfg(not(windows))]
pub fn protect(_data: &[u8]) -> Result<Vec<u8>, String> {
    Err("protected history is only available on Windows".to_owned())
}

#[cfg(not(windows))]
pub fn unprotect(_data: &[u8]) -> Result<Vec<u8>, String> {
    Err("protected history is only available on Windows".to_owned())
}
