//! Xbox Sisu authorization (the official launcher's flow) lives in the native process.
//! The host shows Microsoft's page and hands back only the redirect address. No
//! password, device key, refresh token, or game access token reaches the renderer.
use crate::accounts::{AccountStore, Credential, DeviceRecord};
use base64::{engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD}, Engine};
use p256::ecdsa::{signature::Signer, Signature, SigningKey};
use reqwest::{blocking::{Client, RequestBuilder}, header::HeaderMap};
use serde::Serialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{io::Read, sync::{Arc, Mutex}, time::{Duration, Instant, SystemTime, UNIX_EPOCH}};
type Result<T> = std::result::Result<T, String>;
// Single configuration point. Login, refresh and game arguments all use this value.
// Legacy MSA ID of the official Minecraft Launcher, used through Xbox Sisu.
pub const CLIENT_ID: &str = "00000000402b5328";
fn require_client_id() -> Result<()> {
    if CLIENT_ID.trim().is_empty() {
        return Err("Microsoft Client ID is not configured; configure it and rebuild the launcher".into());
    }
    Ok(())
}
/// Redirect registered for the official launcher; the host intercepts it before navigation.
pub const REDIRECT_URI: &str = "ms-xal-00000000402b5328://auth";
const REDIRECT_SCHEME: &str = "ms-xal-00000000402b5328";
const TITLE_ID: &str = "1794566092";
const TOKEN_URL: &str = "https://login.live.com/oauth20_token.srf";
const SCOPE: &str = "service::user.auth.xboxlive.com::MBI_SSL";
const DEVICE_URL: &str = "https://device.auth.xboxlive.com/device/authenticate";
const SISU_AUTHENTICATE: &str = "https://sisu.xboxlive.com/authenticate";
const SISU_AUTHORIZE: &str = "https://sisu.xboxlive.com/authorize";
const XSTS_URL: &str = "https://xsts.auth.xboxlive.com/xsts/authorize";
const LOGIN_SECONDS: u64 = 900;
const NETWORK: &str = "Cannot reach sign-in services; check your connection and retry";
pub fn now() -> u64 { SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() }
fn client() -> Result<Client> {
    Client::builder().timeout(Duration::from_secs(25)).connect_timeout(Duration::from_secs(10))
        .redirect(reqwest::redirect::Policy::none()).user_agent("mclauncher-advanced/0.1.0")
        .build().map_err(|_|NETWORK.into())
}
fn response_headers(request: RequestBuilder) -> Result<(u16,HeaderMap,Value)> {
    let res=request.send().map_err(|_|NETWORK)?; let status=res.status().as_u16(); let headers=res.headers().clone();
    let mut bytes=Vec::new(); res.take(1024*1024+1).read_to_end(&mut bytes).map_err(|_|NETWORK)?;
    if bytes.len()>1024*1024 {return Err("Invalid response from sign-in service".into())}
    let data=match serde_json::from_slice(&bytes) {
        Ok(data)=>data,
        // Error responses (notably Xbox) can have empty bodies; map them by status.
        Err(_) if !(200..300).contains(&status) => Value::Null,
        Err(_)=>return Err("Invalid response from sign-in service".into()),
    };
    Ok((status,headers,data))
}
fn response(request: RequestBuilder) -> Result<(u16,Value)> { let (status,_,data)=response_headers(request)?; Ok((status,data)) }
fn text<'a>(value:&'a Value,key:&str)->Result<&'a str> {
    value[key].as_str().filter(|s|!s.is_empty()).ok_or("Incomplete response from sign-in service".into())
}
fn failure(stage:&str,status:u16,data:&Value)->String {
    // Only known error codes are surfaced; response bodies can contain credentials.
    match data["error"].as_str().unwrap_or("") {
        "invalid_grant" | "interaction_required" | "login_required" => return "Sign-in expired or was revoked; sign in with Microsoft again".into(),
        "authorization_declined" | "access_denied" => return "Microsoft sign-in was declined".into(),
        "expired_token" => return "The sign-in code expired; start sign-in again".into(),
        "invalid_client" | "unauthorized_client" => return "Microsoft application configuration was rejected; check public client settings".into(),
        _=>{}
    }
    match data["XErr"].as_u64().or(data["AuthorizationToken"]["XErr"].as_u64()) {
        Some(2148916233)=>return "Create an Xbox profile at xbox.com, then sign in again".into(),
        Some(2148916235)=>return "Xbox sign-in is not available for this account region".into(),
        Some(2148916236 | 2148916237)=>return "Complete Xbox age verification, then sign in again".into(),
        Some(2148916238)=>return "This Xbox child account needs family authorization".into(),
        _=>{}
    }
    if stage=="Minecraft" && status==403 {return "Minecraft rejected this application; its Client ID may require Minecraft API approval".into()}
    if stage=="Minecraft profile" && status==404 {return "No Minecraft Java profile found; check game ownership and create a Java profile".into()}
    if status==429 {return "Too many sign-in requests; wait and retry".into()}
    format!("{stage} sign-in failed (HTTP {status})")
}
fn checked(request:RequestBuilder,stage:&str)->Result<Value> {
    let (status,data)=response(request)?;
    if !(200..300).contains(&status) {return Err(failure(stage,status,&data))} Ok(data)
}

// ---- Xbox proof-of-possession device identity ----
fn random<const N:usize>()->Result<[u8;N]> {
    let mut bytes=[0;N]; getrandom::getrandom(&mut bytes).map_err(|_|"Cannot generate a secure random value")?; Ok(bytes)
}
fn hex(bytes:&[u8])->String { bytes.iter().map(|b|format!("{b:02x}")).collect() }
struct Device { id:String, key:SigningKey, token:String }
impl Device {
    fn proof_key(&self)->Value {
        let point=self.key.verifying_key().to_encoded_point(false);
        let (x,y)=(point.x().map(|v|URL_SAFE_NO_PAD.encode(v)),point.y().map(|v|URL_SAFE_NO_PAD.encode(v)));
        json!({"kty":"EC","x":x,"y":y,"crv":"P-256","alg":"ES256","use":"sig"})
    }
}
fn new_key()->Result<(String,SigningKey)> {
    let mut id=random::<16>()?; id[6]=(id[6]&0x0f)|0x40; id[8]=(id[8]&0x3f)|0x80;
    let h=hex(&id).to_uppercase();
    let id=format!("{{{}-{}-{}-{}-{}}}",&h[0..8],&h[8..12],&h[12..16],&h[16..20],&h[20..]);
    for _ in 0..8 { if let Ok(key)=SigningKey::from_slice(&random::<32>()?) {return Ok((id,key))} }
    Err("Cannot generate a secure random value".into())
}
/// Seconds since the epoch from Xbox's `YYYY-MM-DDTHH:MM:SS(.fraction)Z` timestamps.
fn parse_time(value:&str)->Option<u64> {
    let b=value.as_bytes(); if b.len()<19 || b[4]!=b'-' || b[7]!=b'-' || b[10]!=b'T' || b[13]!=b':' || b[16]!=b':' {return None}
    let n=|r:std::ops::Range<usize>|value.get(r)?.parse::<i64>().ok();
    let (y,m,d,hh,mm,ss)=(n(0..4)?,n(5..7)?,n(8..10)?,n(11..13)?,n(14..16)?,n(17..19)?);
    if !(1..=12).contains(&m) || !(1..=31).contains(&d) || hh>23 || mm>59 || ss>60 {return None}
    let (y,m)=if m<=2 {(y-1,m+9)} else {(y,m-3)};
    let era=y.div_euclid(400); let yoe=y-era*400; let doy=(153*m+2)/5+d-1; let doe=yoe*365+yoe/4-yoe/100+doy;
    u64::try_from((era*146097+doe-719468)*86400+hh*3600+mm*60+ss).ok()
}
/// Xbox request signature: ES256 over a policy-versioned buffer bound to time, path and body.
fn signature(key:&SigningKey,path:&str,body:&[u8],unix:u64)->String {
    let time=(unix+11644473600)*10_000_000;
    let mut buffer=Vec::with_capacity(body.len()+64);
    buffer.extend_from_slice(&1_u32.to_be_bytes()); buffer.push(0);
    buffer.extend_from_slice(&time.to_be_bytes()); buffer.push(0);
    for part in [b"POST".as_slice(),path.as_bytes(),b"",body] { buffer.extend_from_slice(part); buffer.push(0); }
    let signed:Signature=key.sign(&buffer);
    let mut out=Vec::with_capacity(76);
    out.extend_from_slice(&1_u32.to_be_bytes()); out.extend_from_slice(&time.to_be_bytes()); out.extend_from_slice(&signed.to_bytes());
    STANDARD.encode(out)
}
fn signed(client:&Client,url:&str,contract:bool,body:&Value,key:&SigningKey,stage:&str)->Result<(HeaderMap,Value)> {
    let bytes=serde_json::to_vec(body).map_err(|_|"Cannot encode sign-in request")?;
    let path=reqwest::Url::parse(url).map_err(|_|"Invalid sign-in service address")?.path().to_owned();
    let mut request=client.post(url).header("Content-Type","application/json; charset=utf-8").header("Accept","application/json")
        .header("Signature",signature(key,&path,&bytes,now()));
    if contract {request=request.header("x-xbl-contract-version","1");}
    let (status,headers,data)=response_headers(request.body(bytes))?;
    if !(200..300).contains(&status) {return Err(failure(stage,status,&data))}
    Ok((headers,data))
}
fn device(client:&Client,store:&AccountStore)->Result<Device> {
    let stored=store.device()?;
    let existing=stored.as_ref().and_then(|r|Some((r.id.clone(),SigningKey::from_slice(&STANDARD.decode(&r.key).ok()?).ok()?)));
    if let (Some(record),Some((id,key)))=(&stored,&existing) {
        if record.not_after>now()+300 && !record.token.is_empty() {return Ok(Device{id:id.clone(),key:key.clone(),token:record.token.clone()})}
    }
    let (id,key)=match existing {Some(pair)=>pair,None=>new_key()?};
    let mut device=Device{id,key,token:String::new()};
    let (_,data)=signed(client,DEVICE_URL,true,&json!({
        "Properties":{"AuthMethod":"ProofOfPossession","Id":device.id,"DeviceType":"Win32","Version":"10.16.0","ProofKey":device.proof_key()},
        "RelyingParty":"http://auth.xboxlive.com","TokenType":"JWT"
    }),&device.key,"Xbox device")?;
    device.token=text(&data,"Token")?.to_owned();
    let not_after=data["NotAfter"].as_str().and_then(parse_time).ok_or("Incomplete response from sign-in service")?;
    store.save_device(DeviceRecord{id:device.id.clone(),key:STANDARD.encode(device.key.to_bytes()),token:device.token.clone(),not_after})?;
    Ok(device)
}
// Structural validation only. An empty store list does not establish whether a
// subscription is active; profile access must still succeed before persistence.
fn validate_entitlements(value:&Value)->Result<()> {
    let items=value["items"].as_array().ok_or("Incomplete response from sign-in service")?;
    for item in items { text(item,"name")?; }
    Ok(())
}
fn exchange(client:&Client,device:&Device,access:&str,refresh:String,session:Option<&str>)->Result<Credential> {
    exchange_at(client,device,access,refresh,session,[SISU_AUTHORIZE,XSTS_URL,"https://api.minecraftservices.com/authentication/login_with_xbox","https://api.minecraftservices.com/entitlements/mcstore","https://api.minecraftservices.com/minecraft/profile"])
}
fn exchange_at(client:&Client,device:&Device,access:&str,refresh:String,session:Option<&str>,urls:[&str;5])->Result<Credential> {
    // Sisu authorize does not accept the contract-version header.
    let (_,sisu)=signed(client,urls[0],false,&json!({
        "AccessToken":format!("t={access}"),"AppId":CLIENT_ID,"DeviceToken":device.token,"ProofKey":device.proof_key(),
        "Sandbox":"RETAIL","SessionId":session,"SiteName":"user.auth.xboxlive.com","RelyingParty":"http://xboxlive.com","UseModernGamertag":true
    }),&device.key,"Xbox")?;
    if sisu["AuthorizationToken"]["XErr"].is_u64() {return Err(failure("Xbox",403,&sisu))}
    let (_,xsts)=signed(client,urls[1],true,&json!({
        "Properties":{"SandboxId":"RETAIL","UserTokens":[text(&sisu["UserToken"],"Token")?],"DeviceToken":device.token,"TitleToken":text(&sisu["TitleToken"],"Token")?},
        "RelyingParty":"rp://api.minecraftservices.com/","TokenType":"JWT"
    }),&device.key,"Xbox XSTS")?;
    let uhs=text(&xsts["DisplayClaims"]["xui"][0],"uhs")?;
    let minecraft=checked(client.post(urls[2]).json(&json!({
        "identityToken":format!("XBL3.0 x={uhs};{}",text(&xsts,"Token")?)
    })),"Minecraft")?;
    let token=text(&minecraft,"access_token")?.to_owned();
    // Minecraft requires the store check before profile access. Do not hard-code
    // product names: subscription entitlements can differ from purchased copies.
    let entitlements=checked(client.get(urls[3]).bearer_auth(&token),"Minecraft entitlements")?;
    validate_entitlements(&entitlements)?;
    let profile=checked(client.get(urls[4]).bearer_auth(&token),"Minecraft profile")?;
    let id=text(&profile,"id")?.to_lowercase(); let name=text(&profile,"name")?.to_owned();
    if !crate::accounts::valid_id(&id) || !crate::accounts::valid_name(&name) {return Err("Invalid Minecraft profile".into())}
    let expires=minecraft["expires_in"].as_u64().filter(|v|*v>0 && *v<=172800).ok_or("Invalid token expiry")?;
    Ok(Credential {id,name,refresh_token:refresh,access_token:token,expires_at:now()+expires,xuid:xsts["DisplayClaims"]["xui"][0]["xid"].as_str().unwrap_or("").into()})
}
#[derive(Clone, Default, Serialize)]
#[serde(rename_all="camelCase")]
pub struct LoginStatus { pub running:bool, pub phase:String, pub user_code:Option<String>, pub verification_uri:Option<String>, pub expires_at:Option<u64>, pub error:Option<String>, pub completed:bool }
/// Native-only authorization state for the current attempt.
#[derive(Default)]
struct Pending { address:Option<String>, state:Option<String>, code:Option<Result<String>> }
#[derive(Default)]
struct Session { generation:u64, status:LoginStatus, pending:Pending }
#[derive(Clone,Default)]
pub struct MicrosoftLogin(Arc<Mutex<Session>>);
impl MicrosoftLogin {
    fn with<T>(&self,id:u64,callback:impl FnOnce(&mut Session)->T)->Result<T> {
        let mut s=self.0.lock().map_err(|_|"Sign-in state is unavailable")?;
        if s.generation!=id || !s.status.running {return Err("Sign-in cancelled".into())}
        Ok(callback(&mut s))
    }
    fn update(&self,id:u64,callback:impl FnOnce(&mut LoginStatus))->Result<()> { self.with(id,|s|callback(&mut s.status)) }
    fn wait_code(&self,id:u64,deadline:Instant)->Result<String> {
        loop {
            if let Some(code)=self.with(id,|s|s.pending.code.take())? {return code}
            if Instant::now()>=deadline {return Err("The sign-in code expired; start sign-in again".into())}
            std::thread::sleep(Duration::from_millis(200));
        }
    }
    fn finish(&self,id:u64,store:&AccountStore,credential:Credential)->Result<()> {
        let mut s=self.0.lock().map_err(|_|"Sign-in state is unavailable")?;
        if s.generation!=id || !s.status.running {return Err("Sign-in cancelled".into())}
        store.add(credential)?;
        s.status=LoginStatus {completed:true,..Default::default()}; s.pending=Pending::default(); Ok(())
    }
}
fn login(session:&MicrosoftLogin,id:u64,store:&AccountStore,ready:impl FnOnce(u64,&str))->Result<()> {
    let client=client()?;
    let device=device(&client,store)?;
    let verifier=hex(&random::<32>()?);
    let challenge=URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()));
    let state=hex(&random::<16>()?);
    let (headers,data)=signed(&client,SISU_AUTHENTICATE,true,&json!({
        "AppId":CLIENT_ID,"DeviceToken":device.token,"Offers":[SCOPE],
        "Query":{"code_challenge":challenge,"code_challenge_method":"S256","state":state,"prompt":"select_account"},
        "RedirectUri":REDIRECT_URI,"Sandbox":"RETAIL","TokenType":"code","TitleId":TITLE_ID
    }),&device.key,"Xbox")?;
    let sisu_session=headers.get("X-SessionId").and_then(|v|v.to_str().ok()).filter(|v|!v.is_empty()).ok_or("Incomplete response from sign-in service")?.to_owned();
    let address=text(&data,"MsaOauthRedirect")?;
    if !address.starts_with("https://login.live.com/") {return Err("Unexpected Microsoft verification address".into())}
    let deadline=Instant::now()+Duration::from_secs(LOGIN_SECONDS);
    session.with(id,|s|{
        s.status.phase="waiting".into(); s.status.expires_at=Some(now()+LOGIN_SECONDS);
        s.pending=Pending{address:Some(address.into()),state:Some(state),code:None};
    })?;
    ready(id,address);
    let code=session.wait_code(id,deadline)?;
    session.with(id,|s|{s.status.phase="verifying".into();s.pending.address=None;})?;
    let token=checked(client.post(TOKEN_URL).header("Accept","application/json").form(&[("client_id",CLIENT_ID),("code",&code),("code_verifier",&verifier),
        ("grant_type","authorization_code"),("redirect_uri",REDIRECT_URI),("scope",SCOPE)]),"Microsoft")?;
    let credential=exchange(&client,&device,text(&token,"access_token")?,text(&token,"refresh_token")?.into(),Some(&sisu_session))?;
    session.finish(id,store,credential)
}
/// Starts sign-in. `ready` receives the attempt and the Microsoft page to show;
/// the host must report the redirect with `deliver_redirect` or abandon the attempt.
pub fn start_microsoft_login(session:&MicrosoftLogin,store:&AccountStore,ready:impl FnOnce(u64,&str)+Send+'static)->Result<()> {
    require_client_id()?;
    let id={let mut s=session.0.lock().map_err(|_|"Sign-in state is unavailable")?;
        if s.status.running {return Err("Sign-in is already running".into())}
        s.generation+=1;s.status=LoginStatus {running:true,phase:"starting".into(),..Default::default()};s.pending=Pending::default();s.generation};
    let session=session.clone();let store=store.clone();let failure_session=session.clone();
    if std::thread::Builder::new().name("microsoft-login".into()).spawn(move || {
        let outcome=std::panic::catch_unwind(std::panic::AssertUnwindSafe(||login(&session,id,&store,ready))).unwrap_or_else(|_|Err("Sign-in stopped unexpectedly; retry".into()));
        if let Err(error)=outcome {let _=session.with(id,|s|{s.status=LoginStatus {error:Some(error),..Default::default()};s.pending=Pending::default();});}
    }).is_err() {let _=failure_session.update(id,|s|{s.running=false;});return Err("Cannot start sign-in".into())}
    Ok(())
}
pub fn microsoft_login_status(session:&MicrosoftLogin)->Result<LoginStatus> {Ok(session.0.lock().map_err(|_|"Sign-in state is unavailable")?.status.clone())}
pub fn cancel_microsoft_login(session:&MicrosoftLogin)->Result<()> {
    let mut s=session.0.lock().map_err(|_|"Sign-in state is unavailable")?;s.generation+=1;s.status=LoginStatus::default();s.pending=Pending::default();Ok(())
}
/// The Microsoft page for the attempt that is waiting for the user, if any.
pub fn authorization_address(session:&MicrosoftLogin)->Result<(u64,String)> {
    let s=session.0.lock().map_err(|_|"Sign-in state is unavailable")?;
    let uri=s.pending.address.as_ref().filter(|_|s.status.running && s.status.phase=="waiting").ok_or("Start sign-in first")?;
    Ok((s.generation,uri.clone()))
}
/// True for the official launcher redirect, which must never be loaded by the host.
pub fn is_redirect(url:&str)->bool {
    reqwest::Url::parse(url).is_ok_and(|u|u.scheme()==REDIRECT_SCHEME)
}
/// Accepts the intercepted redirect. Unknown, stale or mismatched redirects are ignored.
pub fn deliver_redirect(session:&MicrosoftLogin,url:&str)->Result<()> {
    let url=reqwest::Url::parse(url).map_err(|_|"Invalid Microsoft redirect")?;
    if url.scheme()!=REDIRECT_SCHEME || url.host_str()!=Some("auth") {return Err("Invalid Microsoft redirect".into())}
    let param=|name:&str|url.query_pairs().find(|(k,_)|k==name).map(|(_,v)|v.into_owned());
    let mut s=session.0.lock().map_err(|_|"Sign-in state is unavailable")?;
    if !s.status.running || s.status.phase!="waiting" || s.pending.code.is_some() {return Err("Sign-in cancelled".into())}
    if s.pending.state.is_none() || param("state")!=s.pending.state {return Err("Invalid Microsoft redirect".into())}
    s.pending.code=Some(match (param("code").filter(|c|!c.is_empty()),param("error")) {
        (Some(code),None)=>Ok(code),
        (_,Some(error))=>Err(failure("Microsoft",400,&json!({"error":error}))),
        _=>Err("Incomplete response from sign-in service".into()),
    });
    Ok(())
}
/// The host closed the sign-in page; cancel only if that attempt is still waiting for it.
pub fn abandon_authorization(session:&MicrosoftLogin,id:u64) {
    let _=session.with(id,|s|if s.status.phase=="waiting" && s.pending.code.is_none() {
        s.pending.code=Some(Err("Microsoft sign-in window was closed".into()));
    });
}
pub struct Identity {pub name:String,pub uuid:String,pub access_token:String,pub xuid:String}
pub fn identity(store:&AccountStore,selected:&str)->Result<Identity> {
    require_client_id()?;
    let id=selected.strip_prefix('@').filter(|id|crate::accounts::valid_id(id)).ok_or("Sign in before launching the game")?;
    let _guard=store.1.lock().map_err(|_|"Account refresh is unavailable")?;
    let client=client()?;
    let c=refresh_credential(store,id,|refresh| checked(client.post(TOKEN_URL).header("Accept","application/json").form(&[("client_id",CLIENT_ID),("scope",SCOPE),("grant_type","refresh_token"),("refresh_token",refresh)]),"Microsoft"),
        |access,refresh|{let device=device(&client,store)?; exchange(&client,&device,access,refresh,None)})?;
    Ok(Identity{name:c.name,uuid:c.id,access_token:c.access_token,xuid:c.xuid})
}
pub fn skin_profile(store: &AccountStore, selected: &str) -> Result<Value> {
    let identity = identity(store, selected)?;
    let profile = checked(client()?.get("https://api.minecraftservices.com/minecraft/profile").bearer_auth(&identity.access_token), "Minecraft profile")?;
    if profile["id"].as_str().map(str::to_lowercase).as_deref() != Some(identity.uuid.as_str()) {
        return Err("Invalid Minecraft profile".into());
    }
    Ok(profile)
}
pub fn upload_skin(identity: &Identity, model: &str, bytes: Vec<u8>) -> Result<()> {
    upload_skin_at(&client()?, "https://api.minecraftservices.com/minecraft/profile/skins", identity, model, bytes)
}
fn upload_skin_at(client: &Client, endpoint: &str, identity: &Identity, model: &str, bytes: Vec<u8>) -> Result<()> {
    let variant = match model { "default" => "classic", "slim" => "slim", _ => return Err("Invalid skin model".into()) };
    let file = reqwest::blocking::multipart::Part::bytes(bytes).file_name("skin.png").mime_str("image/png").map_err(|_| "Invalid skin PNG")?;
    let form = reqwest::blocking::multipart::Form::new().text("variant", variant).part("file", file);
    let (status, profile) = response(client.post(endpoint).bearer_auth(&identity.access_token).multipart(form))?;
    match status {
        200 => {},
        401 | 403 => return Err("Sign-in expired or was revoked; sign in with Microsoft again".into()),
        429 => return Err("Too many skin updates; try again later".into()),
        _ => return Err("Skin upload failed; retry".into()),
    }
    let expected_variant = if model == "slim" { "SLIM" } else { "CLASSIC" };
    if profile["id"].as_str().map(str::to_lowercase).as_deref() != Some(identity.uuid.as_str()) ||
        !profile["skins"].as_array().is_some_and(|skins| skins.iter().any(|skin| skin["state"] == "ACTIVE" && skin["variant"] == expected_variant)) {
        return Err("Skin update could not be confirmed; reload the account skin".into());
    }
    Ok(())
}
fn refresh_credential(store:&AccountStore,id:&str,request:impl FnOnce(&str)->Result<Value>,exchange:impl FnOnce(&str,String)->Result<Credential>)->Result<Credential> {
    let c=store.credential(id)?;
    if c.expires_at>now()+120 {return Ok(c)}
    let token=request(&c.refresh_token)?;
    let access=text(&token,"access_token")?;
    let mut rotated=c.clone();
    if let Some(refresh)=token["refresh_token"].as_str().filter(|s|!s.is_empty()) {rotated.refresh_token=refresh.into();}
    store.replace(&c,rotated.clone())?;
    let fresh=exchange(access,rotated.refresh_token.clone())?;
    if fresh.id!=id {return Err("Microsoft sign-in returned a different account; sign in again".into())}
    store.replace(&rotated,fresh.clone())?;Ok(fresh)
}

#[cfg(test)] mod tests {
    #[test] fn missing_client_id_is_rejected() {
        assert_eq!(super::require_client_id().is_err(), super::CLIENT_ID.trim().is_empty());
    }

    use super::*;
    #[test] fn entitlement_shape_does_not_invent_subscription_policy() {
        assert!(validate_entitlements(&json!({"items":[]})).is_ok());
        assert!(validate_entitlements(&json!({"items":[{"name":"game_minecraft"}]})).is_ok());
        assert!(validate_entitlements(&json!({"items":[{"name":"future_product"}]})).is_ok());
        for value in [json!({}),json!({"items":null}),json!({"items":{}}),json!({"items":[{}]}),json!({"items":[{"name":""}]})] {
            assert!(validate_entitlements(&value).is_err());
        }
    }
    #[test] fn restricted_expired_and_missing_profile_errors_are_distinct() {
        for code in ["invalid_grant","interaction_required","login_required"] {
            assert!(failure("Microsoft",400,&json!({"error":code})).contains("sign in with Microsoft again"));
        }
        assert!(failure("Microsoft",400,&json!({"error":"expired_token"})).contains("code expired"));
        assert!(failure("Xbox XSTS",401,&json!({"XErr":2148916238_u64})).contains("family authorization"));
        assert!(failure("Minecraft profile",404,&json!({})).contains("No Minecraft Java profile"));
        assert!(failure("Minecraft entitlements",401,&json!({"access_token":"secret"})).contains("HTTP 401"));
    }
    #[test] fn rejected_refresh_cannot_replace_expired_credentials() {
        let (_dir,store,c)=store();
        let result=refresh_credential(&store,&c.id,|_|Err("Sign-in expired or was revoked; sign in with Microsoft again".into()),|_,_|panic!("must not exchange a rejected token"));
        assert!(result.is_err());
        assert_eq!(store.credential(&c.id).unwrap().expires_at,0);
        assert_eq!(store.credential(&c.id).unwrap().refresh_token,"old");
    }
    fn waiting(session:&MicrosoftLogin)->u64 {
        let mut state=session.0.lock().unwrap();state.generation+=1;state.status.running=true;state.status.phase="waiting".into();
        state.pending=Pending{address:Some("https://login.live.com/x".into()),state:Some("abc".into()),code:None};state.generation
    }
    #[test] fn cancelled_login_has_no_browser_destination() {
        let session=MicrosoftLogin::default();
        assert!(authorization_address(&session).is_err());
        let id=waiting(&session);
        assert_eq!(authorization_address(&session).unwrap(),(id,"https://login.live.com/x".into()));
        cancel_microsoft_login(&session).unwrap();
        assert!(authorization_address(&session).is_err());
    }
    #[test] fn redirect_requires_matching_state_and_live_attempt() {
        let session=MicrosoftLogin::default();
        assert!(is_redirect("ms-xal-00000000402b5328://auth/?code=x&state=abc"));
        assert!(!is_redirect("https://login.live.com/oauth20_desktop.srf?code=x"));
        assert!(deliver_redirect(&session,"ms-xal-00000000402b5328://auth/?code=x&state=abc").is_err());
        let id=waiting(&session);
        assert!(deliver_redirect(&session,"ms-xal-00000000402b5328://auth/?code=x&state=evil").is_err());
        assert!(deliver_redirect(&session,"ms-xal-00000000402b5328://other/?code=x&state=abc").is_err());
        deliver_redirect(&session,"ms-xal-00000000402b5328://auth/?code=M.C1&state=abc").unwrap();
        assert!(deliver_redirect(&session,"ms-xal-00000000402b5328://auth/?code=again&state=abc").is_err());
        abandon_authorization(&session,id);
        assert_eq!(session.wait_code(id,Instant::now()).unwrap(),"M.C1");
        let id=waiting(&session);
        deliver_redirect(&session,"ms-xal-00000000402b5328://auth/?error=access_denied&state=abc").unwrap();
        assert!(session.wait_code(id,Instant::now()).unwrap_err().contains("declined"));
        let id=waiting(&session); abandon_authorization(&session,id-1); assert!(session.wait_code(id,Instant::now()).unwrap_err().contains("expired"));
        abandon_authorization(&session,id); assert!(session.wait_code(id,Instant::now()).unwrap_err().contains("closed"));
    }
    #[test] fn xbox_times_and_signatures_follow_the_protocol() {
        assert_eq!(parse_time("2026-11-20T08:30:15.1234567Z"),Some(1795163415));
        assert_eq!(parse_time("1970-01-01T00:00:00Z"),Some(0));
        assert_eq!(parse_time("garbage"),None);
        let (id,key)=new_key().unwrap();
        assert!(id.starts_with('{') && id.ends_with('}') && id.len()==38);
        let raw=STANDARD.decode(signature(&key,"/authenticate",b"{}",1_700_000_000)).unwrap();
        assert_eq!(raw.len(),76); assert_eq!(&raw[..4],&[0,0,0,1]);
        let time=(1_700_000_000_u64+11644473600)*10_000_000; assert_eq!(&raw[4..12],&time.to_be_bytes());
        use p256::ecdsa::signature::Verifier;
        let mut buffer=vec![0,0,0,1,0];buffer.extend_from_slice(&time.to_be_bytes());buffer.extend_from_slice(b"\0POST\0/authenticate\0\0{}\0");
        key.verifying_key().verify(&buffer,&Signature::from_slice(&raw[12..]).unwrap()).unwrap();
    }
    #[test] fn errors_do_not_expose_server_payloads() {
        let body=json!({"access_token":"secret","error_description":"secret","XErr":2148916233_u64});
        assert!(failure("Xbox",401,&body).contains("Xbox profile"));
        assert!(!failure("Microsoft",500,&body).contains("secret"));
        assert!(failure("Minecraft",403,&json!({})).contains("approval"));
    }
    #[test] fn cancelled_or_superseded_sessions_cannot_publish() {
        let s=MicrosoftLogin::default(); {let mut v=s.0.lock().unwrap();v.generation=2;v.status.running=true;}
        assert!(s.update(1,|_|panic!()).is_err());assert!(s.update(2,|v|v.running=false).is_ok());assert!(s.update(2,|_|panic!()).is_err());
    }
    fn store() -> (tempfile::TempDir,AccountStore,Credential) {
        let dir=tempfile::tempdir().unwrap();
        let store=AccountStore(Arc::new(Mutex::new(dir.path().join("accounts.json"))),Arc::new(Mutex::new(())));
        let c=Credential{id:"a".repeat(32),name:"Alex".into(),refresh_token:"old".into(),access_token:"expired".into(),expires_at:0,xuid:String::new()};
        store.add(c.clone()).unwrap(); (dir,store,c)
    }
    #[test] fn rotated_refresh_token_survives_downstream_failure_and_deleted_account_stays_deleted() {
        let (_dir,store,c)=store();
        let result=refresh_credential(&store,&c.id,|_|Ok(json!({"access_token":"ms","refresh_token":"rotated"})),|_,_|Err("Xbox unavailable".into()));
        assert!(result.is_err());assert_eq!(store.credential(&c.id).unwrap().refresh_token,"rotated");
        let result=refresh_credential(&store,&c.id,|_|Ok(json!({"access_token":"ms"})),|_,refresh|{
            let mut public=store.load().unwrap();public.microsoft.clear();public.selected=None;store.save(public).unwrap();
            Ok(Credential{refresh_token:refresh,..c.clone()})
        });
        assert!(result.is_err());assert!(store.credential(&c.id).is_err());
    }
    #[test] fn cached_credentials_skip_network_and_refresh_cannot_switch_identity() {
        let (_dir,store,mut c)=store();c.expires_at=now()+3600;store.add(c.clone()).unwrap();
        assert!(refresh_credential(&store,&c.id,|_|panic!("unexpected network"),|_,_|panic!()).is_ok());
        c.expires_at=0;store.add(c.clone()).unwrap();
        assert!(refresh_credential(&store,&c.id,|_|Ok(json!({"access_token":"ms"})),|_,_|Ok(Credential{id:"b".repeat(32),..c.clone()})).is_err());
        assert_eq!(store.load().unwrap().microsoft[0].id,c.id);
    }
    #[test] fn xbox_to_minecraft_protocol_and_profile_are_verified() {
        use std::io::{BufRead,BufReader,Write};
        let listener=std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let url=format!("http://{}",listener.local_addr().unwrap());
        let server=std::thread::spawn(move ||{
            let responses=[json!({"UserToken":{"Token":"user"},"TitleToken":{"Token":"title"}}),json!({"Token":"xsts","DisplayClaims":{"xui":[{"uhs":"hash","xid":"123"}]}}),json!({"access_token":"game","expires_in":3600}),json!({"items":[]}),json!({"id":"a".repeat(32),"name":"Alex"})];
            for (i,data) in responses.into_iter().enumerate() {
                let (mut stream,_)=listener.accept().unwrap();stream.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
                let mut reader=BufReader::new(stream.try_clone().unwrap());let mut headers=String::new();
                loop {let mut line=String::new();reader.read_line(&mut line).unwrap();if line=="\r\n" {break}headers.push_str(&line);}
                let length=headers.lines().find_map(|line|line.to_lowercase().strip_prefix("content-length:").map(|n|n.trim().parse::<usize>().unwrap())).unwrap_or(0);
                let mut body=vec![0;length];reader.read_exact(&mut body).unwrap();
                if i<3 {let v:Value=serde_json::from_slice(&body).unwrap();
                    assert_eq!(headers.to_lowercase().contains("signature:"),i<2);
                    match i {0=>{assert_eq!(v["AccessToken"],"t=ms");assert_eq!(v["SessionId"],"sid");assert_eq!(v["DeviceToken"],"device");},1=>{assert_eq!(v["Properties"]["SandboxId"],"RETAIL");assert_eq!(v["Properties"]["UserTokens"][0],"user");assert_eq!(v["Properties"]["TitleToken"],"title");},_=>assert_eq!(v["identityToken"],"XBL3.0 x=hash;xsts")}
                } else {assert!(headers.to_lowercase().contains("authorization: bearer game"));}
                let body=data.to_string();write!(stream,"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",body.len(),body).unwrap();
            }
        });
        let client=Client::builder().no_proxy().timeout(Duration::from_secs(5)).build().unwrap();
        let (id,key)=new_key().unwrap();let device=Device{id,key,token:"device".into()};
        let credential=exchange_at(&client,&device,"ms","refresh".into(),Some("sid"),[&url,&url,&url,&url,&url]).unwrap();
        assert_eq!(credential.name,"Alex");assert_eq!(credential.access_token,"game");assert_eq!(credential.xuid,"123");assert!(credential.expires_at>now()+3500);
        server.join().unwrap();
    }

    #[test] fn uploads_multipart_and_checks_account_model_and_failure() {
        use std::io::{BufRead, BufReader, Write};
        for (model, status, id, variant, success) in [
            ("default", 200, "a", "CLASSIC", true), ("slim", 200, "a", "SLIM", true),
            ("slim", 200, "b", "SLIM", false), ("slim", 200, "a", "CLASSIC", false),
            ("slim", 401, "a", "SLIM", false), ("slim", 429, "a", "SLIM", false),
        ] {
            let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
            let endpoint = format!("http://{}/minecraft/profile/skins", listener.local_addr().unwrap());
            let server = std::thread::spawn(move || {
                let (mut stream, _) = listener.accept().unwrap();
                stream.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
                let mut reader = BufReader::new(stream.try_clone().unwrap());
                let mut headers = String::new();
                loop { let mut line = String::new(); reader.read_line(&mut line).unwrap(); if line == "\r\n" { break; } headers.push_str(&line); }
                let lower = headers.to_lowercase();
                assert!(headers.starts_with("POST /minecraft/profile/skins "));
                assert!(lower.contains("authorization: bearer private-token"));
                assert!(lower.contains("multipart/form-data; boundary="));
                let length: usize = lower.lines().find_map(|line| line.strip_prefix("content-length:")).unwrap().trim().parse().unwrap();
                let mut bytes = vec![0; length]; reader.read_exact(&mut bytes).unwrap();
                let body = String::from_utf8_lossy(&bytes);
                assert!(body.contains("name=\"variant\""));
                assert!(body.contains(if model == "slim" { "\r\nslim\r\n" } else { "\r\nclassic\r\n" }));
                assert!(body.contains("filename=\"skin.png\""));
                assert!(body.contains("Content-Type: image/png"));
                assert!(bytes.windows(8).any(|w| w == b"\x89PNG\r\n\x1a\n"));
                let response = json!({"id":id.repeat(32),"skins":[{"state":"ACTIVE","variant":variant}]}).to_string();
                write!(stream,"HTTP/1.1 {status} Test\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",response.len(),response).unwrap();
            });
            let client = Client::builder().no_proxy().timeout(Duration::from_secs(5)).build().unwrap();
            let identity = Identity { name:"Alex".into(), uuid:"a".repeat(32), access_token:"private-token".into(), xuid:String::new() };
            let result = upload_skin_at(&client, &endpoint, &identity, model, include_bytes!("../../src/assets/skins/steve.png").to_vec());
            server.join().unwrap(); assert_eq!(result.is_ok(), success);
            if let Err(error) = result { assert!(!error.contains("private-token")); }
        }
    }
}
