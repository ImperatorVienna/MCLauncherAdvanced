//! Public account summaries and private credentials share one atomic, owner-only file.
//! Tokens never cross IPC. Only verified account identities may be selected.
use serde::{Deserialize, Serialize};
use std::{path::{Path, PathBuf}, sync::{Arc, Mutex}, io::Write};
type Result<T> = std::result::Result<T, String>;
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MicrosoftAccount { pub id: String, pub name: String }
#[derive(Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Accounts {
    pub revision: u64, pub selected: Option<String>,
    #[serde(default)] pub microsoft: Vec<MicrosoftAccount>,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Credential {
    pub id: String, pub name: String, pub refresh_token: String,
    pub access_token: String, pub expires_at: u64, pub xuid: String,
}
#[derive(Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Stored {
    revision: u64, selected: Option<String>,
    #[serde(default)] credentials: Vec<Credential>,
}
impl Stored {
    fn public(&self) -> Accounts { Accounts { revision: self.revision, selected: self.selected.clone(),
        microsoft: self.credentials.iter().map(|c| MicrosoftAccount {id:c.id.clone(),name:c.name.clone()}).collect() } }
}
#[derive(Clone)]
pub struct AccountStore(pub Arc<Mutex<PathBuf>>, pub Arc<Mutex<()>>);
pub fn valid_name(name: &str) -> bool { (3..=16).contains(&name.len()) && name.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'_') }
pub fn valid_id(id: &str) -> bool { id.len()==32 && id.bytes().all(|c| c.is_ascii_hexdigit()) }
fn validate(data: &Accounts) -> Result<()> {
    if data.microsoft.len() > 100 { return Err("Too many accounts".into()); }
    let mut ids = std::collections::HashSet::new();
    for account in &data.microsoft {
        if !valid_id(&account.id) || !valid_name(&account.name) || !ids.insert(&account.id) { return Err("Invalid Microsoft account data".into()); }
    }
    let exists = |s: &String| data.microsoft.iter().any(|a| format!("@{}",a.id)==*s);
    if data.selected.as_ref().is_some_and(|s| !exists(s)) || (!data.microsoft.is_empty() && data.selected.is_none()) { return Err("Select an existing account".into()); }
    Ok(())
}
fn read(path: &Path) -> Result<Stored> {
    if std::fs::symlink_metadata(path).is_ok_and(|m|m.file_type().is_symlink()) { return Err("Account storage cannot be a symbolic link".into()); }
    let mut data: Stored = match std::fs::read(path) {
        Ok(bytes) => serde_json::from_slice(&bytes).map_err(|_| "Account data is corrupt; the file was preserved")?,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Stored::default(),
        Err(_) => return Err("Cannot read account storage".into()),
    };
    // A legacy local selection cannot become an authenticated identity.
    if !data.credentials.iter().any(|c|data.selected.as_deref()==Some(&format!("@{}",c.id))) {
        data.selected=data.credentials.first().map(|c|format!("@{}",c.id));
    }
    validate(&data.public())?; Ok(data)
}
fn write(path: &Path, data: &Stored) -> Result<()> {
    let parent = path.parent().ok_or("Invalid account storage path")?;
    std::fs::create_dir_all(parent).map_err(|_|"Cannot create account storage")?;
    let mut temp = tempfile::NamedTempFile::new_in(parent).map_err(|_|"Cannot create private account storage")?;
    launcher_platform::protect_private_file(temp.as_file()).map_err(|_|"Cannot protect account storage")?;
    temp.write_all(&serde_json::to_vec(data).map_err(|_|"Cannot encode account storage")?).map_err(|_|"Cannot write account storage")?;
    temp.as_file().sync_all().map_err(|_|"Cannot flush account storage")?;
    temp.persist(path).map_err(|_|"Cannot save account storage")?;
    Ok(())
}
#[tauri::command]
pub fn load_accounts(store: tauri::State<'_, AccountStore>) -> Result<Accounts> { store.load() }
#[tauri::command]
pub fn save_accounts(store: tauri::State<'_, AccountStore>, accounts: Accounts) -> Result<Accounts> { store.save(accounts) }
impl AccountStore {
    pub fn load(&self) -> Result<Accounts> { Ok(read(&*self.0.lock().map_err(|_|"Account storage is unavailable")?)?.public()) }
    pub fn save(&self, accounts: Accounts) -> Result<Accounts> {
        validate(&accounts)?;
        let path = self.0.lock().map_err(|_|"Account storage is unavailable")?;
        let mut stored = read(&path)?;
        if stored.revision != accounts.revision { return Err("Account data changed; reload and retry".into()); }
        // Renderer may select or remove a verified account, never fabricate or rename one.
        let old = stored.public();
        if accounts.microsoft.iter().any(|a| !old.microsoft.contains(a)) { return Err("Microsoft accounts must be verified by signing in".into()); }
        stored.credentials.retain(|c| accounts.microsoft.iter().any(|a|a.id==c.id));
        stored.selected=accounts.selected; stored.revision+=1;
        write(&path,&stored)?; Ok(stored.public())
    }
    pub fn add(&self, credential: Credential) -> Result<()> {
        let path=self.0.lock().map_err(|_|"Account storage is unavailable")?;
        let mut stored=read(&path)?;
        stored.credentials.retain(|c|c.id!=credential.id);
        stored.selected=Some(format!("@{}",credential.id)); stored.credentials.push(credential); stored.revision+=1;
        validate(&stored.public())?; write(&path,&stored)
    }
    pub fn credential(&self, id: &str) -> Result<Credential> {
        let path=self.0.lock().map_err(|_|"Account storage is unavailable")?;
        read(&path)?.credentials.into_iter().find(|c|c.id==id).ok_or("Account was removed; sign in again".into())
    }
    // Persist a rotated refresh token before contacting Xbox/Minecraft. Never resurrect a removed account.
    pub fn replace(&self, previous: &Credential, next: Credential) -> Result<()> {
        let path=self.0.lock().map_err(|_|"Account storage is unavailable")?;
        let mut stored=read(&path)?;
        let c=stored.credentials.iter_mut().find(|c|c.id==previous.id && c.refresh_token==previous.refresh_token).ok_or("Account changed; retry the operation")?;
        let renamed=c.name!=next.name; *c=next; if renamed {stored.revision+=1;} validate(&stored.public())?; write(&path,&stored)
    }
}
pub fn initialize(_app: &tauri::AppHandle) -> std::result::Result<AccountStore, Box<dyn std::error::Error>> {
    Ok(AccountStore(Arc::new(Mutex::new(crate::storage_commands::launcher_directory()?.join("accounts.json"))),Arc::new(Mutex::new(()))))
}
#[cfg(test)] mod tests {
    use super::*;
    fn store(path: PathBuf) -> AccountStore { AccountStore(Arc::new(Mutex::new(path)),Arc::new(Mutex::new(()))) }
    #[cfg(unix)]
    #[test] fn credentials_are_private_and_cannot_be_fabricated_or_resurrected() {
        use std::os::unix::fs::PermissionsExt;
        let dir=tempfile::tempdir().unwrap(); let path=dir.path().join("accounts.json"); let store=store(path.clone());
        std::fs::write(&path,r#"{"revision":0,"names":["Steve"],"selected":"Steve"}"#).unwrap();
        let c=Credential {id:"a".repeat(32),name:"Alex".into(),refresh_token:"secret-refresh".into(),access_token:"secret-access".into(),expires_at:0,xuid:String::new()};
        store.add(c.clone()).unwrap();
        let mut public=store.load().unwrap();
        assert!(!serde_json::to_string(&public).unwrap().contains("secret"));
        assert_eq!(std::fs::metadata(path).unwrap().permissions().mode() & 0o777,0o600);
        public.microsoft[0].name="Forged".into(); assert!(store.save(public).is_err());
        let mut public=store.load().unwrap(); public.microsoft.clear(); public.selected=None;
        store.save(public).unwrap();
        assert!(store.replace(&c,c.clone()).is_err()); assert!(store.credential(&c.id).is_err());
    }
    #[test] fn validate_identity_and_selection() {
        let mut a=Accounts {revision:0,selected:Some("Steve".into()),microsoft:vec![],..Default::default()};
        assert!(validate(&a).is_err());
        a.selected=None;assert!(validate(&a).is_ok());
        assert!(serde_json::from_str::<Accounts>(r#"{"revision":0,"names":["Steve"],"selected":"Steve"}"#).is_err());
    }
    #[test] fn legacy_local_identity_cannot_launch() {
        let dir=tempfile::tempdir().unwrap();let path=dir.path().join("accounts.json");
        std::fs::write(&path,r#"{"revision":2,"names":["Steve"],"selected":"Steve"}"#).unwrap();
        let storage=store(path);
        assert_eq!(storage.load().unwrap().selected,None);
        assert!(crate::microsoft_auth::identity(&storage,"Steve").is_err());
        assert!(crate::microsoft_auth::identity(&storage,"@fake").is_err());
    }
}
