use anyhow::{anyhow, Result};
use std::io::Write;

const REPO: &str = "TheHomelessTwig/newc-rs";

pub fn current_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

fn platform_asset() -> Option<&'static str> {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("linux",   "x86_64")  => Some("newc-x86_64-linux"),
        ("linux",   "aarch64") => Some("newc-aarch64-linux"),
        ("macos",   "x86_64")  => Some("newc-x86_64-macos"),
        ("macos",   "aarch64") => Some("newc-aarch64-macos"),
        ("windows", "x86_64")  => Some("newc-x86_64-windows.exe"),
        _                      => None,
    }
}

fn semver_gt(a: &str, b: &str) -> bool {
    let parse = |v: &str| -> Vec<u64> {
        v.split('.').filter_map(|p| p.parse().ok()).collect()
    };
    parse(a) > parse(b)
}

fn fetch_latest_tag() -> Result<String> {
    let url = format!("https://api.github.com/repos/{REPO}/releases/latest");
    let body: serde_json::Value = ureq::get(&url)
        .set("User-Agent", &format!("newc/{}", current_version()))
        .set("Accept", "application/vnd.github.v3+json")
        .call()?
        .into_json()?;

    body["tag_name"]
        .as_str()
        .map(|t| t.to_string())
        .ok_or_else(|| anyhow!("GitHub API returned no tag_name"))
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

    let download_url = format!(
        "https://github.com/{REPO}/releases/download/{tag}/{asset}"
    );

    print!("Downloading {asset}... ");
    std::io::stdout().flush()?;

    let response = ureq::get(&download_url)
        .set("User-Agent", &format!("newc/{current}"))
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

    let current_exe = std::env::current_exe()?;
    print!("Installing to {}... ", current_exe.display());
    std::io::stdout().flush()?;

    match std::fs::copy(&tmp, &current_exe) {
        Ok(_) => {
            println!("done.");
        }
        Err(_) => {
            // System install — escalate with sudo
            println!("(needs sudo)");
            let ok = std::process::Command::new("sudo")
                .args(["cp", tmp.to_str().unwrap(), current_exe.to_str().unwrap()])
                .status()
                .map(|s| s.success())
                .unwrap_or(false);
            if !ok {
                let _ = std::fs::remove_file(&tmp);
                return Err(anyhow!(
                    "Install failed. Run manually:\n  sudo cp {} {}",
                    tmp.display(),
                    current_exe.display()
                ));
            }
        }
    }

    let _ = std::fs::remove_file(&tmp);
    println!("newc updated to v{latest}.");
    Ok(())
}
