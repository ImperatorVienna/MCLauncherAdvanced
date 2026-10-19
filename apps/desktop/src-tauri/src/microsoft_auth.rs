//! Device authorization lives in the native process. No password, client secret,
//! device token, refresh token, or game access token is exposed to the renderer.
use crate::accounts::{AccountStore, Credential};
use reqwest::blocking::{Client, RequestBuilder};
use serde::Serialize;
use serde_json::{json, Value};
use std::{io::Read, sync::{Arc, Mutex}, time::{Duration, Instant, SystemTime, UNIX_EPOCH}};
type Result<T> = std::result::Result<T, String>;
pub const CLIENT_ID: &str = "88ad9fa2-9ee4-4bc9-a8d6-be3ddedd70d7";
const AUTHORITY: &str = "https://login.microsoftonline.com/consumers/oauth2/v2.0";
const SCOPE: &str = "XboxLive.signin offline_access";
const VERIFY: &str = "https://www.microsoft.com/link";
const NETWORK: &str = "Cannot reach sign-in services; check your connection and retry";
pub fn now() -> u64 { SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() }
fn client() -> Result<Client> {
    Client::builder().timeout(Duration::from_secs(25)).connect_timeout(Duration::from_secs(10))
        .redirect(reqwest::redirect::Policy::none()).user_agent("mclauncher-advanced/0.1.0")
        .build().map_err(|_|NETWORK.into())
}
fn response(request: RequestBuilder) -> Result<(u16,Value)> {
    let res=request.send().map_err(|_|NETWORK)?; let status=res.status().as_u16();
    let mut bytes=Vec::new(); res.take(1024*1024+1).read_to_end(&mut bytes).map_err(|_|NETWORK)?;
    if bytes.len()>1024*1024 {return Err("Invalid response from sign-in service".into())}
    let data=serde_json::from_slice(&bytes).map_err(|_|"Invalid response from sign-in service")?;
    Ok((status,data))
}
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
    match data["XErr"].as_u64() {
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
// Structural validation only. An empty store list does not establish whether a
// subscription is active; profile access must still succeed before persistence.
fn validate_entitlements(value:&Value)->Result<()> {
    let items=value["items"].as_array().ok_or("Incomplete response from sign-in service")?;
    for item in items { text(item,"name")?; }
    Ok(())
}
fn exchange(client:&Client,access:&str,refresh:String)->Result<Credential> {
    exchange_at(client,access,refresh,["https://user.auth.xboxlive.com/user/authenticate","https://xsts.auth.xboxlive.com/xsts/authorize","https://api.minecraftservices.com/authentication/login_with_xbox","https://api.minecraftservices.com/entitlements/mcstore","https://api.minecraftservices.com/minecraft/profile"])
}
fn exchange_at(client:&Client,access:&str,refresh:String,urls:[&str;5])->Result<Credential> {
    let xbox=checked(client.post(urls[0]).header("x-xbl-contract-version","1").json(&json!({
        "Properties":{"AuthMethod":"RPS","SiteName":"user.auth.xboxlive.com","RpsTicket":format!("d={access}")},
        "RelyingParty":"http://auth.xboxlive.com","TokenType":"JWT"
    })),"Xbox")?;
    let xsts=checked(client.post(urls[1]).header("x-xbl-contract-version","1").json(&json!({
        "Properties":{"SandboxId":"RETAIL","UserTokens":[text(&xbox,"Token")?]},
        "RelyingParty":"rp://api.minecraftservices.com/","TokenType":"JWT"
    })),"Xbox XSTS")?;
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
#[derive(Default)]
struct Session { generation:u64, status:LoginStatus }
#[derive(Clone,Default)]
pub struct MicrosoftLogin(Arc<Mutex<Session>>);
impl MicrosoftLogin {
    fn update(&self,id:u64,callback:impl FnOnce(&mut LoginStatus))->Result<()> {
        let mut s=self.0.lock().map_err(|_|"Sign-in state is unavailable")?;
        if s.generation!=id || !s.status.running {return Err("Sign-in cancelled".into())}
        callback(&mut s.status); Ok(())
    }
    fn wait(&self,id:u64,seconds:u64,deadline:Instant)->Result<()> {
        let until=Instant::now()+Duration::from_secs(seconds);
        loop {
            self.update(id, |_|{})?;
            if Instant::now()>=deadline {return Err("The sign-in code expired; start sign-in again".into())}
            if Instant::now()>=until {return Ok(())}
            std::thread::sleep(Duration::from_millis(200));
        }
    }
    fn finish(&self,id:u64,store:&AccountStore,credential:Credential)->Result<()> {
        let mut s=self.0.lock().map_err(|_|"Sign-in state is unavailable")?;
        if s.generation!=id || !s.status.running {return Err("Sign-in cancelled".into())}
        store.add(credential)?;
        s.status=LoginStatus {completed:true,..Default::default()}; Ok(())
    }
}
fn login(session:&MicrosoftLogin,id:u64,store:&AccountStore)->Result<()> {
    let client=client()?;
    let device=checked(client.post(format!("{AUTHORITY}/devicecode")).form(&[("client_id",CLIENT_ID),("scope",SCOPE)]),"Microsoft")?;
    let code=text(&device,"device_code")?;
    let user_code=text(&device,"user_code")?.to_owned();
    let uri=text(&device,"verification_uri")?;
    if ![VERIFY,"https://microsoft.com/link","https://www.microsoft.com/devicelogin","https://microsoft.com/devicelogin"].contains(&uri) {return Err("Unexpected Microsoft verification address".into())}
    let expires=device["expires_in"].as_u64().filter(|v|*v>0 && *v<=1800).ok_or("Invalid sign-in code expiry")?;
    let deadline=Instant::now()+Duration::from_secs(expires);
    let mut interval=device["interval"].as_u64().unwrap_or(5).clamp(5,60);
    session.update(id,|s| {s.phase="waiting".into();s.user_code=Some(user_code);s.verification_uri=Some(uri.into());s.expires_at=Some(now()+expires);})?;
    loop {
        session.wait(id,interval,deadline)?;
        let (status,data)=response(client.post(format!("{AUTHORITY}/token")).form(&[("client_id",CLIENT_ID),("grant_type","urn:ietf:params:oauth:grant-type:device_code"),("device_code",code)]))?;
        if (200..300).contains(&status) {
            session.update(id,|s|{s.phase="verifying".into();s.user_code=None;s.verification_uri=None;})?;
            let credential=exchange(&client,text(&data,"access_token")?,text(&data,"refresh_token")?.into())?;
            return session.finish(id,store,credential);
        }
        match data["error"].as_str() {
            Some("authorization_pending")=>{},
            Some("slow_down")=>interval=(interval+5).min(60),
            _=>return Err(failure("Microsoft",status,&data)),
        }
    }
}
pub fn start_microsoft_login(session:&MicrosoftLogin,store:&AccountStore)->Result<()> {
    let id={let mut s=session.0.lock().map_err(|_|"Sign-in state is unavailable")?;
        if s.status.running {return Err("Sign-in is already running".into())}
        s.generation+=1;s.status=LoginStatus {running:true,phase:"starting".into(),..Default::default()};s.generation};
    let session=session.clone();let store=store.clone();let failure_session=session.clone();
    if std::thread::Builder::new().name("microsoft-login".into()).spawn(move || {
        let outcome=std::panic::catch_unwind(std::panic::AssertUnwindSafe(||login(&session,id,&store))).unwrap_or_else(|_|Err("Sign-in stopped unexpectedly; retry".into()));
        if let Err(error)=outcome {let _=session.update(id,|s|{*s=LoginStatus {error:Some(error),..Default::default()};});}
    }).is_err() {let _=failure_session.update(id,|s|{s.running=false;});return Err("Cannot start sign-in".into())}
    Ok(())
}
pub fn microsoft_login_status(session:&MicrosoftLogin)->Result<LoginStatus> {Ok(session.0.lock().map_err(|_|"Sign-in state is unavailable")?.status.clone())}
pub fn cancel_microsoft_login(session:&MicrosoftLogin)->Result<()> {
    let mut s=session.0.lock().map_err(|_|"Sign-in state is unavailable")?;s.generation+=1;s.status=LoginStatus::default();Ok(())
}
pub fn verification_address(session:&MicrosoftLogin)->Result<String> {
    let s=session.0.lock().map_err(|_|"Sign-in state is unavailable")?;
    let uri=s.status.verification_uri.as_ref().filter(|_|s.status.running).ok_or("Start sign-in first")?;
    Ok(uri.clone())
}
pub struct Identity {pub name:String,pub uuid:String,pub access_token:String,pub xuid:String}
pub fn identity(store:&AccountStore,selected:&str)->Result<Identity> {
    let id=selected.strip_prefix('@').filter(|id|crate::accounts::valid_id(id)).ok_or("Sign in before launching the game")?;
    let _guard=store.1.lock().map_err(|_|"Account refresh is unavailable")?;
    let client=client()?;
    let c=refresh_credential(store,id,|refresh| checked(client.post(format!("{AUTHORITY}/token")).form(&[("client_id",CLIENT_ID),("scope",SCOPE),("grant_type","refresh_token"),("refresh_token",refresh)]),"Microsoft"),|access,refresh|exchange(&client,access,refresh))?;
    Ok(Identity{name:c.name,uuid:c.id,access_token:c.access_token,xuid:c.xuid})
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
    #[test] fn cancelled_login_has_no_browser_destination() {
        let session=MicrosoftLogin::default();
        assert!(verification_address(&session).is_err());
        { let mut state=session.0.lock().unwrap();state.status.running=true;state.status.verification_uri=Some(VERIFY.into()); }
        assert_eq!(verification_address(&session).unwrap(),VERIFY);
        cancel_microsoft_login(&session).unwrap();
        assert!(verification_address(&session).is_err());
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
            let responses=[json!({"Token":"xbox"}),json!({"Token":"xsts","DisplayClaims":{"xui":[{"uhs":"hash","xid":"123"}]}}),json!({"access_token":"game","expires_in":3600}),json!({"items":[]}),json!({"id":"a".repeat(32),"name":"Alex"})];
            for (i,data) in responses.into_iter().enumerate() {
                let (mut stream,_)=listener.accept().unwrap();stream.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
                let mut reader=BufReader::new(stream.try_clone().unwrap());let mut headers=String::new();
                loop {let mut line=String::new();reader.read_line(&mut line).unwrap();if line=="\r\n" {break}headers.push_str(&line);}
                let length=headers.lines().find_map(|line|line.to_lowercase().strip_prefix("content-length:").map(|n|n.trim().parse::<usize>().unwrap())).unwrap_or(0);
                let mut body=vec![0;length];reader.read_exact(&mut body).unwrap();
                if i<3 {let v:Value=serde_json::from_slice(&body).unwrap();
                    match i {0=>assert_eq!(v["Properties"]["RpsTicket"],"d=ms"),1=>{assert_eq!(v["Properties"]["SandboxId"],"RETAIL");assert_eq!(v["Properties"]["UserTokens"][0],"xbox");},_=>assert_eq!(v["identityToken"],"XBL3.0 x=hash;xsts")}
                } else {assert!(headers.to_lowercase().contains("authorization: bearer game"));}
                let body=data.to_string();write!(stream,"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",body.len(),body).unwrap();
            }
        });
        let client=Client::builder().no_proxy().timeout(Duration::from_secs(5)).build().unwrap();
        let credential=exchange_at(&client,"ms","refresh".into(),[&url,&url,&url,&url,&url]).unwrap();
        assert_eq!(credential.name,"Alex");assert_eq!(credential.access_token,"game");assert_eq!(credential.xuid,"123");assert!(credential.expires_at>now()+3500);
        server.join().unwrap();
    }

}
