//! Per-instance file availability, independent of in-game pack selection/settings.
use crate::{err, safe_path, Result};
use serde::{Deserialize, Serialize};
use std::{fs, path::{Path, PathBuf}};

#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Kind { Mods, ResourcePacks, Shaders }
impl Kind {
    pub fn directory(self) -> &'static str {
        match self { Self::Mods => "mods", Self::ResourcePacks => "resourcepacks", Self::Shaders => "shaderpacks" }
    }
    fn accepts(self, path: &Path) -> bool {
        if path.is_file() {
            return path.extension().and_then(|e|e.to_str()).is_some_and(|e|
                e.eq_ignore_ascii_case(if matches!(self, Self::Mods) {"jar"} else {"zip"}));
        }
        if !path.is_dir() { return false; }
        // Never traverse links inside unpacked packs while inspecting metadata.
        match self {
            Self::Mods => false,
            Self::ResourcePacks => safe_path(path,"pack.mcmeta").is_ok_and(|p|p.is_file()),
            Self::Shaders => safe_path(path,"shaders").is_ok_and(|p|p.is_dir()),
        }
    }
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Entry { pub name: String, pub enabled: bool, pub conflict: bool }
#[derive(Serialize)]
pub struct Listing { pub directory: PathBuf, pub entries: Vec<Entry> }
fn directories(root: &Path, kind: Kind) -> Result<(PathBuf,PathBuf)> {
    Ok((safe_path(root,kind.directory())?, safe_path(root,&format!(".launcher-disabled/{}",kind.directory()))?))
}
pub fn scan(root: &Path, kind: Kind, create: bool) -> Result<Listing> {
    let active=safe_path(root,kind.directory())?;
    if create { fs::create_dir_all(&active).map_err(err)?; }
    let mut entries: Vec<Entry>=vec![];
    let mut sources=vec![(active.clone(),true)];
    if matches!(kind,Kind::Mods) {sources.push((safe_path(root,".launcher-disabled/mods")?,false));}
    for (directory,enabled) in sources {
        let items=match fs::read_dir(&directory) {
            Ok(items)=>items,
            Err(e) if e.kind()==std::io::ErrorKind::NotFound=>continue,
            Err(e)=>return Err(err(e)),
        };
        for item in items {
            let item=item.map_err(err)?;
            if item.file_type().map_err(err)?.is_symlink() {continue}
            let Ok(name)=item.file_name().into_string() else {continue};
            if safe_path(&directory,&name).is_err() || !kind.accepts(&item.path()) {continue}
            if let Some(existing)=entries.iter_mut().find(|e|e.name==name) {
                existing.conflict=true;
            } else { entries.push(Entry{name,enabled,conflict:false}); }
        }
    }
    entries.sort_by(|a,b|a.name.to_lowercase().cmp(&b.name.to_lowercase()).then(a.name.cmp(&b.name)));
    Ok(Listing{directory:active,entries})
}
pub fn set_enabled(root: &Path, kind: Kind, name: &str, enabled: bool) -> Result<Listing> {
    if !matches!(kind,Kind::Mods) {return Err("Only mods support file enable and disable".into());}
    if name.contains('/') || name.is_empty() {return Err("Invalid content file selection".into());}
    let (active,disabled)=directories(root,kind)?;
    let (source,destination)=if enabled {(&disabled,&active)} else {(&active,&disabled)};
    let source_path=safe_path(source,name)?;
    let destination_path=safe_path(destination,name)?;
    if !kind.accepts(&source_path) {return Err("Content file changed; refresh the list".into());}
    fs::create_dir_all(destination).map_err(err)?;
    move_without_replace(&source_path,&destination_path)?;
    scan(root,kind,false)
}
/// Permanently remove exactly the displayed file; disabled mods stay addressable.
pub fn delete(root: &Path, kind: Kind, name: &str, enabled: bool) -> Result<Listing> {
    if name.is_empty() || name.contains('/') || (!enabled && !matches!(kind,Kind::Mods)) {
        return Err("Invalid content file selection".into());
    }
    let directory=if enabled {safe_path(root,kind.directory())?} else {safe_path(root,".launcher-disabled/mods")?};
    let path=safe_path(&directory,name)?;
    if !kind.accepts(&path) {return Err("Content file changed; refresh the list".into());}
    if path.is_dir() {fs::remove_dir_all(&path).map_err(err)?;} else {fs::remove_file(&path).map_err(err)?;}
    scan(root,kind,false)
}
pub(crate) use launcher_platform::move_without_replace;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn creates_scans_and_restores_without_changing_file_bytes() {
        let root=tempfile::tempdir().unwrap();
        for kind in [Kind::Mods] {
            let listing=scan(root.path(),kind,true).unwrap();
            let name=if matches!(kind,Kind::Mods) {"example.jar"} else {"example.zip"};
            fs::write(listing.directory.join(name),b"unchanged").unwrap();
            fs::write(listing.directory.join("notes.txt"),b"ignore").unwrap();
            assert_eq!(scan(root.path(),kind,false).unwrap().entries.len(),1);
            assert!(!set_enabled(root.path(),kind,name,false).unwrap().entries[0].enabled);
            assert!(!listing.directory.join(name).exists());
            assert!(set_enabled(root.path(),kind,name,true).unwrap().entries[0].enabled);
            assert_eq!(fs::read(listing.directory.join(name)).unwrap(),b"unchanged");
        }
    }
    #[test]
    fn collision_traversal_and_symlinks_do_not_replace_files() {
        let root=tempfile::tempdir().unwrap();
        scan(root.path(),Kind::Mods,true).unwrap();
        fs::write(root.path().join("mods/a.jar"),b"first").unwrap();
        set_enabled(root.path(),Kind::Mods,"a.jar",false).unwrap();
        fs::write(root.path().join("mods/a.jar"),b"second").unwrap();
        assert!(scan(root.path(),Kind::Mods,false).unwrap().entries[0].conflict);
        assert!(set_enabled(root.path(),Kind::Mods,"a.jar",true).is_err());
        assert_eq!(fs::read(root.path().join("mods/a.jar")).unwrap(),b"second");
        assert!(set_enabled(root.path(),Kind::Mods,"../a.jar",true).is_err());
        #[cfg(unix)] {
            std::os::unix::fs::symlink(root.path().join("mods/a.jar"),root.path().join("mods/link.jar")).unwrap();
            assert_eq!(scan(root.path(),Kind::Mods,false).unwrap().entries.len(),1);
            assert!(set_enabled(root.path(),Kind::Mods,"link.jar",false).is_err());
        }
    }
    #[test]
    fn deletes_files_and_pack_directories_without_backup() {
        let root=tempfile::tempdir().unwrap();
        for kind in [Kind::Mods,Kind::ResourcePacks,Kind::Shaders] {
            let listing=scan(root.path(),kind,true).unwrap();
            let name=if matches!(kind,Kind::Mods) {"delete.jar"} else {"delete.zip"};
            fs::write(listing.directory.join(name),b"delete").unwrap();
            fs::write(listing.directory.join("keep.txt"),b"keep").unwrap();
            assert!(delete(root.path(),kind,"../keep.txt",true).is_err());
            assert!(delete(root.path(),kind,name,true).unwrap().entries.is_empty());
            assert!(listing.directory.join("keep.txt").exists());
            assert!(!listing.directory.join(name).exists());
        }
        assert!(!root.path().join(".launcher-disabled").exists());
        fs::write(root.path().join("mods/disabled.jar"),b"mod").unwrap();
        set_enabled(root.path(),Kind::Mods,"disabled.jar",false).unwrap();
        assert!(delete(root.path(),Kind::Mods,"disabled.jar",false).unwrap().entries.is_empty());
        fs::create_dir_all(root.path().join("shaderpacks/Pack/shaders")).unwrap();
        #[cfg(unix)] {
            std::os::unix::fs::symlink(root.path().join("mods/keep.txt"),root.path().join("shaderpacks/Pack/external")).unwrap();
        }
        delete(root.path(),Kind::Shaders,"Pack",true).unwrap();
        assert!(root.path().join("mods/keep.txt").exists());
        assert!(!root.path().join("shaderpacks/Pack").exists());
    }
    #[test]
    fn unpacked_packs_and_read_only_scan() {
        let root=tempfile::tempdir().unwrap();
        assert!(scan(root.path(),Kind::Shaders,false).unwrap().entries.is_empty());
        assert!(!root.path().join("shaderpacks").exists());
        fs::create_dir_all(root.path().join("shaderpacks/Example/shaders")).unwrap();
        assert_eq!(scan(root.path(),Kind::Shaders,false).unwrap().entries.len(),1);
        fs::create_dir_all(root.path().join("resourcepacks/Example")).unwrap();
        fs::write(root.path().join("resourcepacks/Example/pack.mcmeta"),b"{}").unwrap();
        assert_eq!(scan(root.path(),Kind::ResourcePacks,false).unwrap().entries.len(),1);
        for kind in [Kind::ResourcePacks,Kind::Shaders] {
            assert!(set_enabled(root.path(),kind,"Example",false).is_err());
            assert!(set_enabled(root.path(),kind,"Example",true).is_err());
            assert!(root.path().join(kind.directory()).join("Example").is_dir());
        }
        assert!(!root.path().join(".launcher-disabled").exists());
    }
}
