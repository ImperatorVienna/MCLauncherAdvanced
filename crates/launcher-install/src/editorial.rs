//! Read-only Mojang editorial feeds. No UI, account policy or OS integration.
use crate::Result;
use std::{io::Read, time::Duration};
const BASE: &str = "https://launchercontent.mojang.com/v2/";
const LIMIT: u64 = 8 * 1024 * 1024;
pub fn valid_path(path: &str) -> bool {
    path == "news.json" || path == "javaPatchNotes.json" ||
    path.strip_prefix("javaPatchNotes/").and_then(|s|s.strip_suffix(".json"))
        .is_some_and(|s| !s.is_empty() && s.len() <= 160 && s.bytes().all(|b|b.is_ascii_alphanumeric() || b == b'-' || b == b'_'))
}
pub fn fetch(path: &str) -> Result<serde_json::Value> {
    if !valid_path(path) { return Err("Invalid official content path".into()); }
    let client = reqwest::blocking::Client::builder().timeout(Duration::from_secs(20))
        .redirect(reqwest::redirect::Policy::none()).user_agent("MCLauncherAdvanced/1.0 (official content reader)")
        .build().map_err(crate::err)?;
    let response = client.get(format!("{BASE}{path}")).send().map_err(crate::err)?;
    if !response.status().is_success() { return Err("Official content is temporarily unavailable".into()); }
    if response.content_length().is_some_and(|n|n > LIMIT) { return Err("Official content is too large".into()); }
    let mut bytes = Vec::new(); response.take(LIMIT + 1).read_to_end(&mut bytes).map_err(crate::err)?;
    if bytes.len() as u64 > LIMIT { return Err("Official content is too large".into()); }
    serde_json::from_slice(&bytes).map_err(|_|"Invalid official content response".into())
}
pub fn allowed_link(url: &str) -> bool {
    let Ok(u) = reqwest::Url::parse(url) else {return false};
    if u.scheme() != "https" || !u.username().is_empty() || u.password().is_some() || u.port().is_some() {return false;}
    match u.host_str() {
        Some("www.minecraft.net") => true,
        Some("github.com") => u.path() == "/ImperatorVienna/MCLauncherAdvanced" || u.path().starts_with("/ImperatorVienna/MCLauncherAdvanced/"),
        _ => false,
    }
}
#[cfg(test)] mod tests {
    use super::*;
    #[test]
    #[ignore = "Requires network access to Mojang"]
    fn official_live_feeds() {
        let news=fetch("news.json").unwrap();
        assert!(news["entries"].as_array().is_some_and(|v|!v.is_empty()));
        let index=fetch("javaPatchNotes.json").unwrap();
        let first=&index["entries"][0];
        let detail=fetch(first["contentPath"].as_str().unwrap()).unwrap();
        assert_eq!(detail["version"],first["version"]);
        assert!(detail["body"].as_str().is_some_and(|s|!s.is_empty()));
    }
    #[test] fn remote_paths_and_links_are_bounded() {
        for p in ["news.json","javaPatchNotes.json","javaPatchNotes/26-4-snapshot-4.json"] {assert!(valid_path(p));}
        for p in ["../news.json","javaPatchNotes/../secret.json","javaPatchNotes/a/b.json","https://example.com","javaPatchNotes/a.json?x=1"] {assert!(!valid_path(p));}
        assert!(allowed_link("https://www.minecraft.net/en-us/article/example"));
        assert!(allowed_link("https://github.com/ImperatorVienna/MCLauncherAdvanced"));
        for u in ["file:///tmp/a","https://www.minecraft.net.evil.com/","https://user@www.minecraft.net/","https://github.com/ImperatorVienna/MCLauncherAdvanced-evil","https://www.minecraft.net:444/"] {assert!(!allowed_link(u));}
    }
}
