//! Fabric and Quilt metadata adapters. No installers, shell commands or processors.
use crate::{download, check_cancel, err, safe_path, Result};
use launcher_core::ModLoader;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{path::Path, sync::atomic::AtomicBool, io::Write};
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LoaderVersion { pub version: String, pub stable: bool }
fn base(kind: &str) -> Result<&'static str> {
    match kind {
        "fabric" => Ok("https://meta.fabricmc.net/v2"),
        "quilt" => Ok("https://meta.quiltmc.org/v3"),
        _ => Err("Invalid mod loader selection".into()),
    }
}
fn game_id(game: &str) -> Result<()> {
    if game.is_empty() || game.len() > 80 || !game.bytes().all(|b| b.is_ascii_alphanumeric() || b".-_".contains(&b)) { return Err("Invalid Minecraft version format".into()); }
    Ok(())
}
pub fn parse_versions(data: &Value) -> Result<Vec<LoaderVersion>> {
    let mut versions = vec![];
    for item in data.as_array().ok_or("Invalid mod loader catalog")? {
        let loader = &item["loader"];
        let version = loader["version"].as_str().ok_or("Invalid mod loader catalog")?;
        ModLoader {kind:"fabric".into(),version:version.into()}.validate()?;
        let stable = loader["stable"].as_bool().unwrap_or(!version.contains('-'));
        versions.push(LoaderVersion { version: version.into(), stable });
    }
    sort_versions(&mut versions);
    Ok(versions)
}
pub fn sort_versions(versions: &mut Vec<LoaderVersion>) {
    versions.sort_by(|a,b| {
        match (semver::Version::parse(&a.version),semver::Version::parse(&b.version)) {
            (Ok(a),Ok(b)) => b.cmp(&a),
            _ => {
                let parts = |s: &str| s.split(['-','+']).next().unwrap_or("").split('.').map(|p| p.parse::<u64>().unwrap_or(0)).collect::<Vec<_>>();
                parts(&b.version).cmp(&parts(&a.version)).then_with(|| b.version.cmp(&a.version))
            }
        }
    });
    versions.dedup_by(|a,b| a.version == b.version);
}
pub fn catalog(kind: &str, game: &str) -> Result<Vec<LoaderVersion>> {
    game_id(game)?;
    if matches!(kind, "forge" | "neoforge") { return crate::forge::catalog(kind, game); }
    let client = download::client()?;
    // Both services expose only game versions with the required mappings.
    let root = base(kind)?;
    let games: Value = serde_json::from_slice(&download::bytes(&client,&format!("{root}/versions/game"),8*1024*1024)?).map_err(err)?;
    if !games.as_array().ok_or("Invalid mod loader catalog")?.iter().any(|v| v["version"] == game) { return Ok(vec![]); }
    let data = download::bytes(&client, &format!("{root}/versions/loader/{game}"), 16*1024*1024)?;
    parse_versions(&serde_json::from_slice(&data).map_err(err)?)
}
fn maven_path(name: &str) -> Result<String> {
    let parts: Vec<_> = name.split(':').collect();
    if parts.len()!=3 || parts.iter().any(|p| p.is_empty() || !p.bytes().all(|b| b.is_ascii_alphanumeric() || b"._-+".contains(&b))) || parts[0].split('.').any(str::is_empty) || parts.iter().any(|p| *p=="." || *p=="..") {
        return Err("Invalid mod loader library coordinate".into());
    }
    Ok(format!("{}/{}/{}/{}-{}.jar",parts[0].replace('.',"/"),parts[1],parts[2],parts[1],parts[2]))
}
/// Flatten only the explicitly supported fields; base game downloads/assets stay authoritative.
pub fn merge(base: &mut Value, profile: &Value, game: &str) -> Result<()> {
    if profile["inheritsFrom"] != game { return Err("Mod loader profile does not match the game version".into()); }
    let main = profile["mainClass"].as_str().ok_or("Invalid mod loader profile")?;
    if !["net.fabricmc.loader.impl.launch.knot.KnotClient","net.fabricmc.loader.launch.knot.KnotClient","org.quiltmc.loader.impl.launch.knot.KnotClient", "cpw.mods.modlauncher.Launcher", "cpw.mods.bootstraplauncher.BootstrapLauncher", "net.minecraftforge.bootstrap.ForgeBootstrap", "net.neoforged.fml.startup.Client"].contains(&main) { return Err("Unsupported mod loader main class".into()); }
    if !base["arguments"]["game"].is_array() || !base["arguments"]["jvm"].is_array() { return Err("This game uses an unsupported legacy launch format".into()); }
    for kind in ["game","jvm"] {
        if let Some(args) = profile["arguments"].get(kind) {
            let args = args.as_array().ok_or("Invalid mod loader arguments")?;
            base["arguments"][kind].as_array_mut().unwrap().extend(args.clone());
        }
    }
    let extra = profile["libraries"].as_array().ok_or("Invalid mod loader libraries")?;
    let libs = base["libraries"].as_array_mut().ok_or("Invalid Minecraft libraries")?;
    // Loader libraries override a matching Maven group/artifact from vanilla.
    let coordinate = |v: &Value| v["name"].as_str().map(|n| { let parts: Vec<_> = n.split(':').collect(); format!("{}:{}:{}", parts.first().unwrap_or(&""), parts.get(1).unwrap_or(&""), parts.get(3).unwrap_or(&"")) });
    libs.retain(|lib| !extra.iter().any(|e| coordinate(e).is_some() && coordinate(e)==coordinate(lib)));
    libs.extend(extra.clone());
    if let Some(files) = profile.get("launcherAuxiliaryFiles") {
        base["launcherAuxiliaryFiles"] = files.clone();
    }
    base["mainClass"] = json!(main);
    Ok(())
}
pub fn prepare_for_game(root: &Path, game: &str, directory: &str, loader: &ModLoader, vanilla: &Value, java: Option<&Path>, cancel: &AtomicBool, report: &crate::Report<'_>) -> Result<Value> {
    loader.validate()?; game_id(game)?;
    if matches!(loader.kind.as_str(), "forge" | "neoforge") {
        crate::forge::prepare(root, game, directory, loader, vanilla, java, cancel, report)
    } else { prepare(root, game, loader, cancel) }
}
pub fn prepare(root: &Path, game: &str, loader: &ModLoader, cancel: &AtomicBool) -> Result<Value> {
    loader.validate()?; game_id(game)?;
    if !catalog(&loader.kind, game)?.iter().any(|v| v.version == loader.version) { return Err("The selected mod loader does not support this game version".into()); }
    check_cancel(cancel)?;
    let client = download::client()?;
    let url = format!("{}/versions/loader/{}/{}/profile/json",base(&loader.kind)?,game,loader.version);
    let mut profile: Value = serde_json::from_slice(&download::bytes(&client,&url,8*1024*1024)?).map_err(err)?;
    if profile["inheritsFrom"] != game { return Err("Mod loader profile does not match the game version".into()); }
    for lib in profile["libraries"].as_array_mut().ok_or("Invalid mod loader libraries")? {
        check_cancel(cancel)?;
        let path = maven_path(lib["name"].as_str().ok_or("Invalid mod loader library coordinate")?)?;
        let repo = lib["url"].as_str().ok_or("Invalid mod loader repository")?;
        if !["https://maven.fabricmc.net/","https://maven.quiltmc.org/repository/release/"].contains(&repo) { return Err("Invalid mod loader repository".into()); }
        let url = format!("{repo}{path}");
        let hash = match lib["sha1"].as_str() {
            Some(hash) => hash.to_owned(),
            None => String::from_utf8(download::bytes(&client,&format!("{url}.sha1"),1024)?).map_err(err)?.split_whitespace().next().ok_or("Missing mod loader checksum")?.to_owned(),
        };
        if hash.len()!=40 || !hash.bytes().all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase()) { return Err("Invalid mod loader checksum".into()); }
        let local = safe_path(root,&format!("libraries/{path}"))?;
        let size = lib["size"].as_u64().or_else(|| std::fs::metadata(&local).ok().map(|m| m.len())).unwrap_or(0);
        let mut spec = download::File { path: format!("libraries/{path}"), url: url.clone(), hash: hash.clone(), size };
        if !download::valid_file(&local,&spec,cancel)? {
            // Missing size in official Maven profiles: fetch bounded content, verify,
            // then cache atomically. The normal installer subsequently reuses it.
            let bytes = download::bytes(&client,&url,64*1024*1024)?;
            check_cancel(cancel)?;
            if !download::hash_ok(&bytes,&hash) { return Err("Mod loader library checksum mismatch".into()); }
            spec.size = bytes.len() as u64;
            spec.validate()?;
            std::fs::create_dir_all(local.parent().unwrap()).map_err(err)?;
            let mut temp = tempfile::NamedTempFile::new_in(local.parent().unwrap()).map_err(err)?;
            temp.write_all(&bytes).map_err(err)?; temp.as_file().sync_all().map_err(err)?;
            temp.persist(local).map_err(err)?;
        }
        spec.validate()?;
        lib["downloads"] = json!({"artifact":{"path":path,"url":url,"sha1":hash,"size":spec.size}});
    }
    Ok(profile)
}
#[cfg(test)] mod tests {
    use super::*;
    #[test]
    #[ignore = "Downloads verified loader artifacts from official services"]
    fn official_loader_profiles_prepare_for_both_adapters() {
        let temp = tempfile::tempdir().unwrap();
        for (kind,version) in [("fabric","0.16.14"),("quilt","0.29.2")] {
            let loader=ModLoader {kind:kind.into(),version:version.into()};
            let profile=prepare(temp.path(),"1.21.1",&loader,&AtomicBool::new(false)).unwrap();
            for lib in profile["libraries"].as_array().unwrap() {
                let artifact=&lib["downloads"]["artifact"];
                let file=download::File {path:format!("libraries/{}",artifact["path"].as_str().unwrap()),url:artifact["url"].as_str().unwrap().into(),hash:artifact["sha1"].as_str().unwrap().into(),size:artifact["size"].as_u64().unwrap()};
                assert!(download::valid_file(&safe_path(temp.path(),&file.path).unwrap(),&file,&AtomicBool::new(false)).unwrap());
            }
        }
    }
    #[test] fn ordering_and_unstable_versions() {
        let versions=parse_versions(&json!([{"loader":{"version":"0.9.0"}},{"loader":{"version":"0.10.0-beta.1"}},{"loader":{"version":"0.10.0"}}])).unwrap();
        assert_eq!(versions[0].version,"0.10.0"); assert!(versions[0].stable); assert!(!versions[1].stable);
        assert!(maven_path("../bad:lib:1").is_err()); assert!(maven_path("g:a:../bad").is_err());
    }
    #[test] fn merge_preserves_game_and_replaces_conflicting_libraries() {
        let mut game=json!({"downloads":{"client":{"sha1":"original"}},"arguments":{"jvm":["-Xtest"],"game":["--username","test"]},"libraries":[{"name":"g:a:1"},{"name":"g:b:1"}]});
        let profile=json!({"inheritsFrom":"1.21.1","mainClass":"org.quiltmc.loader.impl.launch.knot.KnotClient","arguments":{"jvm":["-Dtest"]},"libraries":[{"name":"g:a:2"}]});
        merge(&mut game,&profile,"1.21.1").unwrap();
        assert_eq!(game["downloads"]["client"]["sha1"],"original"); assert_eq!(game["libraries"].as_array().unwrap().len(),2);
        assert_eq!(game["arguments"]["jvm"],json!(["-Xtest","-Dtest"]));
        assert!(merge(&mut game,&profile,"1.20.1").is_err());
    }
}
