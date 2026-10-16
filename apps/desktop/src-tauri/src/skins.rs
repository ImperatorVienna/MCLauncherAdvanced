//! Local, bounded PNG skin library. No remote requests or game skin injection.
use base64::{Engine, engine::general_purpose::STANDARD};
use serde::{Deserialize, Serialize};
use std::{collections::{BTreeMap, HashSet}, path::PathBuf, sync::Mutex};
use std::io::Read;
const LIMIT: usize = 128 * 1024;
const BUILTINS: [&str; 9] = ["steve","alex","zuri","sunny","noor","makena","kai","efe","ari"];
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all="camelCase", deny_unknown_fields)]
pub struct Skin { id: String, name: String, model: String, png: String }
#[derive(Default, Clone, Serialize, Deserialize)]
#[serde(rename_all="camelCase", deny_unknown_fields)]
pub struct Library { revision: u64, skins: Vec<Skin>, assignments: BTreeMap<String, String>, #[serde(default)] builtin_models: BTreeMap<String, String> }
pub struct SkinStore(Mutex<PathBuf>);
fn png(bytes: &[u8]) -> Result<(), String> {
    if bytes.len() > LIMIT { return Err("Skin PNG exceeds 128 KiB".into()); }
    let mut decoder = png::Decoder::new(std::io::Cursor::new(bytes));
    decoder.set_limits(png::Limits { bytes: 1024 * 1024 });
    let mut reader = decoder.read_info().map_err(|_| "Invalid skin PNG")?;
    let info = reader.info();
    if info.width != 64 || ![32,64].contains(&info.height) || info.animation_control.is_some() { return Err("Skin must be a static 64×64 or 64×32 PNG".into()); }
    let mut buffer = vec![0; reader.output_buffer_size()];
    reader.next_frame(&mut buffer).map_err(|_| "Invalid skin PNG")?;
    Ok(())
}
impl Library {
    fn validate(&self) -> Result<(), String> {
        if self.skins.len() > 100 || self.assignments.len() > 1000 { return Err("Skin library limit reached".into()); }
        let mut ids: HashSet<String> = BUILTINS.iter().map(|n| format!("builtin-{n}")).collect();
        if self.builtin_models.iter().any(|(id, model)| !ids.contains(id) || !["default", "slim"].contains(&model.as_str())) { return Err("Invalid skin model".into()); }
        for skin in &self.skins {
            if skin.id.len() != 36 || !skin.id.bytes().all(|c| c.is_ascii_hexdigit() || c == b'-') || !ids.insert(skin.id.clone()) { return Err("Invalid or duplicate skin ID".into()); }
            if skin.name.trim() != skin.name || skin.name.is_empty() || skin.name.chars().count() > 64 || skin.name.chars().any(char::is_control) { return Err("Skin name must contain 1–64 characters".into()); }
            if !["default","slim"].contains(&skin.model.as_str()) { return Err("Invalid skin model".into()); }
            if skin.png.len() > LIMIT * 4 / 3 + 4 { return Err("Skin PNG exceeds 128 KiB".into()); }
            png(&STANDARD.decode(&skin.png).map_err(|_| "Invalid skin PNG")?)?;
        }
        for (account, id) in &self.assignments {
            if !(3..=16).contains(&account.len()) || !account.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'_') || !ids.contains(id) { return Err("Invalid skin assignment".into()); }
        }
        Ok(())
    }
}
fn read(path: &std::path::Path) -> Result<Library, String> {
    let file = match std::fs::File::open(path) { Ok(file) => file, Err(e) if e.kind()==std::io::ErrorKind::NotFound => return Ok(Library::default()), Err(e) => return Err(e.to_string()) };
    let mut bytes = Vec::new();
    file.take(18 * 1024 * 1024 + 1).read_to_end(&mut bytes).map_err(|e| e.to_string())?;
    if bytes.len() > 18 * 1024 * 1024 { return Err("Skin library limit reached".into()); }
    let data: Library = serde_json::from_slice(&bytes).map_err(|_| "Skin library is corrupt; the original file was preserved")?;
    data.validate()?; Ok(data)
}
#[tauri::command]
pub fn load_skins(store: tauri::State<'_, SkinStore>) -> Result<Library, String> { read(&*store.0.lock().map_err(|_| "Skin storage is unavailable")?) }
#[tauri::command]
pub fn save_skins(store: tauri::State<'_, SkinStore>, library: Library) -> Result<Library, String> {
    let path = store.0.lock().map_err(|_| "Skin storage is unavailable")?;
    persist(&path, library)
}
fn persist(path: &std::path::Path, mut library: Library) -> Result<Library, String> {
    library.validate()?;
    if read(path)?.revision != library.revision { return Err("Skin library changed; reload and retry".into()); }
    library.revision = library.revision.checked_add(1).ok_or("Skin library limit reached")?;
    launcher_core::storage::atomic_write(path, &serde_json::to_vec(&library).map_err(|e| e.to_string())?)?;
    Ok(library)
}
pub fn initialize(_app: &tauri::AppHandle) -> Result<SkinStore, Box<dyn std::error::Error>> { Ok(SkinStore(Mutex::new(crate::storage_commands::launcher_directory()?.join("launcher-skins.json")))) }
#[cfg(test)] mod tests {
    use super::*;
    #[test] fn rejects_invalid_png_and_dangling_assignment() {
        assert!(png(b"not a png").is_err());
        assert!(png(include_bytes!("../../src/assets/skins/steve.png")).is_ok());
        let mut data = Library::default(); data.assignments.insert("Steve".into(), "missing".into());
        assert!(data.validate().is_err());
        data.assignments.insert("Steve".into(), "builtin-alex".into()); assert!(data.validate().is_ok());
    }
    #[test] fn atomic_roundtrip_conflict_and_corruption() {
        let dir = std::env::temp_dir().join(format!("skin-store-{}",std::process::id())); std::fs::create_dir_all(&dir).unwrap(); let path = dir.join("skins.json");
        let data = Library { skins: vec![Skin { id:"12345678-1234-1234-1234-123456789abc".into(), name:"Test".into(), model:"default".into(), png: STANDARD.encode(include_bytes!("../../src/assets/skins/steve.png")) }], ..Default::default() };
        let saved = persist(&path,data.clone()).unwrap(); assert_eq!(saved.revision,1); assert_eq!(read(&path).unwrap().skins.len(),1); assert!(persist(&path,data).is_err());
        std::fs::write(&path,b"broken").unwrap(); assert!(persist(&path,Library::default()).is_err()); assert_eq!(std::fs::read(&path).unwrap(),b"broken"); std::fs::remove_dir_all(dir).unwrap();
    }
}
