//! Release discovery and verification; executable replacement belongs to the host adapter.
use crate::{download, err, Result};
use semver::Version;
use serde::{Deserialize, Serialize};
use std::io::Read;

const RELEASES: &str = "https://api.github.com/repos/ImperatorVienna/MCLauncherAdvanced/releases/latest";
const PREFIX: &str = "https://github.com/ImperatorVienna/MCLauncherAdvanced/releases/download/";
#[derive(Debug, Deserialize)]
pub struct Release {
    tag_name: String,
    draft: bool,
    prerelease: bool,
    assets: Vec<Asset>,
}
#[derive(Debug, Deserialize)]
struct Asset { name: String, browser_download_url: String, size: u64, state: String, digest: Option<String> }
#[derive(Clone, Debug, Serialize)]
pub struct Update { pub version: String, pub file: download::File }

pub fn select(release: Release, current: &str, target: &str) -> Result<Option<Update>> {
    let current = Version::parse(current).map_err(|_| "This build does not support stable updates")?;
    // Never silently move experimental builds into the public stable channel.
    if !current.pre.is_empty() { return Err("This build does not support stable updates".into()); }
    let version = Version::parse(release.tag_name.trim_start_matches('v')).map_err(|_| "Invalid release version")?;
    if release.draft || release.prerelease || !version.pre.is_empty() || version <= current { return Ok(None); }
    let extension = match target { "linux-x86_64" => "AppImage", "win-x86_64" => "exe", _ => return Err("Launcher updates are unavailable for this platform".into()) };
    let name = format!("mclauncher-advanced_{version}-{target}.{extension}");
    let mut assets = release.assets.into_iter().filter(|a| a.name == name);
    let asset = assets.next().ok_or("The release has no package for this platform")?;
    if assets.next().is_some() || asset.state != "uploaded" { return Err("Invalid release package".into()); }
    let expected_url = format!("{PREFIX}{}/{name}", release.tag_name);
    if asset.browser_download_url != expected_url { return Err("Invalid release download URL".into()); }
    let hash = asset.digest.as_deref().and_then(|d|d.strip_prefix("sha256:")).ok_or("The release package has no SHA-256 digest")?.to_owned();
    if hash.len() != 64 { return Err("Invalid release checksum".into()); }
    let file = download::File { path: name, url: asset.browser_download_url, hash, size: asset.size };
    file.validate()?;
    Ok(Some(Update {version:version.to_string(),file}))
}

/// A 404 on latest means no accessible stable release, not that the installed version is latest.
pub fn check(current: &str, target: &str) -> Result<(bool, Option<Update>)> {
    let client = download::client()?;
    let response = client.get(RELEASES).header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28").send().map_err(err)?;
    if response.status() == reqwest::StatusCode::NOT_FOUND { return Ok((false,None)); }
    let mut bytes = Vec::new();
    response.error_for_status().map_err(err)?.take(2_000_001).read_to_end(&mut bytes).map_err(err)?;
    if bytes.len() > 2_000_000 { return Err("Release metadata exceeds the size limit".into()); }
    let release: Release = serde_json::from_slice(&bytes).map_err(err)?;
    Ok((true,select(release,current,target)?))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn release(version: &str, target: &str, extension: &str) -> Release {
        let name = format!("mclauncher-advanced_{version}-{target}.{extension}");
        serde_json::from_value(serde_json::json!({"tag_name":format!("v{version}"),"draft":false,"prerelease":false,"assets":[{"name":name,"state":"uploaded","size":123,"digest":format!("sha256:{}","a".repeat(64)),"browser_download_url":format!("{PREFIX}v{version}/{name}")}]})).unwrap()
    }
    #[test] fn stable_versions_and_exact_target() {
        assert!(select(release("1.10.0","linux-x86_64","AppImage"),"1.9.0","linux-x86_64").unwrap().is_some());
        assert!(select(release("1.0.0","linux-x86_64","AppImage"),"1.0.0","linux-x86_64").unwrap().is_none());
        assert!(select(release("1.0.0","linux-x86_64","AppImage"),"2.0.0","linux-x86_64").unwrap().is_none());
        assert!(select(release("1.0.0","win-x86_64","exe"),"0.1.0","win-x86_64").unwrap().is_some());
        assert!(select(release("1.0.0","win-x86_64","exe"),"0.1.0","linux-x86_64").is_err());
    }
    #[test] fn untrusted_and_incomplete_assets_rejected() {
        let mut r=release("1.0.0","win-x86_64","exe"); r.assets[0].digest=None;
        assert!(select(r,"0.1.0","win-x86_64").is_err());
        let mut r=release("1.0.0","win-x86_64","exe"); r.assets[0].browser_download_url="https://github.com/other/repo/file".into();
        assert!(select(r,"0.1.0","win-x86_64").is_err());
        let mut r=release("1.0.0","win-x86_64","exe"); r.prerelease=true;
        assert!(select(r,"0.1.0","win-x86_64").unwrap().is_none());
        assert!(select(release("1.0.0","win-x86_64","exe"),"1.0.0o","win-x86_64").is_err());
    }
}
