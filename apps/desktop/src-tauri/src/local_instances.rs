//! Discover local version metadata without changing another launcher's game files.
use launcher_core::{storage::Storage, Instance, ModLoader};
use serde_json::Value;
use std::fs;
fn argument<'a>(data: &'a Value, key: &str) -> Option<&'a str> {
    data["arguments"]["game"].as_array()?.windows(2)
        .find_map(|pair| (pair[0].as_str()==Some(key)).then(||pair[1].as_str()).flatten())
}
fn patch_version<'a>(data: &'a Value, id: &str) -> Option<&'a str> {
    data["patches"].as_array()?.iter().find(|p|p["id"].as_str()==Some(id))?["version"].as_str()
}
fn game_version(data: &Value) -> Option<&str> {
    patch_version(data,"game")
        .or_else(||argument(data,"--fml.mcVersion"))
        .or_else(||data["inheritsFrom"].as_str())
        .or_else(||data["clientVersion"].as_str())
        .or_else(||data["id"].as_str())
}
fn loader(data: &Value) -> Option<ModLoader> {
    if let Some(value) = data.get("launcherLoader") {
        return serde_json::from_value::<ModLoader>(value.clone()).ok().filter(|l|l.validate().is_ok());
    }
    for (kind, flag) in [("neoforge","--fml.neoForgeVersion"),("forge","--fml.forgeVersion"),("fabric",""),("quilt","")] {
        if let Some(version)=patch_version(data,kind).or_else(|| if flag.is_empty() {None} else {argument(data,flag)}) {
            let result=ModLoader {kind:kind.into(),version:version.into()};
            if result.validate().is_ok() {return Some(result)}
        }
    }
    for library in data["libraries"].as_array().into_iter().flatten() {
        let name=library["name"].as_str().unwrap_or("");
        for (prefix,kind) in [("net.fabricmc:fabric-loader:","fabric"),("org.quiltmc:quilt-loader:","quilt"),("net.minecraftforge:forge:","forge"),("net.neoforged:neoforge:","neoforge")] {
            if let Some(version)=name.strip_prefix(prefix) {
                let version=if kind=="forge" { version.split_once('-').map(|(_,v)|v).unwrap_or(version) } else {version};
                let result=ModLoader {kind:kind.into(),version:version.into()};
                if result.validate().is_ok() {return Some(result)}
            }
        }
    }
    None
}
pub fn discover(storage: &Storage) -> Result<(),String> {
    let root=&storage.location.root;
    let versions=launcher_install::safe_path(root,"versions")?;
    let entries=match fs::read_dir(&versions) {Ok(v)=>v,Err(e) if e.kind()==std::io::ErrorKind::NotFound=>return Ok(()),Err(e)=>return Err(e.to_string())};
    let mut entries=entries.filter_map(Result::ok).collect::<Vec<_>>(); entries.sort_by_key(|e|e.file_name());
    let store=storage.store(); let mut library=store.load()?; let mut changed=false;
    for entry in entries {
        if !entry.file_type().is_ok_and(|t|t.is_dir()) {continue}
        let name=entry.file_name().to_string_lossy().into_owned();
        if launcher_core::validate_directory_name(&name).is_err() || name.chars().count()>64 {continue}
        let Ok(path)=launcher_install::safe_path(root,&format!("versions/{name}/{name}.json")) else {continue};
        if !fs::metadata(&path).is_ok_and(|m|m.len()<=16*1024*1024) {continue}
        let Ok(bytes)=fs::read(&path) else {continue};
        let Ok(data)=serde_json::from_slice::<Value>(&bytes) else {continue};
        let version=game_version(&data);
        let Some(version)=version else {continue};
        // Our flattened manifests keep the official version in their installation receipt.
        let receipt=fs::read(entry.path().join("launcher-install.json")).ok().and_then(|b|serde_json::from_slice::<Value>(&b).ok());
        let version=receipt.as_ref().and_then(|v|v["minecraftVersion"].as_str()).unwrap_or(version);
        if let Some(existing)=library.instances.iter().position(|i|i.directory().eq_ignore_ascii_case(&name)) {
            // Refresh external discovery records, never override our installed configuration metadata.
            if receipt.is_none() && library.instances[existing].id.starts_with("local-") {
                let mut candidate=library.clone();
                let item=&mut candidate.instances[existing];
                item.minecraft_version=version.into(); item.loader=loader(&data);
                if candidate.validate().is_ok() && (item_version_changed(&library.instances[existing], &candidate.instances[existing])) {
                    library=candidate; changed=true;
                }
            }
            continue;
        }
        if library.instances.len()>=1000 || library.instances.iter().any(|i|i.name.eq_ignore_ascii_case(&name)) {continue}
        let mut serial=library.instances.len();
        let id=loop {let id=format!("local-{serial}"); if !library.instances.iter().any(|i|i.id==id) {break id} serial+=1;};
        let item=Instance {id:id.clone(),name:name.clone(),directory_name:name.clone(),minecraft_version:version.into(),icon:random_icon(&name),loader:loader(&data),java_path:None,launch_settings:Default::default()};
        let mut candidate=library.clone(); candidate.instances.push(item);
        if candidate.selected_id.is_none() {candidate.selected_id=Some(id)}
        if candidate.validate().is_ok() {library=candidate; changed=true;}
    }
    if changed {store.save(library)?;}
    Ok(())
}

fn item_version_changed(old: &Instance, new: &Instance) -> bool {
    old.minecraft_version != new.minecraft_version || old.loader != new.loader
}

fn random_icon(name: &str) -> String {
    let seed=format!("{name}-{:?}-{}",std::time::SystemTime::now(),std::process::id());
    let hash=md5::compute(seed);
    let index=u64::from_le_bytes(hash.0[..8].try_into().unwrap()) as usize % launcher_core::INSTANCE_ICONS.len();
    launcher_core::INSTANCE_ICONS[index].into()
}

#[cfg(test)] mod tests {
    use super::*;
    #[test] fn flattened_loader_metadata_and_existing_records_are_corrected() {
        let data=serde_json::json!({"id":"1.21.11-NeoForge","patches":[{"id":"game","version":"1.21.11"},{"id":"neoforge","version":"21.11.45"}],"arguments":{"game":["--fml.mcVersion","1.21.11","--fml.neoForgeVersion","21.11.45"]},"libraries":[]});
        assert_eq!(game_version(&data),Some("1.21.11"));
        assert_eq!(loader(&data).unwrap(),ModLoader {kind:"neoforge".into(),version:"21.11.45".into()});
        let mut args_only=data.clone(); args_only.as_object_mut().unwrap().remove("patches");
        assert_eq!(game_version(&args_only),Some("1.21.11"));
        assert_eq!(loader(&args_only),loader(&data));
        for kind in ["fabric","quilt","forge","neoforge"] {
            let patch=serde_json::json!({"patches":[{"id":"game","version":"1.21.1"},{"id":kind,"version":"1.2.3"}]});
            assert_eq!(loader(&patch).unwrap().kind,kind);
        }
        let base=std::env::temp_dir().join(format!("local-correction-{}",std::process::id()));
        fs::create_dir_all(&base).unwrap(); let storage=Storage::open(&base).unwrap();
        let dir=storage.location.root.join("versions/1.21.11-NeoForge"); fs::create_dir_all(&dir).unwrap();
        let bytes=serde_json::to_vec(&data).unwrap(); fs::write(dir.join("1.21.11-NeoForge.json"),&bytes).unwrap();
        let old=Instance {id:"local-0".into(),name:"Custom name".into(),directory_name:"1.21.11-NeoForge".into(),minecraft_version:"1.21.11-NeoForge".into(),loader:None,icon:"diamond".into(),java_path:Some("/test/java".into()),launch_settings:Default::default()};
        storage.store().save(launcher_core::Library {instances:vec![old],selected_id:Some("local-0".into()),..Default::default()}).unwrap();
        discover(&storage).unwrap(); let library=storage.store().load().unwrap(); let item=&library.instances[0];
        assert_eq!(item.minecraft_version,"1.21.11"); assert_eq!(item.loader,loader(&data));
        assert_eq!(item.name,"Custom name"); assert_eq!(item.icon,"diamond"); assert_eq!(item.java_path.as_deref(),Some("/test/java"));
        discover(&storage).unwrap(); assert_eq!(storage.store().load().unwrap().revision,library.revision);
        assert_eq!(fs::read(dir.join("1.21.11-NeoForge.json")).unwrap(),bytes);
        fs::remove_dir_all(base).unwrap();
    }
    #[test] fn discovery_names_icons_and_repeat_scan_preserve_existing_files() {
        let base=std::env::temp_dir().join(format!("local-discovery-{}",std::process::id()));
        fs::create_dir_all(&base).unwrap();
        let storage=Storage::open(&base).unwrap();
        for (name,json) in [("Vanilla",r#"{"id":"1.21.1","libraries":[]}"#),("My Fabric",r#"{"id":"fabric-loader-0.16.0-1.21.1","inheritsFrom":"1.21.1","libraries":[{"name":"net.fabricmc:fabric-loader:0.16.0"}]}"#)] {
            let dir=storage.location.root.join("versions").join(name);
            fs::create_dir_all(&dir).unwrap(); fs::write(dir.join(format!("{name}.json")),json).unwrap();
            fs::write(dir.join("options.txt"),b"keep").unwrap();
        }
        discover(&storage).unwrap();
        let first=storage.store().load().unwrap(); assert_eq!(first.instances.len(),2);
        let fabric=first.instances.iter().find(|i|i.name=="My Fabric").unwrap();
        assert_eq!(fabric.directory(),"My Fabric"); assert_eq!(fabric.minecraft_version,"1.21.1");
        assert_eq!(fabric.loader.as_ref().unwrap().kind,"fabric");
        assert!(launcher_core::INSTANCE_ICONS.contains(&fabric.icon.as_str()));
        discover(&storage).unwrap(); let again=storage.store().load().unwrap();
        assert_eq!(again.revision,first.revision); assert_eq!(again.instances[0].icon,first.instances[0].icon);
        assert_eq!(fs::read(storage.location.root.join("versions/My Fabric/options.txt")).unwrap(),b"keep");
        assert!(!storage.location.root.join("versions/My Fabric/launcher-install.json").exists());
        fs::remove_dir_all(base).unwrap();
    }
}
