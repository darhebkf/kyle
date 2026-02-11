use crate::settings;
use anyhow::{Context, Result};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const REPO: &str = "darhebkf/kyle";
const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug)]
struct Release {
    tag_name: String,
    assets: Vec<Asset>,
}

#[derive(Debug)]
struct Asset {
    name: String,
    browser_download_url: String,
}

/// Check for auto-upgrade if enabled in settings.
/// Runs silently, logs errors to stderr but doesn't fail the main command.
pub fn check_auto_upgrade() {
    if !settings::get().auto_upgrade {
        return;
    }

    if let Err(e) = do_upgrade(false) {
        eprintln!("Auto-upgrade failed: {e}");
    }
}

pub fn run() -> Result<()> {
    do_upgrade(true)
}

fn do_upgrade(verbose: bool) -> Result<()> {
    if verbose {
        println!("Checking for updates...");
    }

    let current = VERSION.trim_start_matches('v');
    let latest = get_latest_release()?;
    let latest_version = latest.tag_name.trim_start_matches('v');

    if current == latest_version {
        if verbose {
            println!("Already up to date (v{current})");
        }
        return Ok(());
    }

    if verbose {
        println!("New version available: v{latest_version} (current: v{current})");
    } else {
        eprintln!("Auto-upgrading to v{latest_version}...");
    }

    let target = get_target();
    let asset = latest
        .assets
        .iter()
        .find(|a| a.name.contains(&target))
        .with_context(|| format!("No binary found for target: {target}"))?;

    if verbose {
        println!("Downloading {}...", asset.name);
    }

    let tmp_dir = env::temp_dir();
    let tmp_path = tmp_dir.join(&asset.name);
    download_file(&asset.browser_download_url, &tmp_path)?;

    if settings::get().verify_updates {
        if verbose {
            println!("Verifying checksum...");
        }
        verify_sha256(&tmp_path, &asset.name, &latest.tag_name)?;
    }

    let binary_path = extract_binary(&tmp_path, &tmp_dir)?;
    let current_exe = env::current_exe().context("Failed to get current executable path")?;
    replace_binary(&binary_path, &current_exe)?;

    // Cleanup
    fs::remove_file(&tmp_path).ok();
    fs::remove_file(&binary_path).ok();

    if verbose {
        println!("✓ Updated to v{latest_version}");
    } else {
        eprintln!("✓ Auto-upgraded to v{latest_version}");
    }

    Ok(())
}

fn get_latest_release() -> Result<Release> {
    let url = format!("https://api.github.com/repos/{REPO}/releases/latest");

    let output = std::process::Command::new("curl")
        .args(["-fsSL", "-H", "Accept: application/vnd.github+json", &url])
        .output()
        .context("Failed to run curl")?;

    if !output.status.success() {
        anyhow::bail!("Failed to fetch release info");
    }

    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).context("Failed to parse release JSON")?;

    let tag_name = json["tag_name"]
        .as_str()
        .context("Missing tag_name")?
        .to_string();

    let assets = json["assets"]
        .as_array()
        .context("Missing assets")?
        .iter()
        .filter_map(|a| {
            Some(Asset {
                name: a["name"].as_str()?.to_string(),
                browser_download_url: a["browser_download_url"].as_str()?.to_string(),
            })
        })
        .collect();

    Ok(Release { tag_name, assets })
}

fn get_target() -> String {
    let os = env::consts::OS;
    let arch = env::consts::ARCH;

    match (os, arch) {
        ("linux", "x86_64") => "x86_64-unknown-linux-musl".to_string(),
        ("linux", "aarch64") => "aarch64-unknown-linux-musl".to_string(),
        ("macos", "x86_64") => "x86_64-apple-darwin".to_string(),
        ("macos", "aarch64") => "aarch64-apple-darwin".to_string(),
        ("windows", "x86_64") => "x86_64-pc-windows-msvc".to_string(),
        _ => format!("{}-{}", arch, os),
    }
}

fn download_file(url: &str, path: &Path) -> Result<()> {
    let path_str = path.to_str().context("path contains invalid UTF-8")?;
    let output = std::process::Command::new("curl")
        .args(["-fsSL", "-o", path_str, url])
        .output()
        .context("Failed to run curl")?;

    if !output.status.success() {
        anyhow::bail!("Download failed");
    }

    Ok(())
}

fn extract_binary(archive: &Path, dest: &Path) -> Result<PathBuf> {
    let archive_str = archive
        .to_str()
        .context("archive path contains invalid UTF-8")?;
    let dest_str = dest
        .to_str()
        .context("destination path contains invalid UTF-8")?;

    if archive_str.ends_with(".tar.gz") {
        // Unix: extract tar.gz
        let output = std::process::Command::new("tar")
            .args(["-xzf", archive_str, "-C", dest_str])
            .output()
            .context("Failed to extract archive")?;

        if !output.status.success() {
            anyhow::bail!("Extraction failed");
        }

        Ok(dest.join("kyle"))
    } else if archive_str.ends_with(".zip") {
        // Windows: extract zip
        #[cfg(windows)]
        {
            let output = std::process::Command::new("powershell")
                .args([
                    "-Command",
                    &format!(
                        "Expand-Archive -Path '{}' -DestinationPath '{}' -Force",
                        archive_str, dest_str
                    ),
                ])
                .output()
                .context("Failed to extract archive")?;

            if !output.status.success() {
                anyhow::bail!("Extraction failed");
            }
        }

        Ok(dest.join("kyle.exe"))
    } else {
        anyhow::bail!("Unknown archive format: {}", archive_str);
    }
}

fn verify_sha256(archive_path: &Path, asset_name: &str, version: &str) -> Result<()> {
    let url = format!("https://github.com/{REPO}/releases/download/{version}/SHA256SUMS",);

    let output = std::process::Command::new("curl")
        .args(["-fsSL", &url])
        .output()
        .context("failed to download SHA256SUMS")?;

    if !output.status.success() {
        anyhow::bail!("failed to download SHA256SUMS from release");
    }

    let checksums = String::from_utf8_lossy(&output.stdout);

    let expected = checksums
        .lines()
        .find(|line| line.contains(asset_name))
        .context("asset not found in SHA256SUMS")?
        .split_whitespace()
        .next()
        .context("invalid SHA256SUMS format")?;

    let actual = compute_sha256(archive_path)?;

    if expected != actual {
        anyhow::bail!("SHA256 mismatch: expected {expected}, got {actual}");
    }

    Ok(())
}

fn compute_sha256(path: &Path) -> Result<String> {
    #[cfg(unix)]
    {
        let path_str = path.to_str().context("path contains invalid UTF-8")?;
        let output = std::process::Command::new("sha256sum")
            .arg(path_str)
            .output()
            .context("failed to compute SHA256")?;
        let hash = String::from_utf8_lossy(&output.stdout);
        Ok(hash.split_whitespace().next().unwrap_or("").to_string())
    }

    #[cfg(windows)]
    {
        let output = std::process::Command::new("powershell")
            .args([
                "-Command",
                &format!(
                    "(Get-FileHash -Algorithm SHA256 '{}').Hash.ToLower()",
                    path.display()
                ),
            ])
            .output()
            .context("failed to compute SHA256")?;
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }
}

fn replace_binary(new_binary: &Path, current_exe: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        // Unix: simple rename/move
        fs::copy(new_binary, current_exe).context("Failed to replace binary")?;

        // Make executable
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(current_exe)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(current_exe, perms)?;
    }

    #[cfg(windows)]
    {
        // Windows: rename current to .old, copy new, delete old
        let old_exe = current_exe.with_extension("exe.old");
        let _ = fs::remove_file(&old_exe); // Remove any previous .old file
        fs::rename(current_exe, &old_exe).context("Failed to rename current binary")?;
        fs::copy(new_binary, current_exe).context("Failed to copy new binary")?;
        // Old file will be deleted on next run or manually
    }

    Ok(())
}
