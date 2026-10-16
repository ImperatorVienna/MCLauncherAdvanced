mod archives;
mod provider;
pub use archives::extract_jdk;
use crate::{
    check_cancel,
    download::{self},
    err, phase, require_platform, safe_path, Report, Result,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    fs,
    io::Read,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::atomic::AtomicBool,
    time::{Duration, Instant},
};
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Java {
    pub path: PathBuf,
    pub major: u32,
    pub version: String,
    pub source: String,
}
fn property<'a>(text: &'a str, key: &str) -> Option<&'a str> {
    text.lines().find_map(|line| {
        let (k, v) = line.trim().split_once('=')?;
        (k.trim() == key).then_some(v.trim())
    })
}
pub fn parse_probe(text: &str) -> Option<(u32, String)> {
    let value = property(text, "java.specification.version")?;
    let major = value
        .strip_prefix("1.")
        .unwrap_or(value)
        .parse::<u32>()
        .ok()?;
    if !matches!(property(text, "os.arch")?, "amd64" | "x86_64")
        || property(text, "sun.arch.data.model") != Some("64")
    {
        return None;
    }
    Some((major, property(text, "java.version")?.to_owned()))
}
pub fn probe(path: &Path, source: &str, cancel: &AtomicBool) -> Result<Option<Java>> {
    check_cancel(cancel)?;
    let Ok(path) = path.canonicalize() else {
        return Ok(None);
    };
    if !path.is_file() {
        return Ok(None);
    }
    let output = tempfile::tempfile().map_err(err)?;
    let mut command = Command::new(&path);
    command
        .args(["-XshowSettings:properties", "-version"])
        .env_remove("JAVA_TOOL_OPTIONS")
        .env_remove("_JAVA_OPTIONS")
        .env_remove("JDK_JAVA_OPTIONS")
        .stdin(Stdio::null())
        .stdout(Stdio::from(output.try_clone().map_err(err)?))
        .stderr(Stdio::from(output.try_clone().map_err(err)?));
    launcher_platform::clean_child_environment(&mut command);
    let Ok(mut child) = command.spawn() else {
        return Ok(None);
    };
    let start = Instant::now();
    loop {
        if cancel.load(std::sync::atomic::Ordering::Relaxed)
            || start.elapsed() > Duration::from_secs(5)
            || output.metadata().map_err(err)?.len() > 1024 * 1024
        {
            let _ = child.kill();
            let _ = child.wait();
            check_cancel(cancel)?;
            return Ok(None);
        }
        match child.try_wait() {
            Ok(Some(status)) => {
                if !status.success() {
                    return Ok(None);
                }
                break;
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(25)),
            Err(_) => {
                let _ = child.kill();
                let _ = child.wait();
                return Ok(None);
            }
        }
    }
    use std::io::{Seek, SeekFrom};
    let mut output = output;
    output.seek(SeekFrom::Start(0)).map_err(err)?;
    let mut text = String::new();
    output
        .take(1024 * 1024)
        .read_to_string(&mut text)
        .map_err(err)?;
    Ok(parse_probe(&text).map(|(major, version)| Java {
        path,
        major,
        version,
        source: source.into(),
    }))
}
/// Probe again immediately before using an explicitly selected runtime.
pub fn validate_selected(path: &Path, major: u32, cancel: &AtomicBool) -> Result<Java> {
    let runtime = probe(path, "selected", cancel)?
        .ok_or("Selected Java is unavailable; rescan and select Java again")?;
    if runtime.major < major {
        return Err(format!("Selected Java {} is too old; this game requires Java {major} or newer", runtime.major));
    }
    Ok(runtime)
}
fn children(root: &Path) -> Vec<PathBuf> {
    let mut paths: Vec<_> = fs::read_dir(root)
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok().map(|e| e.path()))
        .collect();
    paths.sort();
    paths
}
pub fn system_candidates() -> Vec<PathBuf> {
    let mut paths = vec![];
    if let Some(home) = std::env::var_os("JAVA_HOME") {
        paths.push(PathBuf::from(home).join("bin").join(launcher_platform::JAVA_EXECUTABLE));
    }
    if let Some(path) = std::env::var_os("PATH") {
        paths.extend(
            std::env::split_paths(&path)
                .filter(|p| p.is_absolute())
                .map(|p| p.join(launcher_platform::JAVA_EXECUTABLE)),
        );
    }
    for root in launcher_platform::java_search_roots() {
        for dir in children(&root) {
            paths.push(dir.join("bin").join(launcher_platform::JAVA_EXECUTABLE));
            paths.push(dir.join("jre/bin").join(launcher_platform::JAVA_EXECUTABLE));
        }
    }
    paths
}
/// Inventory only: never downloads or changes the system runtime.
pub fn inventory(root: &Path) -> Result<Vec<Java>> {
    let cancel = AtomicBool::new(false);
    let mut seen = HashSet::new();
    let mut found = vec![];
    let candidates = system_candidates()
        .into_iter()
        .map(|p| (p, "system"))
        .chain(
            children(root)
                .into_iter()
                .map(|p| (p.join("bin").join(launcher_platform::JAVA_EXECUTABLE), "managed")),
        );
    for (path, source) in candidates {
        let Ok(path) = path.canonicalize() else {
            continue;
        };
        if !seen.insert(path.clone()) {
            continue;
        }
        if seen.len() > 128 {
            break;
        }
        if let Ok(Some(java)) = probe(&path, source, &cancel) {
            found.push(java);
        }
    }
    found.sort_by(|a, b| b.major.cmp(&a.major).then(a.path.cmp(&b.path)));
    Ok(found)
}
pub fn find_java(
    candidates: impl IntoIterator<Item = PathBuf>,
    major: u32,
    source: &str,
    cancel: &AtomicBool,
) -> Result<Option<Java>> {
    let mut seen = HashSet::new();
    for path in candidates {
        check_cancel(cancel)?;
        let Ok(path) = path.canonicalize() else {
            continue;
        };
        if !seen.insert(path.clone()) {
            continue;
        }
        if seen.len() > 64 {
            break;
        }
        if let Some(java) = probe(&path, source, cancel)? {
            if java.major == major {
                return Ok(Some(java));
            }
        }
    }
    Ok(None)
}
/// Explicit managed download, invoked only when the player requests an installation.
pub fn install_managed_java(
    root: &Path,
    major: u32,
    cancel: &AtomicBool,
    report: &Report<'_>,
) -> Result<Java> {
    require_platform()?;
    fs::create_dir_all(root).map_err(err)?;
    if let Some(java) = find_java(
        children(root).into_iter().map(|d| d.join("bin").join(launcher_platform::JAVA_EXECUTABLE)),
        major,
        "managed",
        cancel,
    )? {
        return Ok(java);
    }
    phase(report, format!("Fetching Java {major} download metadata"));
    let client = download::client()?;
    let spec=provider::package(&client,major)?;
    let checksum=spec.hash.clone();
    spec.validate()?;
    let archive = safe_path(root, &spec.path)?;
    download::download_all(
        &client,
        root,
        &[spec],
        cancel,
        report,
        &format!("Downloading Java {major}"),
    )?;
    phase(report, format!("Extracting and verifying Java {major}"));
    let stage = tempfile::tempdir_in(root).map_err(err)?;
    archives::extract(&archive, stage.path(), launcher_platform::JAVA_ARCHIVE_EXTENSION, cancel)?;
    let directories = children(stage.path());
    let home = directories
        .iter()
        .find(|p| p.join("bin").join(launcher_platform::JAVA_EXECUTABLE).is_file())
        .ok_or("Java archive contains no executable")?;
    let java = probe(&home.join("bin").join(launcher_platform::JAVA_EXECUTABLE), "managed", cancel)?
        .filter(|j| j.major == major)
        .ok_or("Downloaded Java cannot run or has the wrong architecture or version")?;
    let version = java.version.replace(
        |c: char| !c.is_ascii_alphanumeric() && !matches!(c, '.' | '-' | '_'),
        "_",
    );
    let name = format!("temurin-{major}-{version}-{}-x64-{}", launcher_platform::OS_NAME, &checksum[..12]);
    let target = safe_path(root, &name)?;
    if target.exists() {
        if let Some(java) =
            probe(&target.join("bin").join(launcher_platform::JAVA_EXECUTABLE), "managed", cancel)?.filter(|j| j.major == major)
        {
            return Ok(java);
        }
        // Keep the previous broken runtime for diagnosis, never overlay a potentially running JVM.
        let backup = root.join(format!(
            "{name}-invalid-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(err)?
                .as_nanos()
        ));
        fs::rename(&target, backup).map_err(err)?;
    }
    check_cancel(cancel)?;
    fs::rename(home, &target).map_err(err)?;
    let _ = fs::remove_file(archive);
    probe(&target.join("bin").join(launcher_platform::JAVA_EXECUTABLE), "managed", cancel)?
        .filter(|j| j.major == major)
        .ok_or("Installed Java cannot run".into())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn detects_major_and_rejects_wrong_architecture() {
        let text="java.specification.version = 1.8\njava.version = 1.8.0_492\nos.arch = amd64\nsun.arch.data.model = 64";
        assert_eq!(parse_probe(text).unwrap().0, 8);
        assert_eq!(parse_probe(&text.replace("1.8\n", "21\n")).unwrap().0, 21);
        assert!(parse_probe(&text.replace("amd64", "aarch64")).is_none());
        assert!(parse_probe(&text.replace("= 64", "= 32")).is_none());
    }
    #[cfg(unix)]
    #[test]
    fn installed_java_is_reused_only_for_exact_major() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("java");
        fs::write(&path,"#!/bin/sh\nprintf 'java.specification.version = 17\\njava.version = 17.0.1\\nos.arch = amd64\\nsun.arch.data.model = 64\\n' >&2\n").unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
        let cancel = AtomicBool::new(false);
        assert_eq!(
            find_java([path.clone()], 17, "system", &cancel)
                .unwrap()
                .unwrap()
                .source,
            "system"
        );
        assert!(find_java([path], 21, "system", &cancel).unwrap().is_none());
    }
    #[test]
    fn extraction_rejects_traversal_and_external_links() {
        use flate2::{write::GzEncoder, Compression};
        for link in [false, true] {
            let dir = tempfile::tempdir().unwrap();
            let archive = dir.path().join("jdk.tar.gz");
            let encoder =
                GzEncoder::new(fs::File::create(&archive).unwrap(), Compression::default());
            let mut builder = tar::Builder::new(encoder);
            let mut header = tar::Header::new_gnu();
            header.set_mode(0o644);
            if link {
                header.set_path("jdk/evil").unwrap();
                header.set_entry_type(tar::EntryType::Symlink);
                header.set_link_name("/tmp").unwrap();
                header.set_size(0);
            } else {
                header.as_mut_bytes()[..9].copy_from_slice(b"../escape");
                header.set_entry_type(tar::EntryType::Regular);
                header.set_size(0);
            }
            header.set_cksum();
            builder.append(&header, &b""[..]).unwrap();
            builder.into_inner().unwrap().finish().unwrap();
            let stage = dir.path().join("stage");
            fs::create_dir(&stage).unwrap();
            assert!(extract_jdk(&archive, &stage, &AtomicBool::new(false)).is_err());
            assert!(!dir.path().join("escape").exists());
        }
    }
}

#[cfg(all(test, unix))]
mod selected_runtime_tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;
    #[test]
    fn validates_selected_runtime_and_rejects_missing_old_or_wrong_architecture() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("java");
        let cancel = AtomicBool::new(false);
        assert!(validate_selected(&path, 21, &cancel).is_err());
        for (major, arch, accepted) in [(17,"amd64",false),(21,"amd64",true),(25,"amd64",true),(21,"aarch64",false)] {
            fs::write(&path,format!("#!/bin/sh\nprintf 'java.specification.version = {major}\\njava.version = {major}.0.1\\nos.arch = {arch}\\nsun.arch.data.model = 64\\n'\n")).unwrap();
            fs::set_permissions(&path,fs::Permissions::from_mode(0o700)).unwrap();
            assert_eq!(validate_selected(&path,21,&cancel).is_ok(),accepted);
        }
    }
}
