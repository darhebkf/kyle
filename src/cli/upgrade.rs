use crate::settings;
use anyhow::{Context, Result};
use std::env;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

const REPO: &str = "darhebkf/kyle";
const VERSION: &str = env!("CARGO_PKG_VERSION");
const CHECK_INTERVAL_SECS: u64 = 24 * 60 * 60;

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

pub fn check_auto_upgrade() {
    if !settings::get().auto_upgrade {
        return;
    }
    if !needs_check(now_unix()) {
        return;
    }
    write_stamp(now_unix());

    let Ok(current_exe) = env::current_exe() else {
        return;
    };
    let _ = Command::new(current_exe)
        .arg("--upgrade-check")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();
}

pub fn background_check() {
    let result = do_upgrade(false);
    let line = match result {
        Ok(()) => "ok".to_string(),
        Err(e) => format!("failed: {e}"),
    };
    append_log(&line);
}

pub fn run() -> Result<()> {
    let result = do_upgrade(true);
    if result.is_ok() {
        write_stamp(now_unix());
    }
    result
}

pub fn print_status() -> Result<()> {
    match recent_log_lines(10) {
        Some(lines) if !lines.is_empty() => {
            println!("Recent auto-upgrade activity:");
            for line in lines {
                println!("  {line}");
            }
        }
        _ => println!("No auto-upgrade activity recorded yet."),
    }
    if let Some(last) = read_stamp() {
        let age = now_unix().saturating_sub(last);
        println!("\nLast check: {} ago", format_age(age));
    } else {
        println!("\nLast check: never");
    }
    Ok(())
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
        _ => format!("{arch}-{os}"),
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
        anyhow::bail!("Unknown archive format: {archive_str}");
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

fn state_dir() -> Option<PathBuf> {
    if let Some(dir) = env::var_os("KYLE_STATE_DIR") {
        return Some(PathBuf::from(dir));
    }
    let base = env::var_os("XDG_STATE_HOME")
        .map(PathBuf::from)
        .or_else(|| dirs::home_dir().map(|h| h.join(".local").join("state")))?;
    Some(base.join("kyle"))
}

fn stamp_path() -> Option<PathBuf> {
    Some(state_dir()?.join("upgrade-stamp"))
}

fn log_path() -> Option<PathBuf> {
    Some(state_dir()?.join("upgrade.log"))
}

fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn read_stamp() -> Option<u64> {
    let path = stamp_path()?;
    let content = fs::read_to_string(path).ok()?;
    content.trim().parse().ok()
}

fn write_stamp(ts: u64) {
    let Some(path) = stamp_path() else { return };
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let _ = fs::write(path, ts.to_string());
}

fn interval_elapsed(last: u64, now: u64, interval: u64) -> bool {
    now.saturating_sub(last) >= interval
}

fn needs_check(now: u64) -> bool {
    match read_stamp() {
        None => true,
        Some(last) => interval_elapsed(last, now, CHECK_INTERVAL_SECS),
    }
}

fn append_log(msg: &str) {
    let Some(path) = log_path() else { return };
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let line = format!("[{}] {msg}\n", now_unix());
    if let Ok(mut f) = fs::OpenOptions::new().create(true).append(true).open(path) {
        let _ = f.write_all(line.as_bytes());
    }
}

fn recent_log_lines(n: usize) -> Option<Vec<String>> {
    let path = log_path()?;
    let content = fs::read_to_string(path).ok()?;
    let mut lines: Vec<String> = content
        .lines()
        .filter(|l| !l.is_empty())
        .map(|l| l.to_string())
        .collect();
    let len = lines.len();
    if len > n {
        lines.drain(0..len - n);
    }
    Some(lines)
}

fn format_age(seconds: u64) -> String {
    if seconds < 60 {
        format!("{seconds}s")
    } else if seconds < 3600 {
        format!("{}m", seconds / 60)
    } else if seconds < 86400 {
        format!("{}h", seconds / 3600)
    } else {
        format!("{}d", seconds / 86400)
    }
}

fn replace_binary(new_binary: &Path, current_exe: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        // On Linux, writing to a running executable fails with ETXTBSY.
        // Remove the old file first (unlinks the inode while the process keeps its handle),
        // then copy the new binary to the now-free path.
        fs::remove_file(current_exe).ok();
        fs::copy(new_binary, current_exe).context("Failed to replace binary")?;

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;
    use tempfile::TempDir;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    struct StateGuard {
        _lock: std::sync::MutexGuard<'static, ()>,
        _temp: TempDir,
        prev: Option<std::ffi::OsString>,
    }

    impl StateGuard {
        fn new() -> Self {
            let lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
            let temp = TempDir::new().unwrap();
            let prev = env::var_os("KYLE_STATE_DIR");
            unsafe {
                env::set_var("KYLE_STATE_DIR", temp.path());
            }
            Self {
                _lock: lock,
                _temp: temp,
                prev,
            }
        }
    }

    impl Drop for StateGuard {
        fn drop(&mut self) {
            unsafe {
                match &self.prev {
                    Some(v) => env::set_var("KYLE_STATE_DIR", v),
                    None => env::remove_var("KYLE_STATE_DIR"),
                }
            }
        }
    }

    #[test]
    fn interval_elapsed_basic() {
        assert!(interval_elapsed(0, 100, 100));
        assert!(interval_elapsed(0, 101, 100));
        assert!(!interval_elapsed(0, 99, 100));
        assert!(!interval_elapsed(50, 100, 100));
    }

    #[test]
    fn interval_elapsed_saturates_on_clock_skew() {
        assert!(!interval_elapsed(200, 100, 50));
    }

    #[test]
    fn needs_check_when_no_stamp() {
        let _g = StateGuard::new();
        assert!(needs_check(now_unix()));
    }

    #[test]
    fn needs_check_false_when_fresh() {
        let _g = StateGuard::new();
        let now = 1_000_000;
        write_stamp(now);
        assert!(!needs_check(now));
    }

    #[test]
    fn needs_check_true_when_stale() {
        let _g = StateGuard::new();
        let old = 1_000_000;
        write_stamp(old);
        assert!(needs_check(old + CHECK_INTERVAL_SECS));
    }

    #[test]
    fn stamp_roundtrip() {
        let _g = StateGuard::new();
        assert_eq!(read_stamp(), None);
        write_stamp(42);
        assert_eq!(read_stamp(), Some(42));
        write_stamp(99);
        assert_eq!(read_stamp(), Some(99));
    }

    #[test]
    fn append_log_creates_file_and_keeps_tail() {
        let _g = StateGuard::new();
        append_log("first");
        append_log("second");
        append_log("third");
        let lines = recent_log_lines(2).unwrap();
        assert_eq!(lines.len(), 2);
        assert!(lines[0].contains("second"));
        assert!(lines[1].contains("third"));
    }

    #[test]
    fn format_age_ranges() {
        assert_eq!(format_age(5), "5s");
        assert_eq!(format_age(90), "1m");
        assert_eq!(format_age(3600 * 2), "2h");
        assert_eq!(format_age(86400 * 3), "3d");
    }
}
