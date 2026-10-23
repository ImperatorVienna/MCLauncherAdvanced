//! Provider adapters. Every request keeps the exact instance version/loader.
//! Search hits are revalidated against file metadata before installation.
use crate::{content::Kind, download, err, safe_path, Result};
use launcher_core::Instance;
use reqwest::blocking::Client;
use serde::{Serialize,Deserialize};
use serde_json::{json,Value};
use sha1::{Digest,Sha1};
use std::{fs,io::Read,path::Path,time::Duration};

#[derive(Serialize)]
#[serde(rename_all="camelCase")]
pub struct Project { pub id:String,pub title:String,pub description:String }
#[derive(Serialize)]
pub struct Search { pub projects:Vec<Project>,pub more:bool }
#[derive(Clone,Debug)]
pub struct Release { pub project:String,pub version:String,pub name:String,pub url:String,pub hash:String,pub size:u64,pub published:String,pub dependencies:Vec<String>,pub loaders:Vec<String> }
pub struct Provider { client:Client }
impl Provider {
    pub fn new()->Result<Self> {
        Ok(Self{client:Client::builder().user_agent("mclauncher-advanced/0.1.0").timeout(Duration::from_secs(30)).redirect(reqwest::redirect::Policy::none()).build().map_err(err)?})
    }
    fn api(&self,path:&str,params:&[(&str,String)],body:Option<Value>)->Result<Value> {
        let mut url=reqwest::Url::parse(&format!("https://api.modrinth.com/v2{path}")).map_err(err)?;
        url.query_pairs_mut().extend_pairs(params.iter().map(|(k,v)|(*k,v.as_str())));
        let request=if let Some(body)=body {self.client.post(url).json(&body)} else {self.client.get(url)};
        let response=request.send().map_err(err)?;
        if !response.status().is_success() {return Err(format!("Content provider returned HTTP {}",response.status().as_u16()));}
        let mut bytes=vec![];
        response.take(16*1024*1024+1).read_to_end(&mut bytes).map_err(err)?;
        if bytes.len()>16*1024*1024 {return Err("Metadata exceeds the size limit".into());}
        serde_json::from_slice(&bytes).map_err(err)
    }
    pub fn supported(&self,item:&Instance,kind:Kind)->Result<()> {
        if matches!(kind,Kind::Mods) && item.loader.is_none() {return Err("Select an instance with a mod loader to browse mods".into());}
        let versions=self.api("/tag/game_version",&[],None)?;
        let values=versions.as_array();
        let found=values.is_some_and(|v|v.iter().any(|v| v["version"].as_str()==Some(&item.minecraft_version)));
        if !found {return Err("This source does not support this game version; install files manually".into());}
        Ok(())
    }
    pub fn search(&self,item:&Instance,kind:Kind,query:&str,offset:u32)->Result<Search> {
        self.supported(item,kind)?;
        if query.len()>200 || offset>9900 {return Err("Invalid content search".into());}
        {
            let mut facets=vec![vec![format!("project_type:{}",match kind {Kind::Mods=>"mod",Kind::ResourcePacks=>"resourcepack",Kind::Shaders=>"shader"})],vec![format!("versions:{}",item.minecraft_version)]];
            if matches!(kind,Kind::Mods) {facets.push(vec![format!("categories:{}",item.loader.as_ref().unwrap().kind)]);}
            let data=self.api("/search",&[("query",query.into()),("facets",json!(facets).to_string()),("offset",offset.to_string()),("limit","20".into())],None)?;
            let projects=array(&data["hits"])?.iter().map(|p|Ok(Project{id:string(p,"project_id")?,title:string(p,"title")?,description:string(p,"description")?})).collect::<Result<Vec<_>>>()?;
            Ok(Search{more:offset as u64+(projects.len() as u64)<data["total_hits"].as_u64().unwrap_or(0),projects})
        }
    }
    pub fn latest(&self,item:&Instance,kind:Kind,project:&str)->Result<Release> {
        valid_id(project)?;
        {
            let project_data=self.api(&format!("/project/{project}"),&[],None)?;
            let expected=match kind {Kind::Mods=>"mod",Kind::ResourcePacks=>"resourcepack",Kind::Shaders=>"shader"};
            if project_data["project_type"]!=expected {return Err("Content type does not match the destination".into());}
            let mut params=vec![("game_versions",json!([item.minecraft_version]).to_string())];
            if matches!(kind,Kind::Mods) {params.push(("loaders",json!([item.loader.as_ref().ok_or("Select an instance with a mod loader to browse mods")?.kind]).to_string()));}
            let data=self.api(&format!("/project/{project}/version"),&params,None)?;
            let mut versions=array(&data)?.iter().filter(|v|compatible(v,item,kind)).collect::<Vec<_>>();
            versions.sort_by_key(|v|std::cmp::Reverse(v["date_published"].as_str().unwrap_or("")));
            mr_release(versions.first().ok_or("No compatible downloadable version was found")?)
        }
    }
    fn dependency(&self,item:&Instance,kind:Kind,id:&str,parent_loaders:&[String],local:&[(String,bool,String)],identities:&serde_json::Map<String,Value>)->Result<Option<Release>> {
        if let Some(id)=id.strip_prefix("project:") {
            valid_id(id)?;
            if dependency_present(id,local,identities) {return Ok(None);}
            // A universal release may list requirements for all its loader targets.
            // Exclude only projects explicitly confined to another parent loader.
            // Missing versions of projects supporting this loader still fail normally.
            if matches!(kind,Kind::Mods) && parent_loaders.len()>1 {
                let project=self.api(&format!("/project/{id}"),&[],None)?;
                if other_loader_dependency(&project,item,parent_loaders) {return Ok(None);}
            }
            return self.latest(item,kind,id).map(Some).map_err(|e|format!("Required dependency {id} could not be resolved: {e}"));
        }
        let id=id.strip_prefix("version:").ok_or("Invalid provider project")?;
        valid_id(id)?;
        let data=self.api(&format!("/version/{id}"),&[],None)?;
        let project=string(&data,"project_id")?;
        if dependency_present(&project,local,identities) {return Ok(None);}
        if !compatible(&data,item,kind) {return Err("A required dependency is incompatible with this instance".into());}
        mr_release(&data).map(Some)
    }
    fn identities(&self,hashes:&[String])->Result<Value> {
        if hashes.is_empty(){return Ok(json!({}));}
        self.api("/version_files",&[],Some(json!({"hashes":hashes,"algorithm":"sha1"})))
    }
    pub fn identify(&self,path:&Path)->Result<Option<Release>> {
        let hash=content_hash(path)?;
        let data=self.api("/version_files",&[],Some(json!({"hashes":[hash],"algorithm":"sha1"})))?;
        data.get(&hash).map(mr_release).transpose()
    }

}
fn array(v:&Value)->Result<&Vec<Value>> {v.as_array().ok_or_else(||"Invalid content provider response".into())}
fn string(v:&Value,key:&str)->Result<String>{v[key].as_str().map(str::to_owned).ok_or_else(||"Invalid content provider response".into())}
fn valid_id(value:&str)->Result<()> {if value.is_empty() || value.len()>100 || !value.bytes().all(|b|b.is_ascii_alphanumeric()||b==b'-'||b==b'_') {Err("Invalid provider project".into())} else {Ok(())}}
fn compatible(v:&Value,item:&Instance,kind:Kind)->bool {
    v["game_versions"].as_array().is_some_and(|a|a.iter().any(|v|v==&item.minecraft_version)) &&
    (!matches!(kind,Kind::Mods) || item.loader.as_ref().is_some_and(|l|v["loaders"].as_array().is_some_and(|a|a.iter().any(|v|v==&l.kind))))
}
// Modrinth project facets aggregate loader support across versions. Use them
// only to rule out another loader's requirement, never to relax file matching.
fn other_loader_dependency(project:&Value,item:&Instance,parent_loaders:&[String])->bool {
    let Some(loader)=item.loader.as_ref() else {return false;};
    if !parent_loaders.contains(&loader.kind) || parent_loaders.len()<2 {return false;}
    let Some(loaders)=project["loaders"].as_array() else {return false;};
    !loaders.is_empty() && loaders.iter().all(|value|value.as_str().is_some_and(|other|
        other!=loader.kind && parent_loaders.iter().any(|parent|parent==other)))
}
// Existing dependencies are user-managed. Provider version pins choose what to
// fetch only when a dependency is absent; they must not force a replacement.
fn dependency_present(project:&str,local:&[(String,bool,String)],identities:&serde_json::Map<String,Value>)->bool {
    local.iter().any(|(_,_,hash)|identities.get(hash).is_some_and(|v|v["project_id"]==project))
}
fn mr_release(v:&Value)->Result<Release> {
    let files=array(&v["files"])?;
    let f=files.iter().find(|f|f["primary"]==true).or(files.first()).ok_or("No downloadable content file")?;
    Ok(Release{project:string(v,"project_id")?,version:string(v,"id")?,name:string(f,"filename")?,url:string(f,"url")?,hash:string(&f["hashes"],"sha1")?,size:f["size"].as_u64().ok_or("Invalid content file size")?,published:string(v,"date_published")?,
        loaders: v["loaders"].as_array().map(|a|a.iter().filter_map(|v|v.as_str().map(str::to_owned)).collect()).unwrap_or_default(),
        dependencies: v["dependencies"].as_array().map(|a|a.iter().filter(|d|d["dependency_type"]=="required").map(|d|d["version_id"].as_str().map(|v|format!("version:{v}")).or_else(||d["project_id"].as_str().map(|v|format!("project:{v}"))).ok_or_else(||"A required dependency cannot be resolved; install it manually".into())).collect::<Result<Vec<_>>>()).transpose()?.unwrap_or_default()})
}
// Hash archives incrementally: high-resolution packs can exceed the former
// 256 MiB inspection cap, and should not block unrelated files in the instance.
fn content_hash(path:&Path)->Result<String> {
    let mut file=fs::File::open(path).map_err(err)?;
    let mut hash=Sha1::new();
    let mut buffer=[0u8;64*1024];
    loop {
        let count=file.read(&mut buffer).map_err(err)?;
        if count==0 {break;}
        hash.update(&buffer[..count]);
    }
    Ok(format!("{:x}",hash.finalize()))
}
pub fn file_spec(release:&Release,kind:Kind)->Result<download::File> {
    if release.name.contains('/') || release.name.contains('\\') || !release.name.to_ascii_lowercase().ends_with(if matches!(kind,Kind::Mods){".jar"} else {".zip"}) {return Err("Invalid content file selection".into());}
    safe_path(Path::new("."),&release.name)?;
    if reqwest::Url::parse(&release.url).map_err(err)?.host_str()!=Some("cdn.modrinth.com") {return Err("Untrusted download URL".into());}
    let f=download::File{path:release.name.clone(),url:release.url.clone(),hash:release.hash.clone(),size:release.size};f.validate()?;Ok(f)
}

#[derive(Clone,Deserialize,Serialize)]
#[serde(rename_all="camelCase")]
pub struct Update { pub name:String,pub enabled:bool,pub hash:String,pub project:String,pub version:String }
#[derive(Serialize)]
pub struct Updates { pub updates:Vec<Update>,pub unmatched:usize }
fn content_path(root:&Path,kind:Kind,name:&str,enabled:bool)->Result<std::path::PathBuf> {
    if name.contains('/') || name.contains('\\') || !name.to_ascii_lowercase().ends_with(if matches!(kind,Kind::Mods){".jar"}else{".zip"}) || (!enabled && !matches!(kind,Kind::Mods)) {return Err("Invalid content file selection".into());}
    let directory=safe_path(root,if enabled {kind.directory()} else {".launcher-disabled/mods"})?;
    safe_path(&directory,name)
}
fn local_files(root:&Path,kind:Kind)->Result<Vec<(String,bool,String)>> {
    let mut files=vec![];
    for e in crate::content::scan(root,kind,false)?.entries {
        if e.conflict || safe_path(&safe_path(root,kind.directory())?,&e.name)?.is_dir(){continue;}
        let hash=content_hash(&content_path(root,kind,&e.name,e.enabled)?)?;
        files.push((e.name,e.enabled,hash));
    }
    Ok(files)
}
/// Keep cached update suggestions only while their original local bytes still exist.
pub fn retain_updates(root:&Path,kind:Kind,candidates:Vec<Update>)->Result<Vec<Update>> {
    let entries=crate::content::scan(root,kind,false)?.entries;
    let mut retained=vec![];
    for mut update in candidates {
        let Some(entry)=entries.iter().find(|e|e.name==update.name && !e.conflict) else {continue;};
        let path=content_path(root,kind,&entry.name,entry.enabled)?;
        if path.is_file() && content_hash(&path)?==update.hash {
            update.enabled=entry.enabled;
            retained.push(update);
        }
    }
    Ok(retained)
}
#[derive(Serialize)]
#[serde(rename_all="camelCase")]
pub struct ContentLabel {pub name:String,pub display_name:String}
/// Cosmetic metadata only: filenames remain the identity for all local operations.
pub fn labels(root:&Path,kind:Kind)->Result<Vec<ContentLabel>> {
    let local=local_files(root,kind)?;
    if local.is_empty(){return Ok(vec![]);}
    let provider=Provider::new()?;
    let mut labels=vec![];
    let mut titles=std::collections::HashMap::<String,String>::new();
    for chunk in local.chunks(100) {
        let hashes=chunk.iter().map(|(_,_,hash)|hash.clone()).collect::<Vec<_>>();
        let versions=provider.identities(&hashes)?;
        for (name,_,hash) in chunk {
            let Some(version)=versions.get(hash) else {continue};
            let project=string(version,"project_id")?;
            if !titles.contains_key(&project) {
                valid_id(&project)?;
                let data=provider.api(&format!("/project/{project}"),&[],None)?;
                let expected=match kind {Kind::Mods=>"mod",Kind::ResourcePacks=>"resourcepack",Kind::Shaders=>"shader"};
                if data["project_type"]!=expected {continue;}
                titles.insert(project.clone(),string(&data,"title")?);
            }
            labels.push(ContentLabel{name:name.clone(),display_name:format!("{}_{}",titles[&project],string(version,"version_number")?)});
        }
    }
    Ok(labels)
}
pub fn updates(root:&Path,item:&Instance,kind:Kind)->Result<Updates> {
    let provider=Provider::new()?;provider.supported(item,kind)?;
    let local=local_files(root,kind)?;
    let mut result=Updates{updates:vec![],unmatched:crate::content::scan(root,kind,false)?.entries.len().saturating_sub(local.len())};
    for chunk in local.chunks(100) {
        let hashes=chunk.iter().map(|(_,_,h)|h.clone()).collect::<Vec<_>>();
        let current=provider.identities(&hashes)?;
        let mut request=json!({"hashes":hashes,"algorithm":"sha1","game_versions":[item.minecraft_version]});
        request["loaders"]=match kind {Kind::Mods=>json!([item.loader.as_ref().unwrap().kind]),Kind::ResourcePacks=>json!(["minecraft"]),Kind::Shaders=>json!(["iris","optifine","canvas"])};
        let latest=provider.api("/version_files/update",&[],Some(request))?;
        for (name,enabled,hash) in chunk {
            let Some(old)=current.get(hash) else {result.unmatched+=1;continue};
            if let Some(new)=latest.get(hash) {
                if compatible(new,item,kind) && new["project_id"]==old["project_id"] && new["id"]!=old["id"] &&
                    new["date_published"].as_str()>old["date_published"].as_str() {
                    let release=mr_release(new)?;
                    file_spec(&release,kind)?;
                    result.updates.push(Update{name:name.clone(),enabled:*enabled,hash:hash.clone(),project:release.project,version:release.version});
                }
            }
        }
    }
    Ok(result)
}
#[derive(Serialize)]
pub struct CatalogState {pub project:String,pub installed:bool,pub update:Option<Update>}
fn catalog_state(project:&str,local:&[(String,bool,String)],identities:&serde_json::Map<String,Value>,latest:Option<&Release>)->CatalogState {
    let matches=local.iter().filter(|(_,_,hash)|identities.get(hash).is_some_and(|v|v["project_id"]==project)).collect::<Vec<_>>();
    let update=if matches.len()==1 {
        let (name,enabled,hash)=matches[0];
        latest.filter(|release|release.project==project && identities[hash]["id"]!=release.version && identities[hash]["date_published"].as_str().is_some_and(|date|date<release.published.as_str()))
            .map(|release|Update{name:name.clone(),enabled:*enabled,hash:hash.clone(),project:project.into(),version:release.version.clone()})
    } else {None};
    CatalogState{project:project.into(),installed:!matches.is_empty(),update}
}
/// Identify local archives, including disabled mods, independently of filenames.
/// Duplicate versions are treated as installed; never choose one to replace arbitrarily.
pub fn catalog_states(root:&Path,item:&Instance,kind:Kind,projects:&[String])->Result<Vec<CatalogState>> {
    if projects.len()>20 {return Err("Invalid content search".into());}
    for project in projects {valid_id(project)?;}
    let local=local_files(root,kind)?;
    let provider=Provider::new()?;
    let mut identities=serde_json::Map::new();
    for chunk in local.chunks(100) {
        let values=provider.identities(&chunk.iter().map(|(_,_,hash)|hash.clone()).collect::<Vec<_>>())?;
        identities.extend(values.as_object().ok_or("Invalid content provider response")?.clone());
    }
    projects.iter().map(|project| {
        let installed=dependency_present(project,&local,&identities);
        let latest=if installed {Some(provider.latest(item,kind,project)?)}else{None};
        Ok(catalog_state(project,&local,&identities,latest.as_ref()))
    }).collect()
}
pub fn install(root:&Path,item:&Instance,kind:Kind,project:&str,update:Option<Update>,cancel:&std::sync::atomic::AtomicBool,report:&crate::Report<'_>)->Result<()> {
    let provider=Provider::new()?;provider.supported(item,kind)?;
    crate::check_cancel(cancel)?;
    let selected=provider.latest(item,kind,project)?;
    let old_path=if let Some(update)=&update {
        if update.project!=project || update.version!=selected.version {return Err("Update selection changed; check for updates again".into());}
        let path=content_path(root,kind,&update.name,update.enabled)?;
        if content_hash(&path)?!=update.hash {return Err("Content file changed; refresh the list".into());}
        let old=provider.identify(&path)?.ok_or("Content file is not recognized by Modrinth")?;
        if old.project!=selected.project || old.published>=selected.published {return Err("Update selection changed; check for updates again".into());}
        Some(path)
    } else {None};
    // Resolve dependencies before any visible changes. Leave existing dependencies
    // untouched, including their versions and enabled state; fetch missing projects.
    let local=local_files(root,kind)?;
    let mut identities=serde_json::Map::new();
    for chunk in local.chunks(100) {
        if let Value::Object(values)=provider.identities(&chunk.iter().map(|(_,_,h)|h.clone()).collect::<Vec<_>>())? {identities.extend(values);}
    }
    let mut plan:Vec<Release>=vec![];
    let mut queue=vec![selected];
    let mut seen=std::collections::HashMap::<String,String>::new();
    while let Some(release)=queue.pop() {
        crate::check_cancel(cancel)?;
        if let Some(version)=seen.get(&release.project) {
            if version!=&release.version {return Err("Required dependency versions conflict; install manually".into());}
            continue;
        }
        seen.insert(release.project.clone(),release.version.clone());
        if seen.len()>64 {return Err("Too many required dependencies".into());}
        let is_target=release.project==project;
        let existing=local.iter().find(|(_,_,hash)|identities.get(hash).is_some_and(|v|v["project_id"]==release.project));
        if let Some((name,enabled,hash))=existing {
            if !(is_target && update.is_some()) {
                let metadata=&identities[hash];
                if metadata["id"]==release.version && *enabled {continue;}
                return Err(format!("A version of this project already exists: {name}; update or remove it first"));
            }
        }
        for dependency in &release.dependencies {
            if let Some(missing)=provider.dependency(item,kind,dependency,&release.loaders,&local,&identities)? {queue.push(missing);}
        }
        file_spec(&release,kind)?;
        plan.push(release);
    }
    let active=safe_path(root,kind.directory())?;
    fs::create_dir_all(&active).map_err(err)?;
    let stage=tempfile::tempdir_in(root).map_err(err)?;
    let mut files=vec![];
    let mut targets=vec![];
    for (index,release) in plan.iter().enumerate() {
        let directory=if update.as_ref().is_some_and(|u|!u.enabled && u.project==release.project) {safe_path(root,".launcher-disabled/mods")?} else {active.clone()};
        fs::create_dir_all(&directory).map_err(err)?;
        let target=safe_path(&directory,&release.name)?;
        if targets.contains(&target) {return Err("Downloaded files have conflicting names".into());}
        let mut spec=file_spec(release,kind)?;
        if target.exists() && Some(&target)!=old_path.as_ref() {
            if download::valid_file(&target,&spec,cancel)? {continue;}
            return Err("A file with this name already exists in the destination; no files were replaced".into());
        }
        spec.path=index.to_string();
        files.push(spec);targets.push(target);
    }
    download::download_all(&download::client()?,stage.path(),&files,cancel,report,"Downloading content files")?;
    crate::check_cancel(cancel)?;
    // Recheck the old bytes immediately before replacing; cancellation cannot
    // interrupt the short commit/rollback section.
    if let (Some(path),Some(update))=(&old_path,&update) {
        if content_hash(path)?!=update.hash {return Err("Content file changed; refresh the list".into());}
    }
    commit_files(stage,&files,&targets,old_path.as_deref())
}
fn commit_files(stage:tempfile::TempDir,files:&[download::File],targets:&[std::path::PathBuf],old_path:Option<&Path>)->Result<()> {
    let parked=stage.path().join("original");
    if let Some(path)=old_path {crate::content::move_without_replace(path,&parked)?;}
    let mut committed=vec![];
    let result=(|| {
        for (spec,target) in files.iter().zip(targets.iter()) {
            crate::content::move_without_replace(&stage.path().join(&spec.path),target)?;
            committed.push(target.clone());
        }
        Ok(())
    })();
    if let Err(error)=result {
        let mut cleanup_error=None;
        for path in &committed {if let Err(e)=fs::remove_file(path) {cleanup_error=Some(err(e));}}
        if let Some(path)=old_path {
            if let Err(restore)=crate::content::move_without_replace(&parked,path) {
                let recovery=stage.keep();
                return Err(format!("Content rollback failed: {restore}; original file retained at {}",recovery.display()));
            }
        }
        if let Some(cleanup)=cleanup_error {return Err(format!("Content rollback cleanup failed: {cleanup}; update failed: {error}"));}
        return Err(error);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cached_updates_survive_other_updates_for_all_content_kinds() {
        for kind in [Kind::Mods,Kind::ResourcePacks,Kind::Shaders] {
            let root=tempfile::tempdir().unwrap();let dir=root.path().join(kind.directory());
            fs::create_dir_all(&dir).unwrap();
            let ext=if matches!(kind,Kind::Mods){"jar"}else{"zip"};
            let names=[format!("first.{ext}"),format!("second.{ext}")];
            for name in &names {fs::write(dir.join(name),b"original").unwrap();}
            let candidates=names.iter().map(|name|Update{name:name.clone(),enabled:true,hash:content_hash(&dir.join(name)).unwrap(),project:name.clone(),version:"next".into()}).collect::<Vec<_>>();
            assert_eq!(retain_updates(root.path(),kind,candidates.clone()).unwrap().len(),2);
            fs::write(dir.join(&names[0]),b"replacement").unwrap();
            if matches!(kind,Kind::Mods) {
                let disabled=root.path().join(".launcher-disabled/mods");fs::create_dir_all(&disabled).unwrap();
                fs::rename(dir.join(&names[1]),disabled.join(&names[1])).unwrap();
            }
            let remaining=retain_updates(root.path(),kind,candidates).unwrap();
            assert_eq!(remaining.len(),1);assert_eq!(remaining[0].name,names[1]);
            assert_eq!(remaining[0].enabled,!matches!(kind,Kind::Mods));
        }
    }
    #[test]
    fn catalog_actions_preserve_disabled_files_and_never_downgrade_or_choose_duplicates() {
        let local=vec![("renamed.jar".into(),false,"hash".into())];
        let identities=json!({"hash":{"project_id":"p","id":"v1","date_published":"2026-01-01"}}).as_object().unwrap().clone();
        let mut latest=Release{project:"p".into(),version:"v2".into(),name:"new.jar".into(),url:String::new(),hash:String::new(),size:0,published:"2026-02-01".into(),dependencies:vec![],loaders:vec![]};
        let state=catalog_state("p",&local,&identities,Some(&latest));
        assert!(state.installed);let update=state.update.unwrap();assert!(!update.enabled);assert_eq!(update.name,"renamed.jar");
        assert!(!catalog_state("absent",&local,&identities,None).installed);
        latest.published="2025-01-01".into();assert!(catalog_state("p",&local,&identities,Some(&latest)).update.is_none());
        latest.published="2026-02-01".into();latest.version="v1".into();assert!(catalog_state("p",&local,&identities,Some(&latest)).update.is_none());
        latest.version="v2".into();let mut duplicate=local.clone();duplicate.push(("duplicate.jar".into(),true,"hash".into()));
        assert!(catalog_state("p",&duplicate,&identities,Some(&latest)).update.is_none());
    }
    fn instance()->Instance {Instance{id:"test".into(),name:"Test".into(),minecraft_version:"1.21.1".into(),icon:"grass".into(),loader:Some(launcher_core::ModLoader{kind:"fabric".into(),version:"0.16.0".into()}),directory_name:"test".into(),java_path:None,launch_settings:Default::default()}}
    #[test]
    fn update_commits_new_filename_without_touching_configuration() {
        let root=tempfile::tempdir().unwrap();let path=root.path().join("options.txt");
        fs::write(&path,"resourcePacks:[\"vanilla\",\"file/old.zip\"]").unwrap();
        let old=root.path().join("old.zip");let new=root.path().join("new.zip");fs::write(&old,b"original").unwrap();
        let stage=tempfile::tempdir_in(root.path()).unwrap();fs::write(stage.path().join("0"),b"updated").unwrap();
        let file=download::File{path:"0".into(),url:String::new(),hash:String::new(),size:7};
        commit_files(stage,&[file],&[new.clone()],Some(&old)).unwrap();
        assert!(!old.exists());assert_eq!(fs::read(new).unwrap(),b"updated");
        assert_eq!(fs::read_to_string(path).unwrap(),"resourcePacks:[\"vanilla\",\"file/old.zip\"]");
    }
    #[test]
    fn update_rolls_back_on_destination_conflict() {
        let root=tempfile::tempdir().unwrap();
        let path=root.path().join("options.txt");
        fs::write(&path,"resourcePacks:[\"file/old.zip\"]").unwrap();
        let old=root.path().join("old.zip");let new=root.path().join("new.zip");
        fs::write(&old,b"original").unwrap();
        let stage=tempfile::tempdir_in(root.path()).unwrap();fs::write(stage.path().join("0"),b"updated").unwrap();
        fs::write(&new,b"external").unwrap();
        fs::write(&path,"resourcePacks:[]").unwrap();
        let file=download::File{path:"0".into(),url:String::new(),hash:String::new(),size:7};
        assert!(commit_files(stage,&[file],&[new.clone()],Some(&old)).is_err());
        assert_eq!(fs::read(old).unwrap(),b"original");assert_eq!(fs::read(new).unwrap(),b"external");
        assert_eq!(fs::read_to_string(path).unwrap(),"resourcePacks:[]");
    }
    #[test]
    fn existing_dependency_versions_and_enabled_states_do_not_block_updates() {
        // Deliberately different versions/loaders: presence alone is the policy.
        let identities=serde_json::Map::from_iter([("hash".into(),json!({"id":"installed-beta","project_id":"sodium","game_versions":["older"],"loaders":["other"]}))]);
        let mut local=vec![("renamed.jar".into(),true,"hash".into())];
        assert!(dependency_present("sodium",&local,&identities));
        assert!(!dependency_present("missing",&local,&identities));
        local[0].1=false;
        assert!(dependency_present("sodium",&local,&identities));
        local.push(("another-version.jar".into(),true,"hash".into()));
        assert!(dependency_present("sodium",&local,&identities));
        assert!(!dependency_present("sodium",&local,&serde_json::Map::new()));
    }
    #[test]
    fn all_content_kinds_skip_existing_dependencies_without_network_requests() {
        let provider=Provider::new().unwrap();let item=instance();
        for kind in [Kind::Mods,Kind::ResourcePacks,Kind::Shaders] {
            let root=tempfile::tempdir().unwrap();let directory=root.path().join(kind.directory());
            fs::create_dir_all(&directory).unwrap();
            let name=if matches!(kind,Kind::Mods) {"renamed.jar"} else {"renamed.zip"};
            fs::write(directory.join(name),b"existing dependency").unwrap();
            let local=local_files(root.path(),kind).unwrap();
            let identities=serde_json::Map::from_iter([(local[0].2.clone(),json!({"project_id":"dependency","id":"different-version"}))]);
            assert!(provider.dependency(&item,kind,"project:dependency",&[],&local,&identities).unwrap().is_none());
            assert_eq!(fs::read(directory.join(name)).unwrap(),b"existing dependency");
            if matches!(kind,Kind::Mods) {
                crate::content::set_enabled(root.path(),kind,name,false).unwrap();
                let local=local_files(root.path(),kind).unwrap();
                assert!(provider.dependency(&item,kind,"project:dependency",&[],&local,&identities).unwrap().is_none());
                assert!(!directory.join(name).exists());
            }
        }
    }
    #[test]
    #[ignore = "Uses live Modrinth Iris/Sodium metadata and downloads into a temporary instance"]
    fn live_iris_update_keeps_installed_sodium() {
        let mut item=instance();item.minecraft_version="1.21.11".into();item.loader.as_mut().unwrap().kind="neoforge".into();
        let provider=Provider::new().unwrap();let root=tempfile::tempdir().unwrap();
        let old=mr_release(&provider.api("/version/v6TgIIUM",&[],None).unwrap()).unwrap();
        let sodium=provider.latest(&item,Kind::Mods,"AANobbMI").unwrap();
        let latest=provider.latest(&item,Kind::Mods,&old.project).unwrap();
        let cancel=std::sync::atomic::AtomicBool::new(false);
        download::download_all(&download::client().unwrap(),&root.path().join("mods"),&[file_spec(&old,Kind::Mods).unwrap(),file_spec(&sodium,Kind::Mods).unwrap()],&cancel,&|_|{},"test").unwrap();
        let sodium_path=root.path().join("mods").join(&sodium.name);let before=content_hash(&sodium_path).unwrap();
        let update=Update{name:old.name.clone(),enabled:true,hash:content_hash(&root.path().join("mods").join(&old.name)).unwrap(),project:old.project.clone(),version:latest.version};
        install(root.path(),&item,Kind::Mods,&old.project,Some(update),&cancel,&|_|{}).unwrap();
        assert_eq!(content_hash(&sodium_path).unwrap(),before);
        assert!(root.path().join("mods").join(latest.name).exists());
        assert!(!root.path().join("mods").join(old.name).exists());
        assert_eq!(crate::content::scan(root.path(),Kind::Mods,false).unwrap().entries.len(),2);
    }
    #[test]
    fn large_packs_do_not_block_hash_inspection() {
        let root=tempfile::tempdir().unwrap();
        fs::create_dir(root.path().join("resourcepacks")).unwrap();
        let small=root.path().join("resourcepacks/small.zip");
        fs::write(&small,b"abc").unwrap();
        assert_eq!(content_hash(&small).unwrap(),"a9993e364706816aba3e25717850c26c9cd0d89d");
        let large=root.path().join("resourcepacks/large.zip");
        fs::File::create(&large).unwrap().set_len(257*1024*1024).unwrap();
        let files=local_files(root.path(),Kind::ResourcePacks).unwrap();
        assert_eq!(files.len(),2);
        assert!(files.iter().any(|(name,_,hash)|name=="large.zip" && hash.len()==40));
    }
    #[test]
    fn universal_dependencies_only_exclude_explicit_other_loader_projects() {
        let mut item=instance();item.minecraft_version="26.3".into();item.loader.as_mut().unwrap().kind="neoforge".into();
        let universal=vec!["fabric".into(),"neoforge".into()];
        assert!(other_loader_dependency(&json!({"loaders":["fabric"]}),&item,&universal));
        for data in [json!({"loaders":["neoforge"]}),json!({"loaders":["fabric","neoforge"]}),json!({"loaders":["forge"]}),json!({"loaders":[]}),json!({}),json!({"loaders":[null]})] {
            assert!(!other_loader_dependency(&data,&item,&universal));
        }
        assert!(!other_loader_dependency(&json!({"loaders":["fabric"]}),&item,&["neoforge".into()]));
        item.loader.as_mut().unwrap().kind="fabric".into();
        assert!(!other_loader_dependency(&json!({"loaders":["fabric"]}),&item,&universal));
    }
    #[test]
    #[ignore = "Uses live Modrinth metadata and downloads into temporary instances"]
    fn live_controlify_neoforge_install() {
        for game in ["26.3","1.21.11"] {
            let mut item=instance();item.minecraft_version=game.into();item.loader.as_mut().unwrap().kind="neoforge".into();
            let root=tempfile::tempdir().unwrap();
            install(root.path(),&item,Kind::Mods,"DOUdJVEm",None,&std::sync::atomic::AtomicBool::new(false),&|_|{}).unwrap();
            let files=crate::content::scan(root.path(),Kind::Mods,false).unwrap();
            assert!(files.entries.iter().any(|f|f.name.starts_with("controlify-")));
            assert!(files.entries.iter().any(|f|f.name.contains("yet_another_config_lib")));
            assert!(!files.entries.iter().any(|f|f.name.starts_with("fabric-api-")));
        }
    }
    #[test]
    fn exact_version_and_loader_are_required() {
        let item=instance();
        assert!(compatible(&json!({"game_versions":["1.21.1"],"loaders":["fabric"]}),&item,Kind::Mods));
        assert!(!compatible(&json!({"game_versions":["1.21"],"loaders":["fabric"]}),&item,Kind::Mods));
        assert!(!compatible(&json!({"game_versions":["1.21.1"],"loaders":["quilt"]}),&item,Kind::Mods));
        assert!(compatible(&json!({"game_versions":["1.21.1"],"loaders":["iris"]}),&item,Kind::Shaders));
    }
    #[test]
    fn provider_paths_and_downloads_are_scoped() {
        for invalid in ["../escape","one/two","?q=anything",""]{assert!(valid_id(invalid).is_err());}
        let mut release=Release{project:"test".into(),version:"v1".into(),name:"mod.jar".into(),url:"https://cdn.modrinth.com/data/test/mod.jar".into(),hash:"a".repeat(40),size:20,published:"2025".into(),dependencies:vec![],loaders:vec![]};
        assert!(file_spec(&release,Kind::Mods).is_ok());
        release.name="../mod.jar".into();assert!(file_spec(&release,Kind::Mods).is_err());
        release.name="mod.jar".into();release.url="https://example.com/mod.jar".into();assert!(file_spec(&release,Kind::Mods).is_err());
        assert!(content_path(Path::new("/tmp"),Kind::Mods,"../mod.jar",false).is_err());
    }
    #[test]
    fn pack_updates_scan_archives_without_loader_or_disabled_directories() {
        let root=tempfile::tempdir().unwrap();
        for kind in [Kind::ResourcePacks,Kind::Shaders] {
            let directory=root.path().join(kind.directory());
            fs::create_dir_all(directory.join("unpacked/shaders")).unwrap();
            fs::write(directory.join("unpacked/pack.mcmeta"),"{}").unwrap();
            fs::write(directory.join("custom.zip"),b"archive").unwrap();
            let local=local_files(root.path(),kind).unwrap();
            assert_eq!(local.len(),1);
            assert_eq!(local[0].0,"custom.zip");
            assert!(local[0].1);
            assert_eq!(local[0].2,format!("{:x}",Sha1::digest(b"archive")));
            assert!(content_path(root.path(),kind,"custom.zip",false).is_err());
            assert!(content_path(root.path(),kind,"../custom.zip",true).is_err());
            let mut item=instance();item.loader=None;
            assert!(compatible(&json!({"game_versions":["1.21.1"],"loaders":["minecraft"]}),&item,kind));
            assert!(!compatible(&json!({"game_versions":["1.20.1"]}),&item,kind));
        }
    }
    #[test]
    fn pinned_dependencies_and_primary_file_are_preserved() {
        let release=mr_release(&json!({"project_id":"project","id":"version","date_published":"2025",
            "files":[{"filename":"server.jar","primary":false},{"filename":"client.jar","primary":true,"url":"https://cdn.modrinth.com/client.jar","hashes":{"sha1":"a".repeat(40)},"size":20}],
            "dependencies":[{"dependency_type":"required","project_id":"dep","version_id":"pinned"},{"dependency_type":"optional","project_id":"optional"}]})).unwrap();
        assert_eq!(release.name,"client.jar");assert_eq!(release.dependencies,vec!["version:pinned"]);
    }
    #[test]
    #[ignore = "Uses the live Modrinth API and downloads into a temporary instance"]
    fn live_catalog_and_verified_install() {
        let item=instance();let p=Provider::new().unwrap();
        let result=p.search(&item,Kind::Mods,"Fabric API",0).unwrap();
        assert!(!result.projects.is_empty());
        for kind in [Kind::ResourcePacks,Kind::Shaders] {
            let packs=p.search(&item,kind,"",0).unwrap();
            let release=p.latest(&item,kind,&packs.projects[0].id).unwrap();
            assert!(file_spec(&release,kind).unwrap().path.ends_with(".zip"));
        }

        let root=tempfile::tempdir().unwrap();
        install(root.path(),&item,Kind::Mods,"P7dR8mSH",None,&std::sync::atomic::AtomicBool::new(false),&|_|{}).unwrap();
        let listing=crate::content::scan(root.path(),Kind::Mods,false).unwrap();
        assert_eq!(listing.entries.len(),1);
        assert!(updates(root.path(),&item,Kind::Mods).unwrap().updates.is_empty());
        // Recreate a renamed, disabled older file and exercise the actual update.
        let versions=p.api("/project/P7dR8mSH/version",&[("game_versions",json!([item.minecraft_version]).to_string()),("loaders",json!(["fabric"]).to_string())],None).unwrap();
        let latest=p.latest(&item,Kind::Mods,"P7dR8mSH").unwrap();
        let old=mr_release(array(&versions).unwrap().iter().find(|v|v["id"]!=latest.version && compatible(v,&item,Kind::Mods)).unwrap()).unwrap();
        fs::remove_file(root.path().join("mods").join(&listing.entries[0].name)).unwrap();
        let mut spec=file_spec(&old,Kind::Mods).unwrap();spec.path="custom-name.jar".into();
        download::download_all(&download::client().unwrap(),&root.path().join("mods"),&[spec],&std::sync::atomic::AtomicBool::new(false),&|_|{},"test").unwrap();
        crate::content::set_enabled(root.path(),Kind::Mods,"custom-name.jar",false).unwrap();
        let suggestions=updates(root.path(),&item,Kind::Mods).unwrap().updates;
        assert_eq!(suggestions.len(),1);assert_eq!(suggestions[0].name,"custom-name.jar");
        install(root.path(),&item,Kind::Mods,"P7dR8mSH",Some(suggestions[0].clone()),&std::sync::atomic::AtomicBool::new(false),&|_|{}).unwrap();
        let final_list=crate::content::scan(root.path(),Kind::Mods,false).unwrap();
        assert_eq!(final_list.entries.len(),1);assert!(!final_list.entries[0].enabled);
        assert_eq!(final_list.entries[0].name,latest.name);
        assert!(!root.path().join(".launcher-disabled/mods/custom-name.jar").exists());

    }
}
