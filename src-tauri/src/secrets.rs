use std::{mem::zeroed, ptr};

use windows_sys::{
    core::PWSTR,
    Win32::{
        Foundation::{GetLastError, ERROR_NOT_FOUND},
        Security::Credentials::{
            CredDeleteW, CredFree, CredReadW, CredWriteW, CREDENTIALW, CRED_PERSIST_LOCAL_MACHINE,
            CRED_TYPE_GENERIC,
        },
    },
};

use crate::models::StoredTokens;

const TARGET: &str = "MusicMonitor/SpotifyOAuth";

fn wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(std::iter::once(0)).collect()
}

/// Stores the token document in Windows Credential Manager. All raw pointer use
/// is confined to this module; buffers remain alive for each Win32 call.
pub fn write(tokens: &StoredTokens) -> Result<(), String> {
    let secret = serde_json::to_vec(tokens)
        .map_err(|error| format!("Could not serialize Spotify credentials: {error}"))?;
    let mut target = wide(TARGET);
    let mut username = wide("Spotify OAuth tokens");

    // SAFETY: CREDENTIALW points only to the owned buffers above, which outlive
    // CredWriteW. The API copies the data before returning.
    let mut credential: CREDENTIALW = unsafe { zeroed() };
    credential.Type = CRED_TYPE_GENERIC;
    credential.TargetName = target.as_mut_ptr();
    credential.CredentialBlobSize = u32::try_from(secret.len())
        .map_err(|_| "Spotify credential document is too large".to_owned())?;
    credential.CredentialBlob = secret.as_ptr().cast_mut();
    credential.Persist = CRED_PERSIST_LOCAL_MACHINE;
    credential.UserName = username.as_mut_ptr();

    // SAFETY: The structure and its buffers are valid for the duration of the call.
    let written = unsafe { CredWriteW(&credential, 0) };
    if written == 0 {
        return Err(format!(
            "Windows Credential Manager rejected the Spotify credentials (error {})",
            unsafe { GetLastError() }
        ));
    }
    Ok(())
}

pub fn read() -> Result<Option<StoredTokens>, String> {
    let target = wide(TARGET);
    let mut raw: *mut CREDENTIALW = ptr::null_mut();

    // SAFETY: target is a valid NUL-terminated string and raw is an out pointer.
    let found = unsafe { CredReadW(target.as_ptr(), CRED_TYPE_GENERIC, 0, &mut raw) };
    if found == 0 {
        let error = unsafe { GetLastError() };
        return if error == ERROR_NOT_FOUND {
            Ok(None)
        } else {
            Err(format!(
                "Windows Credential Manager could not read Spotify credentials (error {error})"
            ))
        };
    }

    // SAFETY: CredReadW returned one allocated CREDENTIALW block. We copy the
    // blob before releasing that block exactly once with CredFree.
    let parsed = unsafe {
        let credential = &*raw;
        let blob = std::slice::from_raw_parts(
            credential.CredentialBlob,
            credential.CredentialBlobSize as usize,
        );
        let result = serde_json::from_slice::<StoredTokens>(blob)
            .map_err(|error| format!("Stored Spotify credentials are invalid: {error}"));
        CredFree(raw.cast());
        result
    }?;

    Ok(Some(parsed))
}

pub fn delete() -> Result<(), String> {
    let target = wide(TARGET);
    // SAFETY: target is a valid NUL-terminated string for the duration of the call.
    let deleted = unsafe { CredDeleteW(target.as_ptr(), CRED_TYPE_GENERIC, 0) };
    if deleted == 0 {
        let error = unsafe { GetLastError() };
        if error != ERROR_NOT_FOUND {
            return Err(format!(
                "Windows Credential Manager could not delete Spotify credentials (error {error})"
            ));
        }
    }
    Ok(())
}

#[allow(dead_code)]
fn _assert_pwstr_compatibility(_: PWSTR) {}
