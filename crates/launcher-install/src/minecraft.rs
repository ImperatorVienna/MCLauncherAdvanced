use sha1::Digest;
use crate::{
    atomic_json, check_cancel,
    download::{self, File},
    err,
    java::{self, Java},
    phase, require_platform, safe_path, Report, Result,
};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::BTreeMap,
    fs,
    io::{Read, Write},
    path::Path,
    sync::atomic::AtomicBool,
};
/// Conventional launcher isolation: this is the working directory / gameDir.
/// Libraries and asset objects remain rooted at the shared .minecraft directory.
pub fn instance_directory(root: &Path, directory_name: &str) -> Result<std::path::PathBuf> {
    if directory_name.trim().is_empty()
        || directory_name.len() > 255
        || directory_name.contains('/')
    {
        return Err("Invalid instance directory name".into());
    }
    safe_path(root, &format!("versions/{directory_name}"))
}

pub const MANIFEST: &str = "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Version {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub url: String,
    pub sha1: String,
    pub release_time: String,
}
#[derive(Deserialize)]
struct Manifest {
    versions: Vec<Version>,
}
pub fn catalog() -> Result<Vec<Version>> {
    require_platform()?;
    let bytes = download::bytes(&download::client()?, MANIFEST, 8 * 1024 * 1024)?;
    let manifest: Manifest = serde_json::from_slice(&bytes).map_err(err)?;
    Ok(manifest
        .versions
        .into_iter()
        .filter(|v| matches!(v.kind.as_str(), "release" | "snapshot"))
        .collect())
}
pub fn recommendation(id: &str) -> Result<u32> {
    let version = catalog()?
        .into_iter()
        .find(|v| v.id == id)
        .ok_or("Minecraft version not found in the official catalog")?;
    let (data, _) = metadata(&download::client()?, &version, &AtomicBool::new(false))?;
    java_major(&data)
}
fn text<'a>(v: &'a Value, key: &str) -> Result<&'a str> {
    v[key]
        .as_str()
        .ok_or_else(|| format!("Official metadata is missing {key}"))
}
pub fn java_major(metadata: &Value) -> Result<u32> {
    if let Some(java) = metadata.get("javaVersion") {
        return java["majorVersion"]
            .as_u64()
            .filter(|n| (8..=99).contains(n))
            .map(|n| n as u32)
            .ok_or("Invalid Java version declaration".into());
    }
    // Explicit release-only compatibility table; never infer from adjacent catalog entries.
    if metadata["type"] != "release" {
        return Err("This historical preview has no Java declaration; compatibility is unknown".into());
    }
    let id = text(metadata, "id")?;
    let parts: Vec<_> = id
        .split('.')
        .map(str::parse::<u32>)
        .collect::<std::result::Result<_, _>>()
        .map_err(|_| "Cannot determine the Java requirement for this historical version")?;
    match parts.as_slice() {
        [1, minor] | [1, minor, _] if *minor <= 16 => Ok(8),
        [1, 17] | [1, 17, _] => Ok(16),
        [1, 18..=19] | [1, 18..=19, _] | [1, 20] | [1, 20, 0..=4] => Ok(17),
        [1, 20, 5..=u32::MAX] | [1, 21] | [1, 21, _] => Ok(21),
        _ => Err("This version has no Java declaration or known compatibility rule".into()),
    }
}
fn matches_regex(pattern: &str, value: &str) -> Result<bool> {
    Ok(regex::Regex::new(&format!("^(?:{pattern})$"))
        .map_err(err)?
        .is_match(value))
}
pub fn allowed(library: &Value, kernel: &str) -> Result<bool> {
    allowed_for_os(library,kernel,launcher_platform::OS_NAME)
}
pub fn allowed_for_os(library: &Value, kernel: &str,host_os:&str) -> Result<bool> {
    let Some(rules) = library.get("rules") else {
        return Ok(true);
    };
    let mut allowed = false;
    for rule in rules.as_array().ok_or("Invalid library rules")? {
        let mut applies = true;
        if let Some(os) = rule.get("os") {
            if let Some(name) = os.get("name") {
                applies &= name == host_os;
            }
            if let Some(arch) = os.get("arch") {
                applies &= matches_regex(arch.as_str().ok_or("Invalid architecture rule")?, "x86_64")?;
            }
            if let Some(version) = os.get("version") {
                applies &= matches_regex(version.as_str().ok_or("Invalid operating system version rule")?, kernel)?;
            }
        }
        if let Some(features) = rule.get("features") {
            // No demo/custom-resolution/quick-play features enabled in vanilla installation.
            for value in features.as_object().ok_or("Invalid library feature rule")?.values() {
                applies &= value.as_bool() == Some(false);
            }
        }
        if applies {
            allowed = match rule["action"].as_str() {
                Some("allow") => true,
                Some("disallow") => false,
                _ => return Err("Unknown library rule".into()),
            };
        }
    }
    Ok(allowed)
}
fn spec(download: &Value, path: String) -> Result<File> {
    let file = File {
        path,
        url: text(download, "url")?.into(),
        hash: text(download, "sha1")?.into(),
        size: download["size"].as_u64().ok_or("Missing file size")?,
    };
    file.validate()?;
    Ok(file)
}
#[derive(Clone, Serialize, Deserialize)]
struct Native {
    path: String,
    exclude: Vec<String>,
}
pub struct Plan {
    pub files: Vec<File>,
    natives: Vec<Native>,
}
pub fn plan(metadata: &Value, index: &Value, kernel: &str) -> Result<Plan> {
    let id = text(metadata, "id")?;
    let mut files = vec![spec(
        &metadata["downloads"]["client"],
        format!("versions/{id}/{id}.jar"),
    )?];
    let mut natives = vec![];
    for lib in metadata["libraries"].as_array().ok_or("Minecraft version has no library list")? {
        if !allowed(lib, kernel)? {
            continue;
        }
        if let Some(artifact) = lib["downloads"].get("artifact") {
            files.push(spec(
                artifact,
                format!("libraries/{}", text(artifact, "path")?),
            )?);
        }
        if let Some(classifier) = lib["natives"][launcher_platform::OS_NAME].as_str() {
            let key = classifier.replace("${arch}", "64");
            let artifact = &lib["downloads"]["classifiers"][&key];
            let path = format!("libraries/{}", text(artifact, "path")?);
            files.push(spec(artifact, path.clone())?);
            natives.push(Native {
                path,
                exclude: lib["extract"]["exclude"]
                    .as_array()
                    .map(|a| {
                        a.iter()
                            .filter_map(|s| s.as_str().map(str::to_owned))
                            .collect()
                    })
                    .unwrap_or_default(),
            });
        }
    }
    if let Some(file) = metadata["logging"]["client"].get("file") {
        files.push(spec(
            file,
            format!("assets/log_configs/{}", text(file, "id")?),
        )?);
    }
    for object in index["objects"]
        .as_object()
        .ok_or("Invalid asset index")?
        .values()
    {
        let hash = text(object, "hash")?;
        if hash.len() != 40 || !hash.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err("Invalid asset hash".into());
        }
        files.push(File {
            path: format!("assets/objects/{}/{hash}", &hash[..2]),
            url: format!(
                "https://resources.download.minecraft.net/{}/{hash}",
                &hash[..2]
            ),
            hash: hash.into(),
            size: object["size"].as_u64().ok_or("Missing asset size")?,
        });
    }
    if let Some(auxiliary) = metadata.get("launcherAuxiliaryFiles") {
        let auxiliary: Vec<File> = serde_json::from_value(auxiliary.clone()).map_err(err)?;
        for file in auxiliary {
            if !file.path.starts_with("libraries/") { return Err("Invalid auxiliary mod loader path".into()); }
            files.push(file);
        }
    }
    let mut unique: BTreeMap<String, File> = BTreeMap::new();
    for file in files {
        file.validate()?;
        safe_path(Path::new("/unused"), &file.path)?;
        if let Some(old) = unique.get(&file.path) {
            if old.hash != file.hash || old.size != file.size {
                return Err("Official metadata contains conflicting files".into());
            }
        } else {
            unique.insert(file.path.clone(), file);
        }
    }
    Ok(Plan {
        files: unique.into_values().collect(),
        natives,
    })
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Receipt {
    pub minecraft_version: String,
    #[serde(default)]
    pub loader: Option<launcher_core::ModLoader>,
    #[serde(default)]
    pub java: Option<Java>,
    pub files: Vec<File>,
}
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Installed {
    pub minecraft_version: String,
    pub installed: bool,
    pub java: Option<Java>,
}
pub fn installed(root: &Path, version: &str) -> Installed {
    installed_named(root, version, version)
}
pub fn installed_named(root: &Path, version: &str, directory: &str) -> Installed {
    installed_with_loader(root, version, directory, None)
}
pub fn installed_with_loader(root: &Path, version: &str, directory: &str, loader: Option<&launcher_core::ModLoader>) -> Installed {
    let receipt = safe_path(root, &format!("versions/{directory}/launcher-install.json"))
        .ok()
        .and_then(|p| fs::read(p).ok())
        .and_then(|b| serde_json::from_slice::<Receipt>(&b).ok());
    // Lightweight inventory only; install/repair always performs complete hash validation.
    let valid = receipt.as_ref().is_some_and(|r| {
        r.minecraft_version == version
            && r.loader.as_ref() == loader
            && !r.files.is_empty()
            && r.files.iter().all(|f| {
                safe_path(root, &f.path)
                    .ok()
                    .and_then(|p| fs::metadata(p).ok())
                    .is_some_and(|m| m.is_file() && m.len() == f.size)
            })
    });
    Installed {
        minecraft_version: version.into(),
        installed: valid,
        java: receipt.and_then(|r| r.java),
    }
}
fn metadata(client: &Client, version: &Version, cancel: &AtomicBool) -> Result<(Value, Vec<u8>)> {
    check_cancel(cancel)?;
    let bytes = download::bytes(client, &version.url, 8 * 1024 * 1024)?;
    if !download::hash_ok(&bytes, &version.sha1) {
        return Err("Minecraft version metadata verification failed".into());
    }
    let data: Value = serde_json::from_slice(&bytes).map_err(err)?;
    if data["id"] != version.id || data.get("inheritsFrom").is_some() {
        return Err("Only standalone vanilla Minecraft versions are supported".into());
    }
    Ok((data, bytes))
}
fn extract_natives(
    root: &Path,
    id: &str,
    natives: &[Native],
    cancel: &AtomicBool,
) -> Result<Vec<File>> {
    let target = safe_path(root, &format!("versions/{id}/natives"))?;
    fs::create_dir_all(&target).map_err(err)?;
    let mut records = vec![];
    for native in natives {
        let path = safe_path(root, &native.path)?;
        let mut zip = zip::ZipArchive::new(fs::File::open(path).map_err(err)?).map_err(err)?;
        for i in 0..zip.len() {
            check_cancel(cancel)?;
            let mut entry = zip.by_index(i).map_err(err)?;
            let name = entry.name().to_string();
            if entry.is_dir()
                || name.starts_with("META-INF/")
                || native.exclude.iter().any(|p| name.starts_with(p))
            {
                continue;
            }
            if entry.is_symlink() || entry.size() > 256 * 1024 * 1024 {
                return Err("Invalid native library archive entry".into());
            }
            let destination = safe_path(&target, &name)?;
            fs::create_dir_all(destination.parent().unwrap()).map_err(err)?;
            let mut tmp =
                tempfile::NamedTempFile::new_in(destination.parent().unwrap()).map_err(err)?;
            let mut data = vec![];
            entry
                .by_ref()
                .take(256 * 1024 * 1024 + 1)
                .read_to_end(&mut data)
                .map_err(err)?;
            if data.len() as u64 != entry.size() {
                return Err("Native library extracted size mismatch".into());
            }
            tmp.write_all(&data).map_err(err)?;
            tmp.as_file().sync_all().map_err(err)?;
            tmp.persist(destination).map_err(err)?;
            use sha1::Digest;
            records.push(File {
                path: format!("versions/{id}/natives/{name}"),
                url: "https://libraries.minecraft.net/".into(),
                size: data.len() as u64,
                hash: format!("{:x}", sha1::Sha1::digest(&data)),
            });
        }
    }
    Ok(records)
}
fn legacy_assets(
    root: &Path,
    id: &str,
    version: &str,
    index: &Value,
    cancel: &AtomicBool,
) -> Result<Vec<File>> {
    let mut records = vec![];
    for (prefix, enabled) in [
        (
            format!("assets/virtual/{id}"),
            index["virtual"].as_bool().unwrap_or(false),
        ),
        (
            format!("versions/{version}/resources"),
            index["map_to_resources"].as_bool().unwrap_or(false),
        ),
    ] {
        if !enabled {
            continue;
        }
        for (name, object) in index["objects"].as_object().ok_or("Invalid asset index")? {
            check_cancel(cancel)?;
            let hash = text(object, "hash")?;
            let source = safe_path(root, &format!("assets/objects/{}/{hash}", &hash[..2]))?;
            let relative = format!("{prefix}/{name}");
            let destination = safe_path(root, &relative)?;
            fs::create_dir_all(destination.parent().unwrap()).map_err(err)?;
            let tmp =
                tempfile::NamedTempFile::new_in(destination.parent().unwrap()).map_err(err)?;
            fs::copy(source, tmp.path()).map_err(err)?;
            tmp.as_file().sync_all().map_err(err)?;
            tmp.persist(destination).map_err(err)?;
            records.push(File {
                path: relative,
                url: format!(
                    "https://resources.download.minecraft.net/{}/{hash}",
                    &hash[..2]
                ),
                hash: hash.into(),
                size: object["size"].as_u64().ok_or("Missing asset size")?,
            });
        }
    }
    Ok(records)
}
pub fn install(
    root: &Path,
    java_path: Option<&Path>,
    id: &str,
    cancel: &AtomicBool,
    report: &Report<'_>,
) -> Result<Receipt> {
    install_named(root, java_path, id, id, cancel, report)
}
pub fn install_named(root: &Path, java_path: Option<&Path>, id: &str, directory: &str, cancel: &AtomicBool, report: &Report<'_>) -> Result<Receipt> {
    install_with_loader(root, java_path, id, directory, None, cancel, report)
}
pub fn install_with_loader(root: &Path, java_path: Option<&Path>, id: &str, directory: &str, loader: Option<&launcher_core::ModLoader>, cancel: &AtomicBool, report: &Report<'_>) -> Result<Receipt> {
    require_platform()?;
    check_cancel(cancel)?;
    instance_directory(root, directory)?;
    safe_path(root, &format!("versions/{directory}/{directory}.json"))?;
    phase(report, "Fetching official version metadata");
    let version = catalog()?
        .into_iter()
        .find(|v| v.id == id)
        .ok_or("Minecraft version not found in the official catalog; check the instance")?;
    let client = download::client()?;
    let (mut data, _) = metadata(&client, &version, cancel)?;

    if let Some(loader) = loader {
        phase(report, "Downloading and verifying mod loader");
        let profile = crate::loaders::prepare_for_game(root, id, directory, loader, &data, java_path, cancel, report)?;
        crate::loaders::merge(&mut data, &profile, id)?;
        data["launcherLoader"] = serde_json::to_value(loader).map_err(err)?;
    }
    data["id"] = Value::String(directory.into());
    let raw = serde_json::to_vec_pretty(&data).map_err(err)?;
    let index_spec = spec(
        &data["assetIndex"],
        format!("assets/indexes/{}.json", text(&data["assetIndex"], "id")?),
    )?;
    if index_spec.size > 32 * 1024 * 1024 {
        return Err("Asset index exceeds the size limit".into());
    }
    download::download_all(
        &client,
        root,
        &[index_spec.clone()],
        cancel,
        report,
        "Downloading asset index",
    )?;
    let index: Value =
        serde_json::from_slice(&fs::read(safe_path(root, &index_spec.path)?).map_err(err)?)
            .map_err(err)?;
    let kernel = launcher_platform::kernel_version().unwrap_or_default();
    let mut plan = plan(&data, &index, kernel.trim())?;
    // Runtime selection is explicit and independent of the game's recommended major.
    // Game files may be installed before the player selects/downloads Java.
    let java = java_path
        .map(|path| {
            java::probe(path, "selected", cancel)?
                .ok_or_else(|| String::from("The selected Java cannot run; select another runtime"))
        })
        .transpose()?;
    download::download_all(
        &client,
        root,
        &plan.files,
        cancel,
        report,
        "Downloading and verifying game files",
    )?;
    phase(report, "Preparing native libraries and legacy assets");
    plan.files
        .extend(extract_natives(root, directory, &plan.natives, cancel)?);
    plan.files.extend(legacy_assets(
        root,
        text(&data["assetIndex"], "id")?,
        directory,
        &index,
        cancel,
    )?);
    plan.files.push(index_spec);
    check_cancel(cancel)?;
    let json_path = format!("versions/{directory}/{directory}.json");
    let path = safe_path(root, &json_path)?;
    // The local manifest ID matches its isolated directory; hash the adapted bytes.
    let mut tmp =
        tempfile::NamedTempFile::new_in(path.parent().ok_or("Invalid instance path")?).map_err(err)?;
    tmp.write_all(&raw).map_err(err)?;
    tmp.as_file().sync_all().map_err(err)?;
    tmp.persist(path).map_err(err)?;
    plan.files.push(File {
        path: json_path,
        url: version.url,
        hash: format!("{:x}", sha1::Sha1::digest(&raw)),
        size: raw.len() as u64,
    });
    let receipt = Receipt {
        minecraft_version: id.into(),
        loader: loader.cloned(),
        java,
        files: plan.files,
    };
    check_cancel(cancel)?;
    atomic_json(
        &safe_path(root, &format!("versions/{directory}/launcher-install.json"))?,
        &receipt,
    )?;
    Ok(receipt)
}
/// Prepare a replacement without writing the instance's manifest or receipt.
/// Game version is fixed, so native/assets records remain valid and user files stay untouched.
pub fn prepare_loader_change(root: &Path, item: &launcher_core::Instance, loader: &launcher_core::ModLoader, cancel: &AtomicBool, report: &Report<'_>) -> Result<(Vec<u8>, Receipt)> {
    loader.validate()?;
    if item.loader.as_ref().is_some_and(|old| old.kind != loader.kind) {
        return Err("Switching mod loader families is not supported".into());
    }
    if !installed_with_loader(root, &item.minecraft_version, item.directory(), item.loader.as_ref()).installed {
        return Err("Repair the instance before changing its mod loader".into());
    }
    let dir = instance_directory(root, item.directory())?;
    let mut receipt: Receipt = serde_json::from_slice(&fs::read(dir.join("launcher-install.json")).map_err(err)?).map_err(err)?;
    phase(report, "Fetching official version metadata");
    let version = catalog()?.into_iter().find(|v| v.id == item.minecraft_version).ok_or("Minecraft version not found in the official catalog; check the instance")?;
    let client = download::client()?;
    let (mut data, _) = metadata(&client, &version, cancel)?;
    phase(report, "Downloading and verifying mod loader");
    let profile = crate::loaders::prepare_for_game(root, &item.minecraft_version, item.directory(), loader, &data, item.java_path.as_deref().map(Path::new), cancel, report)?;
    crate::loaders::merge(&mut data, &profile, &item.minecraft_version)?;
    data["id"] = Value::String(item.directory().into());
    data["launcherLoader"] = serde_json::to_value(loader).map_err(err)?;
    let kernel = launcher_platform::kernel_version().unwrap_or_default();
    let dependencies: Vec<File> = plan(&data, &serde_json::json!({"objects":{}}), kernel.trim())?.files.into_iter().filter(|f| f.path.starts_with("libraries/")).collect();
    download::download_all(&client, root, &dependencies, cancel, report, "Downloading and verifying mod loader")?;
    let raw = serde_json::to_vec_pretty(&data).map_err(err)?;
    let manifest = format!("versions/{0}/{0}.json", item.directory());
    receipt.files.retain(|f| !f.path.starts_with("libraries/") && f.path != manifest);
    receipt.files.extend(dependencies);
    receipt.files.push(File {path:manifest,url:version.url,hash:format!("{:x}", sha1::Sha1::digest(&raw)),size:raw.len() as u64});
    receipt.loader = Some(loader.clone());
    check_cancel(cancel)?;
    Ok((raw, receipt))
}
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn conventional_version_directories_preserve_existing_data() {
        let dir = tempfile::tempdir().unwrap();
        let old = dir.path().join("versions/1.21.11-NeoForge");
        fs::create_dir_all(old.join("saves")).unwrap();
        fs::create_dir_all(old.join("mods")).unwrap();
        fs::write(old.join("saves/world"), b"existing world").unwrap();
        fs::write(old.join("options.txt"), b"settings").unwrap();
        assert_eq!(
            instance_directory(dir.path(), "1.21.11-NeoForge").unwrap(),
            old
        );
        let vanilla = instance_directory(dir.path(), "1.21.11").unwrap();
        assert!(!vanilla.join("saves/world").exists());
        assert_eq!(fs::read(old.join("options.txt")).unwrap(), b"settings");
        assert!(!dir.path().join("instances").exists());
        for bad in ["..", "../escape", "a/b", ""] {
            assert!(instance_directory(dir.path(), bad).is_err());
        }
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(&old, dir.path().join("versions/link")).unwrap();
            assert!(instance_directory(dir.path(), "link").is_err());
        }
    }
    #[test]
    fn historical_resources_stay_in_the_version_game_directory() {
        let dir = tempfile::tempdir().unwrap();
        let hash = "a9993e364706816aba3e25717850c26c9cd0d89d";
        let source = dir.path().join(format!("assets/objects/a9/{hash}"));
        fs::create_dir_all(source.parent().unwrap()).unwrap();
        fs::write(source, b"abc").unwrap();
        let index =
            json!({"map_to_resources":true,"objects":{"sound/test.ogg":{"hash":hash,"size":3}}});
        let records = legacy_assets(
            dir.path(),
            "legacy",
            "1.6.4",
            &index,
            &AtomicBool::new(false),
        )
        .unwrap();
        assert_eq!(records[0].path, "versions/1.6.4/resources/sound/test.ogg");
        assert!(!dir.path().join("resources").exists());
    }
    #[test]
    fn authoritative_java_and_explicit_fallbacks() {
        assert_eq!(
            java_major(&json!({"id":"anything","javaVersion":{"majorVersion":25}})).unwrap(),
            25
        );
        for (id, major) in [
            ("1.12.2", 8),
            ("1.17.1", 16),
            ("1.18.2", 17),
            ("1.20.4", 17),
            ("1.20.6", 21),
            ("1.21.1", 21),
        ] {
            assert_eq!(
                java_major(&json!({"id":id,"type":"release"})).unwrap(),
                major
            );
        }
        assert!(java_major(&json!({"id":"26.4","type":"release"})).is_err());
        assert!(java_major(&json!({"id":"21w08a","type":"snapshot"})).is_err());
        assert!(java_major(
            &json!({"id":"1.12.2","type":"release","javaVersion":{"majorVersion":0}})
        )
        .is_err());
    }
    #[test]
    fn rules_are_ordered_and_filter_os_arch_features() {
        assert!(allowed(&json!({}), "6.1").unwrap());
        assert!(!allowed(
            &json!({"rules":[{"action":"allow","os":{"name":"windows"}}]}),
            "6.1"
        )
        .unwrap());
        assert!(!allowed(
            &json!({"rules":[{"action":"allow"},{"action":"disallow","os":{"name":"linux"}}]}),
            "6.1"
        )
        .unwrap());
        assert!(!allowed(
            &json!({"rules":[{"action":"allow","os":{"arch":"aarch64"}}]}),
            "6.1"
        )
        .unwrap());
        assert!(allowed(&json!({"rules":[{"action":"allow","os":{"name":"linux","arch":"x86_64","version":"^6.*"}}]}),"6.1").unwrap());
        assert!(!allowed(
            &json!({"rules":[{"action":"allow","features":{"is_demo_user":true}}]}),
            "6.1"
        )
        .unwrap());
    }
    #[test]
    fn windows_rules_select_windows_libraries_and_arguments() {
        let windows=json!({"rules":[{"action":"allow","os":{"name":"windows","arch":"x86_64"}}]});
        assert!(allowed_for_os(&windows,"10.0.19045","windows").unwrap());
        assert!(!allowed_for_os(&windows,"6.1","linux").unwrap());
        let linux=json!({"rules":[{"action":"allow","os":{"name":"linux"}}]});
        assert!(!allowed_for_os(&linux,"10.0.19045","windows").unwrap());
    }
    #[test]
    fn receipt_never_marks_partial_install_complete() {
        let dir = tempfile::tempdir().unwrap();
        assert!(!installed(dir.path(), "1.21.1").installed);
    }
    #[test]
    fn named_installations_of_same_release_have_independent_receipts() {
        let dir = tempfile::tempdir().unwrap();
        let name = "\u{4e2d}\u{6587} \u{751f}\u{5b58}";
        let metadata = json!({"id":name,"downloads":{"client":{"url":"https://piston-data.mojang.com/client.jar","sha1":"a9993e364706816aba3e25717850c26c9cd0d89d","size":3}},"libraries":[]});
        let plan = plan(&metadata, &json!({"objects":{}}), "6.1").unwrap();
        assert_eq!(plan.files[0].path, "versions/\u{4e2d}\u{6587} \u{751f}\u{5b58}/\u{4e2d}\u{6587} \u{751f}\u{5b58}.jar");
        let target = instance_directory(dir.path(), name).unwrap();
        fs::create_dir_all(&target).unwrap();
        fs::write(target.join(format!("{name}.jar")), b"abc").unwrap();
        atomic_json(&target.join("launcher-install.json"), &Receipt { minecraft_version:"26.3".into(), loader:None, java:None, files:plan.files }).unwrap();
        assert!(installed_named(dir.path(), "26.3", name).installed);
        assert!(!installed_named(dir.path(), "26.3", "\u{53e6}\u{4e00}\u{4efd}").installed);
        assert!(!installed_named(dir.path(), "1.21", name).installed);
    }
    #[test]
    fn installed_game_does_not_require_a_java_selection() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("versions/test")).unwrap();
        fs::write(dir.path().join("versions/test/test.jar"), b"game").unwrap();
        let receipt = Receipt {
            minecraft_version: "test".into(),
            loader: None,
            java: None,
            files: vec![File {
                path: "versions/test/test.jar".into(),
                url: "https://example.invalid".into(),
                hash: "".into(),
                size: 4,
            }],
        };
        atomic_json(
            &dir.path().join("versions/test/launcher-install.json"),
            &receipt,
        )
        .unwrap();
        assert!(installed(dir.path(), "test").installed);
        assert!(installed(dir.path(), "test").java.is_none());
        fs::remove_file(dir.path().join("versions/test/test.jar")).unwrap();
        assert!(!installed(dir.path(), "test").installed);
    }
    #[test]
    fn native_zip_cannot_escape_game_directory() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("bad.jar");
        let mut zip = zip::ZipWriter::new(fs::File::create(&path).unwrap());
        zip.start_file("../../escape", zip::write::SimpleFileOptions::default())
            .unwrap();
        zip.write_all(b"bad").unwrap();
        zip.finish().unwrap();
        let native = Native {
            path: "bad.jar".into(),
            exclude: vec![],
        };
        assert!(extract_natives(dir.path(), "test", &[native], &AtomicBool::new(false)).is_err());
        assert!(!dir.path().join("versions/escape").exists());
    }
    #[test]
    fn cancellation_cannot_commit_an_install_record() {
        let dir = tempfile::tempdir().unwrap();
        assert!(install(dir.path(), None, "1.21.1", &AtomicBool::new(true), &|_| {}).is_err());
        assert!(!installed(dir.path(), "1.21.1").installed);
    }
    #[test]
    fn plan_includes_linux_libraries_logging_and_deduplicated_assets() {
        let download = |path: &str| json!({"path":path,"id":path,"url":"https://libraries.minecraft.net/test","sha1":"a9993e364706816aba3e25717850c26c9cd0d89d","size":3});
        let data = json!({"id":"test","downloads":{"client":download("client")},"libraries":[
            {"downloads":{"artifact":download("linux.jar")},"rules":[{"action":"allow","os":{"name":"linux"}}]},
            {"downloads":{"artifact":download("windows.jar")},"rules":[{"action":"allow","os":{"name":"windows"}}]},
            {"downloads":{"artifact":download("x86.jar")},"rules":[{"action":"allow","os":{"arch":"x86"}}]},
            {"downloads":{"classifiers":{"natives-linux":download("native.jar")}},"natives":{"linux":"natives-linux"},"extract":{"exclude":["META-INF/"]}}
        ],"launcherAuxiliaryFiles":[{"path":"libraries/patched.jar","url":"https://maven.neoforged.net/installer.jar","hash":"a9993e364706816aba3e25717850c26c9cd0d89d","size":3}],"logging":{"client":{"file":download("log.xml")}}});
        let object = json!({"hash":"a9993e364706816aba3e25717850c26c9cd0d89d","size":3});
        let index = json!({"objects":{"first":object,"duplicate":object}});
        let plan = plan(&data, &index, "6.1").unwrap();
        let paths: Vec<_> = plan.files.iter().map(|f| f.path.as_str()).collect();
        assert!(paths.contains(&"versions/test/test.jar"));
        assert!(paths.contains(&"libraries/linux.jar"));
        assert!(paths.contains(&"libraries/patched.jar"));
        assert!(paths.contains(&"assets/log_configs/log.xml"));
        assert!(!paths.contains(&"libraries/windows.jar"));
        assert!(!paths.contains(&"libraries/x86.jar"));
        assert_eq!(
            paths
                .iter()
                .filter(|p| p.starts_with("assets/objects/"))
                .count(),
            1
        );
        assert_eq!(plan.natives.len(), 1);
    }
}
