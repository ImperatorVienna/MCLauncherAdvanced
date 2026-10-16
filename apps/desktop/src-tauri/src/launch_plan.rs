//! Converts installed vanilla metadata to argv. Never invokes a shell.
use launcher_core::{Instance, launch_settings::DisplayMode};
use launcher_install::{safe_path, minecraft};
use serde_json::Value;
use std::{collections::HashMap, path::{Path, PathBuf}};
type Result<T> = std::result::Result<T, String>;
pub struct Plan { pub directory: PathBuf, pub args: Vec<String> }
fn field<'a>(v: &'a Value, key: &str) -> Result<&'a str> { v[key].as_str().ok_or_else(|| format!("Missing launch metadata: {key}")) }
fn existing(root: &Path, relative: &str) -> Result<PathBuf> {
    let p = safe_path(root, relative)?;
    if !p.is_file() { return Err(format!("Game file is missing; repair the instance: {relative}")); }
    Ok(p)
}
fn substitute(value: &str, variables: &HashMap<&str, String>) -> Result<String> {
    let mut output = value.to_owned();
    for (name, replacement) in variables { output = output.replace(&format!("${{{name}}}"), replacement); }
    if output.contains("${") { return Err(format!("Unsupported launch argument: {value}")); }
    if output.contains('\0') { return Err("Invalid launch argument".into()); }
    Ok(output)
}
fn arguments(list: &Value, kernel: &str, variables: &HashMap<&str, String>, custom_size: bool) -> Result<Vec<String>> {
    let mut result = vec![];
    for arg in list.as_array().ok_or("Invalid launch argument list")? {
        if let Some(text) = arg.as_str() { result.push(substitute(text, variables)?); continue; }
        let mut rule = arg.clone();
        // Reuse the installer's OS/rule evaluator, with launch-time feature values.
        if let Some(rules) = rule.get_mut("rules").and_then(Value::as_array_mut) {
            for rule in rules {
                if let Some(features) = rule.get_mut("features").and_then(Value::as_object_mut) {
                    if let Some(expected) = features.get_mut("has_custom_resolution") {
                        let matches = expected.as_bool() == Some(custom_size);
                        *expected = Value::Bool(!matches);
                    }
                }
            }
        }
        if !minecraft::allowed(&rule, kernel)? { continue; }
        match &arg["value"] {
            Value::String(text) => result.push(substitute(text, variables)?),
            Value::Array(values) => for text in values { result.push(substitute(text.as_str().ok_or("Invalid launch argument")?, variables)?); },
            _ => return Err("Invalid launch argument".into()),
        }
    }
    Ok(result)
}
#[cfg(test)]
pub fn build(root: &Path, item: &Instance, metadata: &Value, player: &str, uuid: &str, memory_mib: u64, kernel: &str) -> Result<Plan> {
    build_authenticated(root,item,metadata,&crate::microsoft_auth::Identity {name:player.into(),uuid:uuid.into(),access_token:"test-token".into(),xuid:String::new()},memory_mib,kernel)
}
pub fn build_authenticated(root: &Path, item: &Instance, metadata: &Value, identity: &crate::microsoft_auth::Identity, memory_mib: u64, kernel: &str) -> Result<Plan> {
    let player = &identity.name; let uuid = &identity.uuid;
    item.launch_settings.validate()?;
    if metadata.get("inheritsFrom").is_some() { return Err("Inherited or modded versions are not supported yet".into()); }
    let stored_loader: Option<launcher_core::ModLoader> = metadata.get("launcherLoader").map(|v| serde_json::from_value(v.clone())).transpose().map_err(|_| "Invalid mod loader selection")?;
    if stored_loader != item.loader { return Err("Installed mod loader does not match the instance; repair it".into()); }
    let directory = minecraft::instance_directory(root, item.directory())?;
    let jar = existing(root, &format!("versions/{0}/{0}.jar", item.directory()))?;
    let natives = safe_path(root, &format!("versions/{}/natives", item.directory()))?;
    if !natives.is_dir() { return Err("Native libraries are missing; repair the instance".into()); }
    let mut classpath = vec![];
    for lib in metadata["libraries"].as_array().ok_or("Minecraft version has no library list")? {
        if !minecraft::allowed(lib, kernel)? { continue; }
        if let Some(artifact) = lib["downloads"].get("artifact") {
            classpath.push(existing(root, &format!("libraries/{}", field(artifact, "path")?))?);
        } else if lib.get("natives").is_none() {
            return Err("Unsupported library metadata; repair the instance".into());
        }
    }
    classpath.push(jar);
    if classpath.iter().any(|p| p.to_string_lossy().contains(launcher_platform::CLASSPATH_SEPARATOR)) { return Err("Classpath paths cannot contain the platform separator".into()); }
    let assets_id = field(&metadata["assetIndex"], "id")?;
    let index: Value = serde_json::from_slice(&std::fs::read(existing(root, &format!("assets/indexes/{assets_id}.json"))?).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
    let assets = safe_path(root, "assets")?;
    let game_assets = if index["map_to_resources"] == true { directory.join("resources") } else if index["virtual"] == true { safe_path(root, &format!("assets/virtual/{assets_id}"))? } else { assets.clone() };
    let path = launcher_platform::child_path;
    let variables = HashMap::from([
        ("auth_player_name", player.clone()), ("auth_uuid", uuid.replace('-', "")),
        ("auth_access_token", identity.access_token.clone()), ("auth_session", format!("token:{}:{}",identity.access_token,uuid.replace('-',""))),
        ("user_type", "msa".into()), ("user_properties", "{}".into()),
        ("clientid", crate::microsoft_auth::CLIENT_ID.into()), ("auth_xuid", identity.xuid.clone()),
        ("version_name", item.directory_name.clone()), ("version_type", field(metadata, "type")?.into()),
        ("game_directory", path(&directory)), ("assets_root", path(&assets)), ("game_assets", path(&game_assets)),
        ("assets_index_name", assets_id.into()), ("natives_directory", path(&natives)),
        ("launcher_name", "mclauncher-advanced".into()), ("launcher_version", env!("CARGO_PKG_VERSION").into()),
        ("classpath", classpath.iter().map(|p| path(p)).collect::<Vec<_>>().join(launcher_platform::CLASSPATH_SEPARATOR)),
        ("classpath_separator", launcher_platform::CLASSPATH_SEPARATOR.into()), ("library_directory", path(&root.join("libraries"))),
        ("resolution_width", item.launch_settings.width.to_string()), ("resolution_height", item.launch_settings.height.to_string()),
    ]);
    let custom_size = item.launch_settings.display_mode == DisplayMode::Windowed;
    let mut args = if metadata["arguments"].get("jvm").is_some() {
        arguments(&metadata["arguments"]["jvm"], kernel, &variables, custom_size)?
    } else { vec![format!("-Djava.library.path={}", path(&natives)), "-cp".into(), variables["classpath"].clone()] };
    // Explicit heap limit wins over any manifest-provided defaults.
    args.retain(|a| !a.starts_with("-Xmx") && !a.starts_with("-Xms"));
    args.push(format!("-Xmx{memory_mib}M"));
    if let Some(logging) = metadata["logging"].get("client") {
        let file = existing(root, &format!("assets/log_configs/{}", field(&logging["file"], "id")?))?;
        args.push(field(logging, "argument")?.replace("${path}", &path(&file)));
    }
    args.push(field(metadata, "mainClass")?.into());
    let mut game = if metadata["arguments"].get("game").is_some() {
        arguments(&metadata["arguments"]["game"], kernel, &variables, custom_size)?
    } else {
        field(metadata, "minecraftArguments")?.split_whitespace().map(|a| substitute(a, &variables)).collect::<Result<Vec<_>>>()?
    };
    if custom_size && !game.iter().any(|a| a == "--width") {
        game.extend(["--width".into(), item.launch_settings.width.to_string(), "--height".into(), item.launch_settings.height.to_string()]);
    }
    if item.launch_settings.display_mode == DisplayMode::Fullscreen { game.push("--fullscreen".into()); }
    args.extend(game);
    Ok(Plan { directory, args })
}
/// Set only fullscreen for an explicit mode; preserve all other game preferences.
pub fn apply_display(directory: &Path, mode: &DisplayMode) -> Result<()> {
    if *mode == DisplayMode::Inherit { return Ok(()); }
    let file = safe_path(directory, "options.txt")?;
    let text = match std::fs::read_to_string(&file) { Ok(s) => s, Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(), Err(e) => return Err(e.to_string()) };
    let mut lines: Vec<_> = text.lines().filter(|l| !l.starts_with("fullscreen:")).map(str::to_owned).collect();
    lines.push(format!("fullscreen:{}", *mode == DisplayMode::Fullscreen));
    launcher_core::storage::atomic_write(&file, (lines.join("\n") + "\n").as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    fn variables() -> HashMap<&'static str, String> { HashMap::from([("game_directory", "/tmp/a folder".into()), ("resolution_width", "1280".into())]) }
    #[test] fn arguments_preserve_spaces_and_filter_features_and_platform() {
        let data = json!(["--gameDir", "${game_directory}",
            {"rules":[{"action":"allow","os":{"name":"windows"}}],"value":"windows-only"},
            {"rules":[{"action":"allow","features":{"is_demo_user":true}}],"value":"--demo"},
            {"rules":[{"action":"allow","features":{"has_custom_resolution":true}}],"value":["--width","${resolution_width}"]}]);
        assert_eq!(arguments(&data, "6.1", &variables(), true).unwrap(), ["--gameDir", "/tmp/a folder", "--width", "1280"]);
        assert_eq!(arguments(&data, "6.1", &variables(), false).unwrap(), ["--gameDir", "/tmp/a folder"]);
        assert!(substitute("${unknown}", &variables()).is_err());
    }
    fn fixture() -> (PathBuf, Instance, Value) {
        let root = std::env::temp_dir().join(format!("launcher-plan-{}-{}", std::process::id(), std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        for file in ["versions/test name/test name.jar", "libraries/example.jar", "assets/indexes/test.json", "assets/log_configs/client.xml"] {
            let p = root.join(file); std::fs::create_dir_all(p.parent().unwrap()).unwrap(); std::fs::write(p, b"{}").unwrap();
        }
        std::fs::create_dir_all(root.join("versions/test name/natives")).unwrap();
        let item = Instance { id: "test".into(), name: "test name".into(), directory_name: "test name".into(), minecraft_version: "1.21.1".into(), icon: "grass".into(), java_path: None, loader: None, launch_settings: Default::default() };
        let metadata = json!({"type":"release", "mainClass":"net.minecraft.client.main.Main", "assetIndex":{"id":"test"},
            "libraries":[{"downloads":{"artifact":{"path":"example.jar"}}}],
            "logging":{"client":{"argument":"-Dlog4j.configurationFile=${path}","file":{"id":"client.xml"}}},
            "arguments":{"jvm":["-Djava.library.path=${natives_directory}","-cp","${classpath}"],"game":["--username","${auth_player_name}","--uuid","${auth_uuid}","--accessToken","${auth_access_token}","--gameDir","${game_directory}"]}});
        (root,item,metadata)
    }
    #[test] fn complete_modern_and_legacy_plans_and_missing_files() {
        let (root, mut item, mut metadata) = fixture();
        item.launch_settings.display_mode = DisplayMode::Windowed;
        let plan = build(&root,&item,&metadata,"Steve","1234-5678",2048,"6.1").unwrap();
        assert_eq!(plan.directory,root.join("versions/test name"));
        assert!(plan.args.contains(&"-Xmx2048M".into()));
        assert!(plan.args.windows(2).any(|pair| pair == ["--uuid","12345678"]));
        assert!(plan.args.windows(2).any(|pair| pair == ["--accessToken","test-token"]));
        assert!(plan.args.iter().any(|a| a.contains("example.jar:") && a.ends_with("test name.jar")));
        metadata.as_object_mut().unwrap().remove("arguments");
        metadata["minecraftArguments"] = json!("--username ${auth_player_name} --gameDir ${game_directory}");
        assert!(build(&root,&item,&metadata,"Steve","id",1024,"6.1").unwrap().args.contains(&plan.directory.to_string_lossy().into_owned()));
        std::fs::remove_file(root.join("libraries/example.jar")).unwrap();
        assert!(build(&root,&item,&metadata,"Steve","id",1024,"6.1").is_err());
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test] fn microsoft_identity_is_used_in_modern_and_legacy_arguments() {
        let (root,item,mut metadata)=fixture();
        let identity=crate::microsoft_auth::Identity{name:"Alex".into(),uuid:"a".repeat(32),access_token:"game-token".into(),xuid:"12345".into()};
        metadata["arguments"]["game"]=json!(["--username","${auth_player_name}","--uuid","${auth_uuid}","--accessToken","${auth_access_token}","--userType","${user_type}","--clientId","${clientid}","--xuid","${auth_xuid}"]);
        let plan=build_authenticated(&root,&item,&metadata,&identity,2048,"6.1").unwrap();
        for (flag,value) in [("--username","Alex"),("--accessToken","game-token"),("--userType","msa"),("--xuid","12345"),("--clientId",crate::microsoft_auth::CLIENT_ID)] {
            assert!(plan.args.windows(2).any(|p|p==[flag,value]));
        }
        metadata.as_object_mut().unwrap().remove("arguments");
        metadata["minecraftArguments"]=json!("--session ${auth_session}");
        let plan=build_authenticated(&root,&item,&metadata,&identity,2048,"6.1").unwrap();
        assert!(plan.args.contains(&format!("token:game-token:{}",identity.uuid)));
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test] fn loader_launch_uses_merged_entry_and_rejects_mismatch() {
        let (root,mut item,mut metadata)=fixture();
        item.loader=Some(launcher_core::ModLoader {kind:"fabric".into(),version:"0.16.14".into()});
        assert!(build(&root,&item,&metadata,"Steve","id",1024,"6.1").is_err());
        metadata["launcherLoader"]=serde_json::to_value(&item.loader).unwrap();
        metadata["mainClass"]=json!("net.fabricmc.loader.impl.launch.knot.KnotClient");
        let plan=build(&root,&item,&metadata,"Steve","id",1024,"6.1").unwrap();
        assert!(plan.args.contains(&"net.fabricmc.loader.impl.launch.knot.KnotClient".into()));
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test] fn forge_family_launch_expands_module_paths_and_retains_client_and_universal() {
        for (kind,main) in [("forge","net.minecraftforge.bootstrap.ForgeBootstrap"),("neoforge","cpw.mods.bootstraplauncher.BootstrapLauncher"),("neoforge","net.neoforged.fml.startup.Client")] {
            let (root,mut item,mut metadata)=fixture();
            item.loader=Some(launcher_core::ModLoader{kind:kind.into(),version:"1.0.0".into()});
            let mut profile=json!({"inheritsFrom":"1.21.1","mainClass":main,"arguments":{"game":["--launchTarget","forgeclient"],"jvm":["-DlibraryDirectory=${library_directory}","-p","${library_directory}/module.jar","-DignoreList=${version_name}.jar"]},"libraries":[]});
            for classifier in ["client","universal"] {
                std::fs::write(root.join(format!("libraries/{classifier}.jar")),b"test").unwrap();
                profile["libraries"].as_array_mut().unwrap().push(json!({"name":format!("test:loader:1:{classifier}"),"downloads":{"artifact":{"path":format!("{classifier}.jar")}}}));
            }
            launcher_install::loaders::merge(&mut metadata,&profile,"1.21.1").unwrap();
            metadata["launcherLoader"]=serde_json::to_value(&item.loader).unwrap();
            let plan=build(&root,&item,&metadata,"Steve","id",1024,"6.1").unwrap();
            assert!(plan.args.contains(&main.into()));
            assert!(plan.args.contains(&format!("-DlibraryDirectory={}",root.join("libraries").display())));
            assert!(plan.args.contains(&"-DignoreList=test name.jar".into()));
            let cp=&plan.args[plan.args.iter().position(|a|a=="-cp").unwrap()+1];
            assert!(cp.contains("client.jar") && cp.contains("universal.jar"));
            assert!(!plan.args.iter().any(|a|a.contains("${")));
            std::fs::remove_dir_all(root).unwrap();
        }
    }
    #[test] fn explicit_display_preserves_unrelated_options() {
        let (root,item,_) = fixture(); let dir = root.join("versions").join(item.directory());
        std::fs::write(dir.join("options.txt"), "music:0.5\nfullscreen:true\n").unwrap();
        apply_display(&dir,&DisplayMode::Inherit).unwrap();
        assert!(std::fs::read_to_string(dir.join("options.txt")).unwrap().contains("fullscreen:true"));
        apply_display(&dir,&DisplayMode::Windowed).unwrap();
        assert_eq!(std::fs::read_to_string(dir.join("options.txt")).unwrap(), "music:0.5\nfullscreen:false\n");
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    #[ignore = "requires a local JDK; run explicitly for JVM integration validation"]
    fn real_jvm_uses_generated_arguments_directory_memory_and_exit_status() {
        let (root, mut item, mut metadata) = fixture();
        item.launch_settings.display_mode = DisplayMode::Windowed;
        metadata["mainClass"] = json!("LaunchSmoke");
        metadata.as_object_mut().unwrap().remove("logging");
        let source = root.join("LaunchSmoke.java");
        std::fs::write(&source, r#"public class LaunchSmoke { public static void main(String[] args) { System.out.println(System.getProperty("user.dir")); System.out.println(Runtime.getRuntime().maxMemory()); for (String a: args) System.out.println(a); System.err.println("stderr captured"); if (args[1].equals("Failure")) System.exit(7); } }"#).unwrap();
        assert!(std::process::Command::new("/usr/bin/javac").arg(&source).status().unwrap().success());
        let jar = root.join("versions/test name/test name.jar");
        assert!(std::process::Command::new("/usr/bin/jar").args(["cf"]).arg(jar).arg("-C").arg(&root).arg("LaunchSmoke.class").status().unwrap().success());
        for (player, code) in [("Steve",0),("Failure",7)] {
            let plan = build(&root,&item,&metadata,player,"1234-5678",256,"6.1").unwrap();
            let log = root.join(format!("{player}.log"));
            let mut child = crate::launch_commands::spawn_java("/usr/bin/java", &plan, std::fs::File::create(&log).unwrap()).unwrap();
            assert_eq!(child.wait().unwrap().code(), Some(code));
            let output = std::fs::read_to_string(log).unwrap();
            assert!(output.contains(plan.directory.to_str().unwrap()));
            assert!(output.contains("268435456"));
            assert!(output.contains("--username\n"));
            assert!(output.contains("--uuid\n12345678"));
            assert!(output.contains("--width\n1280"));
            assert!(output.contains("stderr captured"));
        }
        std::fs::remove_dir_all(root).unwrap();
    }

}
