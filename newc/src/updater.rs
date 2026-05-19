//! Self-update logic for the `newc` binary.
//!
//! [`check`] queries the GitHub Releases API for the latest tag and returns the
//! version string if it is newer than the running binary. [`update`] performs
//! the full download-and-install flow.
//!
//! Platform detection (`platform_asset`) maps `(OS, ARCH)` pairs to the
//! pre-built asset filenames published with each GitHub release. On non-Windows
//! systems `install_binary` falls back to `sudo cp` if a direct write fails
//! (e.g. the binary lives in `/usr/local/bin`). On Windows the replacement is
//! deferred to a batch script that runs after the process exits.

use anyhow::{anyhow, Result};
use std::io::Write;
use std::path::Path;
use std::time::Duration;

const REPO: &str = "TheHomelessTwig/newc-rs";

/// Return the version of the currently running binary (from `CARGO_PKG_VERSION`).
pub fn current_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

fn platform_asset() -> Option<&'static str> {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("linux",   "x86_64")  => Some("newc-x86_64-linux"),
        ("linux",   "aarch64") => Some("newc-aarch64-linux"),
        ("macos",   "aarch64") => Some("newc-aarch64-macos"),
        ("windows", "x86_64")  => Some("newc-x86_64-windows.exe"),
        _                      => None,
    }
}

pub(crate) fn semver_gt(a: &str, b: &str) -> bool {
    let parse = |v: &str| -> Vec<u64> {
        v.split('.').filter_map(|p| p.parse().ok()).collect()
    };
    parse(a) > parse(b)
}

fn agent() -> ureq::Agent {
    ureq::AgentBuilder::new()
        .timeout(Duration::from_secs(15))
        .build()
}

fn fetch_latest_tag() -> Result<String> {
    let url = format!("https://api.github.com/repos/{REPO}/releases/latest");
    let body: serde_json::Value = agent()
        .get(&url)
        .set("User-Agent", &format!("newc/{}", current_version()))
        .set("Accept", "application/vnd.github.v3+json")
        .call()?
        .into_json()?;

    body["tag_name"]
        .as_str()
        .map(|t| t.to_string())
        .ok_or_else(|| anyhow!("GitHub API returned no tag_name"))
}

fn download_asset(tag: &str, asset: &str) -> Result<std::path::PathBuf> {
    let url = format!("https://github.com/{REPO}/releases/download/{tag}/{asset}");
    print!("Downloading {asset}... ");
    std::io::stdout().flush()?;

    let response = agent()
        .get(&url)
        .set("User-Agent", &format!("newc/{}", current_version()))
        .call()?;

    let tmp = std::env::temp_dir().join("newc-update");
    {
        let mut file = std::fs::File::create(&tmp)?;
        std::io::copy(&mut response.into_reader(), &mut file)?;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&tmp, std::fs::Permissions::from_mode(0o755))?;
    }
    println!("done.");
    Ok(tmp)
}

fn install_binary(tmp: &Path, target: &Path) -> Result<()> {
    print!("Installing to {}... ", target.display());
    std::io::stdout().flush()?;

    #[cfg(windows)]
    {
        if std::fs::copy(tmp, target).is_ok() {
            println!("done.");
            return Ok(());
        }
        return install_windows_deferred(tmp, target);
    }

    #[cfg(not(windows))]
    match std::fs::copy(tmp, target) {
        Ok(_) => {
            println!("done.");
            Ok(())
        }
        Err(_) => {
            println!("(needs sudo)");
            let ok = std::process::Command::new("sudo")
                .arg("cp")
                .arg(tmp)
                .arg(target)
                .status()
                .map(|s| s.success())
                .unwrap_or(false);
            if ok {
                Ok(())
            } else {
                Err(anyhow!(
                    "Install failed. Run manually:\n  sudo cp {} {}",
                    tmp.display(),
                    target.display()
                ))
            }
        }
    }
}

#[cfg(windows)]
fn install_windows_deferred(tmp: &Path, target: &Path) -> Result<()> {
    use std::os::windows::process::CommandExt;
    const DETACHED_PROCESS: u32 = 0x00000008;
    const CREATE_NEW_PROCESS_GROUP: u32 = 0x00000200;

    let bat = std::env::temp_dir().join("newc-finish-update.bat");
    let script = format!(
        "@echo off\r\ntimeout /t 1 /nobreak >nul\r\ncopy /y \"{src}\" \"{dst}\"\r\ndel \"{src}\"\r\ndel \"%~f0\"\r\n",
        src = tmp.display(),
        dst = target.display(),
    );
    std::fs::write(&bat, script.as_bytes())?;
    std::process::Command::new("cmd")
        .arg("/c")
        .arg(&bat)
        .creation_flags(DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP)
        .spawn()?;
    println!("done.\nUpdate installs after newc exits.");
    Ok(())
}

/// Returns Some(latest_version) if a newer release exists, None if up-to-date.
pub fn check() -> Result<Option<String>> {
    let tag = fetch_latest_tag()?;
    let latest = tag.trim_start_matches('v').to_string();
    if semver_gt(&latest, current_version()) {
        Ok(Some(latest))
    } else {
        Ok(None)
    }
}

/// Downloads and installs the latest release binary if one is available.
pub fn update() -> Result<()> {
    let current = current_version();
    print!("Checking for updates (current: v{current})... ");
    std::io::stdout().flush()?;

    let tag = fetch_latest_tag()?;
    let latest = tag.trim_start_matches('v');

    if !semver_gt(latest, current) {
        println!("already up to date.");
        return Ok(());
    }
    println!("v{latest} available.");

    let asset = platform_asset().ok_or_else(|| {
        anyhow!(
            "No pre-built binary for {}-{}.\nBuild from source: https://github.com/{REPO}",
            std::env::consts::OS,
            std::env::consts::ARCH
        )
    })?;

    let current_exe = std::env::current_exe()?;
    let tmp = download_asset(&tag, asset)?;
    install_binary(&tmp, &current_exe)?;
    let _ = std::fs::remove_file(&tmp);
    println!("newc updated to v{latest}.");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::semver_gt;

    #[test]
    fn newer_patch() { assert!(semver_gt("0.2.9", "0.2.8")); }
    #[test]
    fn newer_minor() { assert!(semver_gt("0.3.0", "0.2.9")); }
    #[test]
    fn newer_major() { assert!(semver_gt("1.0.0", "0.9.9")); }
    #[test]
    fn same_version_not_newer() { assert!(!semver_gt("0.2.9", "0.2.9")); }
    #[test]
    fn older_version_not_newer() { assert!(!semver_gt("0.2.8", "0.2.9")); }
}
