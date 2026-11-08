//! Downloads a new executable, checks it against its manifest and puts it in
//! place of the running one. The running process is untouched: the new build
//! starts at the next launch (ADR-0010).

use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

use super::manifest::Manifest;
use super::state::UpdateError;

/// Downloads, verifies and installs the build `manifest` describes.
pub async fn install(client: &reqwest::Client, manifest: &Manifest) -> Result<(), UpdateError> {
    let data = download(client, &manifest.url).await?;
    verify(&data, manifest)?;
    let exe = std::env::current_exe().map_err(|error| {
        tracing::warn!(%error, "cannot locate the running executable");
        UpdateError::CantWrite
    })?;
    tokio::task::spawn_blocking(move || swap(&exe, &data))
        .await
        .map_err(|error| {
            tracing::warn!(%error, "executable swap did not finish");
            UpdateError::CantWrite
        })?
}

/// Body of `url`, or why it could not be read.
pub async fn download(client: &reqwest::Client, url: &str) -> Result<Vec<u8>, UpdateError> {
    let response = client.get(url).send().await.map_err(|error| {
        tracing::warn!(%error, url, "update request failed");
        UpdateError::NoConnection
    })?;
    let status = response.status();
    if status == reqwest::StatusCode::NOT_FOUND {
        tracing::warn!(url, "nothing published at the update URL");
        return Err(UpdateError::NotPublished);
    }
    if !status.is_success() {
        tracing::warn!(%status, url, "update server error");
        return Err(UpdateError::NoConnection);
    }
    let body = response.bytes().await.map_err(|error| {
        tracing::warn!(%error, url, "update download interrupted");
        UpdateError::Incomplete
    })?;
    Ok(body.to_vec())
}

/// The download has the size and SHA-256 the manifest announces.
pub fn verify(data: &[u8], manifest: &Manifest) -> Result<(), UpdateError> {
    let size_matches = u64::try_from(data.len()).is_ok_and(|len| len == manifest.size);
    let checksum = sha256_hex(data);
    if size_matches && checksum.eq_ignore_ascii_case(&manifest.sha256) {
        Ok(())
    } else {
        tracing::warn!(
            size = data.len(),
            expected_size = manifest.size,
            %checksum,
            expected = %manifest.sha256,
            "update does not match its manifest"
        );
        Err(UpdateError::Incomplete)
    }
}

/// Writes the new executable next to the running one, then swaps them. On
/// Windows `self-replace` renames the running file, which Windows allows.
fn swap(exe: &Path, data: &[u8]) -> Result<(), UpdateError> {
    let staged = stage(exe, data)?;
    let result = self_replace::self_replace(&staged).map_err(|error| {
        tracing::warn!(%error, exe = %exe.display(), "cannot replace the executable");
        UpdateError::CantWrite
    });
    // `self_replace` copies the staged file: remove it either way.
    if let Err(error) = std::fs::remove_file(&staged) {
        tracing::debug!(%error, "staged update not removed");
    }
    result
}

/// Writes `data` beside `exe`, which also proves the folder is writable.
fn stage(exe: &Path, data: &[u8]) -> Result<PathBuf, UpdateError> {
    let staged = staged_path(exe);
    std::fs::write(&staged, data).map_err(|error| {
        tracing::warn!(%error, path = %staged.display(), "cannot write the update");
        UpdateError::CantWrite
    })?;
    Ok(staged)
}

/// `gymdiff.exe` → `gymdiff.exe.update`, in the same folder.
fn staged_path(exe: &Path) -> PathBuf {
    let mut name = exe.file_name().unwrap_or_default().to_os_string();
    name.push(".update");
    exe.with_file_name(name)
}

fn sha256_hex(data: &[u8]) -> String {
    Sha256::digest(data)
        .iter()
        .fold(String::with_capacity(64), |mut hex, byte| {
            let _ = write!(hex, "{byte:02x}");
            hex
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn manifest_for(data: &[u8]) -> Manifest {
        Manifest {
            version: "0.1.0".to_owned(),
            build: 58,
            commit: "a1b2c3d".to_owned(),
            target: "x86_64-pc-windows-msvc".to_owned(),
            url: "https://example.invalid/gymdiff.exe".to_owned(),
            size: data.len() as u64,
            sha256: sha256_hex(data),
        }
    }

    #[test]
    fn sha256_is_lower_case_hex() {
        assert_eq!(
            sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn verify_accepts_only_the_announced_file() {
        let data = b"new gymdiff";
        let manifest = manifest_for(data);
        assert_eq!(verify(data, &manifest), Ok(()));
        assert_eq!(
            verify(b"new gymdiff, truncated?", &manifest),
            Err(UpdateError::Incomplete)
        );
        let tampered = Manifest {
            sha256: sha256_hex(b"other"),
            ..manifest
        };
        assert_eq!(verify(data, &tampered), Err(UpdateError::Incomplete));
    }

    #[test]
    fn stages_next_to_the_executable() {
        let dir = tempfile::tempdir().unwrap();
        let exe = dir.path().join("gymdiff.exe");
        let staged = stage(&exe, b"new build").unwrap();
        assert_eq!(staged, dir.path().join("gymdiff.exe.update"));
        assert_eq!(std::fs::read(&staged).unwrap(), b"new build");
    }

    #[test]
    fn staging_in_a_missing_folder_cannot_write() {
        let dir = tempfile::tempdir().unwrap();
        let exe = dir.path().join("missing").join("gymdiff.exe");
        assert_eq!(stage(&exe, b"x"), Err(UpdateError::CantWrite));
    }
}
